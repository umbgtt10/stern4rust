// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::args::Args;
use crate::baseline::Baseline;
use crate::baseline_outcome::BaselineOutcome;
use crate::config::Config;
use crate::config_file::ConfigFile;
use crate::exclusion_outcome::ExclusionOutcome;
use crate::exclusion_set::ExclusionSet;
use crate::header_source::HeaderSource;
use crate::json_printer::JsonPrinter;
use crate::manifest_resolver::ManifestResolver;
use crate::offence::Offence;
use crate::offence_threshold::OffenceThreshold;
use crate::output_format::OutputFormat;
use crate::report_printer::ReportPrinter;
use crate::rule_registry::RuleRegistry;
use crate::rule_selection::RuleSelection;
use crate::rules::header_rule::HeaderRule;
use crate::run_outcome::RunOutcome;
use crate::source_file::SourceFile;
use crate::source_reader::SourceReader;
use crate::source_walker::SourceWalker;
use crate::test_file_rewriter::TestFileRewriter;
use anyhow::Context;
use anyhow::Result;
use std::fs::write as write_file;
use std::path::Path;
use std::path::PathBuf;

// Exit codes are the whole contract with a gate script:
//
//   0  every rule satisfied
//   1  could not run -- returned as an Err and turned into 1 by main
//   2  at least one rule broken
//
// 2 is kept distinct from 1 on purpose. A script that treats every non-zero code
// alike cannot tell "your code has a problem" from "I could not read your code",
// and the second one silently passing is how a gate stops meaning anything.
//
// The line between the two is what can still be enumerated. A bad manifest or an
// unknown package is a 1: without it there is no list of files to judge. A single
// unreadable file is a 2, reported against readable-source like any other
// finding -- it is a fact about the tree, and aborting on it would hide every
// offence already found in every other file.
pub struct Runner;

impl Runner {
    pub fn run(args: Args) -> Result<RunOutcome> {
        let config = Self::config_from(args)?;
        Self::validate_selection(&config)?;
        let registry = RuleRegistry::from_config(&config);
        if registry.is_empty() {
            return Err(anyhow::anyhow!(
                "no rules are configured, so nothing would be checked -- pass --header-file to \
                 enable the header rule"
            ));
        }

        let roots = ManifestResolver::package_roots(&config)?;
        let exclusions = ExclusionSet::new(&config.excludes)?;
        let mut offences = Vec::new();
        let mut files_scanned = 0usize;
        let mut excluded = Vec::new();
        let mut fixed = 0usize;

        for root in &roots {
            // Read the whole package before judging it. Rules whose subject is
            // the tree -- "there is exactly one all_tests.rs" -- cannot be
            // answered a file at a time, and the file that carries the offence
            // is often the one that does not exist.
            let outcome = exclusions.apply(SourceWalker::walk(root), root);
            let mut files: Vec<SourceFile> = Vec::new();
            for path in outcome.kept {
                files_scanned += 1;
                match SourceReader::read(root, &path) {
                    Ok(file) => files.push(file),
                    Err(offence) => offences.push(*offence),
                }
            }
            if config.fix {
                let (rewritten, count) = Self::repair(root, files)?;
                files = rewritten;
                fixed += count;
            }
            for file in &files {
                offences.extend(registry.check(file));
            }
            offences.extend(registry.check_workspace(&files));
            excluded.push(outcome.excluded);
        }

        // Rules run in registration order and the tree-wide pass runs last, so
        // without this the report jumps between files. Sorting is the report's
        // business rather than any rule's -- a rule states facts, and their
        // order on the page is not one of them.
        offences.sort_by(|left, right| left.sort_key().cmp(&right.sort_key()));

        if config.write_baseline {
            return Self::record(&config, offences);
        }
        let baselined = Self::baselined(&config, offences)?;
        let offences = baselined.kept;
        Self::report(
            &config,
            &registry,
            files_scanned,
            &Self::merged(excluded),
            &BaselineOutcome::new(Vec::new(), baselined.suppressed, baselined.stale),
            fixed,
            &offences,
        );
        Ok(RunOutcome::of(offences.len()))
    }

    // A misspelled rule name is an error rather than a switch that quietly
    // matches nothing, and asking for the header rule without a header file is
    // an error rather than an empty run. Both would otherwise look exactly like
    // a run that worked.
    fn validate_selection(config: &Config) -> Result<()> {
        let known = RuleRegistry::known_names();
        let unknown = config.selection.unknown_in(&known);
        if !unknown.is_empty() {
            return Err(anyhow::anyhow!(
                "unknown rule name(s): {} -- the rules are: {}",
                unknown.join(", "),
                known.join(", ")
            ));
        }
        if config.selection.selects_explicitly(HeaderRule::NAME)
            && config.expected_header.is_empty()
        {
            return Err(anyhow::anyhow!(
                "--rule {} needs --header-file, otherwise the run would apply no rules at all",
                HeaderRule::NAME
            ));
        }
        Ok(())
    }

    // The command line wins over the file, every time and per setting. A
    // repository states its defaults in stern4rust.toml; a person overrides one
    // of them for one run without having to restate the rest.
    //
    // "Wins" is replacement rather than merging for the list settings. Merging
    // would make `--rule header` mean "header plus whatever the file already
    // selected", which is the opposite of what naming one rule means everywhere
    // else in this tool.
    fn config_from(args: Args) -> Result<Config> {
        let directory = Self::manifest_directory(&args.manifest_path);
        let file = ConfigFile::load(&directory)?;
        let found = file.as_ref();
        let header_file = args
            .header_file
            .or_else(|| found.and_then(|file| file.header_file_from(&directory)));
        let expected_header = match &header_file {
            Some(path) => HeaderSource::read(path)?,
            None => Vec::new(),
        };
        let threshold = args
            .offence_threshold
            .or_else(|| found.and_then(|file| file.offence_threshold))
            .unwrap_or(OffenceThreshold::DEFAULT);
        Ok(Config {
            // Discovered beside the manifest when nobody named one, the same
            // way stern4rust.toml is. Implicit suppression would be
            // unacceptable if it were invisible; every report that used a
            // baseline names it and states how many offences it hid, so a
            // reader can always see that one is in force.
            baseline: args
                .baseline
                .or_else(|| found.and_then(|file| file.baseline_from(&directory)))
                .or_else(|| Self::discovered_baseline(&directory, args.write_baseline)),
            write_baseline: args.write_baseline,
            fix: args.fix,
            config_file: found.map(|_| directory.join(ConfigFile::NAME)),
            manifest_path: args.manifest_path,
            packages: args.packages,
            excludes: Self::preferred(args.excludes, found.map(|file| &file.exclude)),
            expected_header,
            format: args.format,
            offence_threshold: OffenceThreshold::new(threshold),
            selection: RuleSelection::new(
                Self::preferred(args.rules, found.map(|file| &file.rules)),
                Self::preferred(args.skipped_rules, found.map(|file| &file.skip)),
            ),
        })
    }

    // When writing, the default path is the destination whether or not it
    // exists yet. When reading, only an existing file counts -- otherwise every
    // run without a baseline would fail trying to load one.
    fn discovered_baseline(directory: &Path, writing: bool) -> Option<PathBuf> {
        let path = directory.join(Self::BASELINE_NAME);
        (writing || path.exists()).then_some(path)
    }

    fn preferred(from_args: Vec<String>, from_file: Option<&Vec<String>>) -> Vec<String> {
        if !from_args.is_empty() {
            return from_args;
        }
        from_file.cloned().unwrap_or_default()
    }

    // The config lives beside the manifest it configures, so a workspace and a
    // package in it can hold different ones.
    fn manifest_directory(manifest_path: &Option<PathBuf>) -> PathBuf {
        manifest_path
            .as_ref()
            .and_then(|path| path.parent())
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."))
    }

    pub const BASELINE_NAME: &'static str = "stern4rust-baseline.json";

    // Rewrites what can be rewritten and hands back the files as they now are,
    // so the checks that follow judge the repaired tree. Whatever is still
    // wrong is reported exactly as it would have been without --fix -- a fixer
    // that quietly swallowed what it could not fix would be worse than no fixer
    // at all.
    fn repair(root: &Path, files: Vec<SourceFile>) -> Result<(Vec<SourceFile>, usize)> {
        let mut repaired = Vec::with_capacity(files.len());
        let mut count = 0;
        for file in files {
            match TestFileRewriter::rewrite(&file) {
                Some(contents) => {
                    let path = root.join(file.relative_path());
                    write_file(&path, &contents)
                        .with_context(|| format!("{} could not be rewritten", path.display()))?;
                    repaired.push(SourceFile::new(file.relative_path(), &contents));
                    count += 1;
                }
                None => repaired.push(file),
            }
        }
        Ok((repaired, count))
    }

    // Recording is not judging. The run exits clean because nothing was
    // assessed -- the offences were written down, which is what was asked for.
    fn record(config: &Config, offences: Vec<Offence>) -> Result<RunOutcome> {
        let path = config
            .baseline
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("--write-baseline needs a path to write to"))?;
        let baseline = Baseline::of(&offences);
        baseline.save(path)?;
        println!(
            "stern4rust wrote {} offence(s) to {}",
            baseline.len(),
            path.display()
        );
        Ok(RunOutcome::Clean)
    }

    // A baseline that was asked for and is not there is an error rather than an
    // empty one. A gate whose baseline path has a typo would otherwise report
    // every existing offence and look like a regression.
    fn baselined(config: &Config, offences: Vec<Offence>) -> Result<BaselineOutcome> {
        let Some(path) = &config.baseline else {
            return Ok(BaselineOutcome::new(offences, 0, 0));
        };
        Ok(Baseline::load(path)?.apply(offences))
    }

    // One package's exclusions say nothing on their own: a pattern matching
    // nothing in package A and forty files in package B has done its job, and
    // reporting it as dead for A would be a wrong answer rather than a missing
    // one. So the counts are summed across roots before anybody looks at them.
    fn merged(per_root: Vec<Vec<(String, usize)>>) -> ExclusionOutcome {
        let mut totals: Vec<(String, usize)> = Vec::new();
        for counts in per_root {
            for (pattern, count) in counts {
                match totals.iter_mut().find(|(known, _)| *known == pattern) {
                    Some(entry) => entry.1 += count,
                    None => totals.push((pattern, count)),
                }
            }
        }
        ExclusionOutcome::new(Vec::new(), totals)
    }

    fn report(
        config: &Config,
        registry: &RuleRegistry,
        files_scanned: usize,
        excluded: &ExclusionOutcome,
        baselined: &BaselineOutcome,
        fixed: usize,
        offences: &[Offence],
    ) {
        let threshold = config.offence_threshold;
        let applied = Self::owned(&registry.names());
        let skipped = Self::owned(&RuleRegistry::skipped_names(&config.selection));
        let unconfigured = Self::owned(&registry.unconfigured_names(config));
        match config.format {
            OutputFormat::Text => ReportPrinter::new(files_scanned)
                .with_threshold(threshold)
                .with_rules(applied.clone(), skipped.clone(), unconfigured.clone())
                .with_exclusions(excluded.excluded.clone())
                .with_config_file(Self::shown(config))
                .with_baseline(
                    Self::baseline_shown(config),
                    baselined.suppressed,
                    baselined.stale,
                )
                .with_fixed(fixed)
                .print(offences),
            OutputFormat::Json => JsonPrinter::new(files_scanned)
                .with_threshold(threshold)
                .with_rules(applied, skipped, unconfigured)
                .with_exclusions(excluded.excluded.clone())
                .with_config_file(Self::shown(config))
                .with_baseline(
                    Self::baseline_shown(config),
                    baselined.suppressed,
                    baselined.stale,
                )
                .with_fixed(fixed)
                .print(offences),
        }
    }

    fn baseline_shown(config: &Config) -> Option<String> {
        config
            .baseline
            .as_ref()
            .map(|path| path.to_string_lossy().replace('\\', "/"))
    }

    fn shown(config: &Config) -> Option<String> {
        config
            .config_file
            .as_ref()
            .map(|path| path.to_string_lossy().replace('\\', "/"))
    }

    fn owned(names: &[&str]) -> Vec<String> {
        names.iter().map(|name| (*name).to_string()).collect()
    }
}
