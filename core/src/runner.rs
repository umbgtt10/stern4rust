// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::adoption::baseline::Baseline;
use crate::adoption::baseline_outcome::BaselineOutcome;
use crate::adoption::exclusion_outcome::ExclusionOutcome;
use crate::adoption::exclusion_set::ExclusionSet;
use crate::finding::model::manifest_dependency::ManifestDependency;
use crate::reporting::json_printer::JsonPrinter;
use crate::reporting::offence::Offence;
use crate::reporting::offence_threshold::OffenceThreshold;
use crate::reporting::output_format::OutputFormat;
use crate::reporting::package_roster::PackageRoster;
use crate::reporting::report_printer::ReportPrinter;
use crate::reporting::rule_listing::RuleListing;
use crate::reporting::run_outcome::RunOutcome;
use crate::reporting::scan_totals::ScanTotals;
use crate::rule_registry::RuleRegistry;
use crate::rules::source::header_rule::HeaderRule;
use crate::settings::args::Args;
use crate::settings::config::Config;
use crate::settings::config_file::ConfigFile;
use crate::settings::header_source::HeaderSource;
use crate::settings::manifest_resolver::ManifestResolver;
use crate::settings::package_config::PackageConfig;
use crate::settings::package_sections::PackageSections;
use crate::settings::rule_selection::RuleSelection;
use crate::settings::scanned_package::ScannedPackage;
use crate::source_file::SourceFile;
use crate::source_reader::SourceReader;
use crate::source_walker::SourceWalker;
use crate::test_file_rewriter::TestFileRewriter;
use anyhow::Context;
use anyhow::Result;
use std::collections::HashSet;
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
        let (outcome, report) = Self::run_reporting(args)?;
        println!("{report}");
        Ok(outcome)
    }

    // The same run, handing back what it would have printed.
    //
    // Both per-package bugs left the run perfectly Ok while the report
    // contradicted itself, so a test asserting on the outcome could not see
    // either. This is the seam that lets one assert on what was said.
    pub fn run_reporting(args: Args) -> Result<(RunOutcome, String)> {
        // Before anything is read. The listing answers from the registry alone,
        // so it works in a checkout with no manifest worth reading and cannot
        // fail the way a run can.
        if args.list_rules {
            return Ok((RunOutcome::Clean, Self::rule_listing(&args)));
        }
        let sections = PackageSections::load(&Self::manifest_directory(&args.manifest_path))?;
        let config = Self::config_from(&args, None)?;
        Self::validate_selection(&config)?;
        let config = Config {
            workspace_dependencies: ManifestResolver::workspace_dependencies(&config),
            ..config
        };
        let packages = ManifestResolver::packages(&config)?;
        let workspace = ManifestResolver::workspace_package_names(&config)?;
        sections.validate(&workspace.iter().map(String::as_str).collect::<Vec<_>>())?;
        // What the report answers for. A rule that stood down for any package
        // did not apply to this run, so the licence stated here is the one every
        // scanned package agrees on and nothing otherwise. Checking is per
        // package; only the summary is aggregate, and it understates rather than
        // overstates -- see
        // [ADR-PerPackageConfiguration](../docs/ADRs/ADR-PerPackageConfiguration.md),
        // where the per-package report is the piece still to come.
        let config = Config {
            manifest_license: ScannedPackage::agreed_license(&packages),
            selection: config.selection.also_skipping(
                &sections.skipped_anywhere(
                    &packages
                        .iter()
                        .map(|package| package.name.as_str())
                        .collect::<Vec<_>>(),
                ),
            ),
            ..config
        };
        let registry = RuleRegistry::from_config(&config);
        if registry.is_empty() {
            return Err(anyhow::anyhow!(
                "no rules are configured, so nothing would be checked -- pass --header-file to \
                 enable the header rule"
            ));
        }

        let mut offences = Vec::new();
        let mut files_scanned = 0usize;
        let mut excluded = Vec::new();
        let mut fixed = 0usize;
        let mut rosters: Vec<PackageRoster> = Vec::new();
        // Read once: every manifest path in `workspace_dependencies` is stated
        // relative to it.
        let workspace_root = ManifestResolver::workspace_root(&config).unwrap_or_default();

        for package in &packages {
            // Everything the manifest decides is decided here, by the package
            // about to be walked, rather than once for the run -- including
            // which manifest declarations this package is answerable for.
            //
            // The whole workspace's declarations used to be handed to every
            // package, and `check_workspace` runs once per package, so each
            // finding was stated once per member. Twenty real findings became
            // 580 in `etheram-ibft-embassy`, which has twenty-nine of them, and
            // the report gave no sign: every copy was identical, so the count
            // simply tracked the member count.
            //
            // Filtering here rather than deduplicating afterwards, because a
            // finding about `alpha/Cargo.toml` belongs to `alpha` and to no
            // other package. Deduplication would have to guess that; the loop
            // already knows it.
            let package_config = Config {
                manifest_license: package.license.clone(),
                workspace_dependencies: ManifestDependency::in_manifest(
                    &config.workspace_dependencies,
                    &ManifestResolver::relative_to(
                        &workspace_root,
                        &package.root.join(ManifestResolver::MANIFEST),
                    ),
                ),
                ..Self::config_from(&args, sections.of(&package.name))?
            };
            let registry = RuleRegistry::from_config(&package_config);
            rosters.push(PackageRoster::new(
                &package.name,
                Self::owned(&registry.names()),
                Self::owned(&RuleRegistry::skipped_names(&package_config.selection)),
                registry
                    .unconfigured(&package_config)
                    .into_iter()
                    .map(|(name, requirement)| (name.to_string(), requirement.to_string()))
                    .collect(),
            ));
            let exclusions = ExclusionSet::new(&package_config.excludes)?;
            let root = &package.root;
            // Read the whole package before judging it. Rules whose subject is
            // the tree -- "there is exactly one all_tests.rs" -- cannot be
            // answered a file at a time, and the file that carries the offence
            // is often the one that does not exist.
            let outcome = exclusions.apply(SourceWalker::walk(root), root);
            let mut files: Vec<SourceFile> = Vec::new();
            let mut found: Vec<Offence> = Vec::new();
            for path in outcome.kept {
                files_scanned += 1;
                match SourceReader::read(root, &path) {
                    Ok(file) => files.push(file),
                    Err(offence) => found.push(*offence),
                }
            }
            if config.fix {
                let (rewritten, count) = Self::repair(root, files)?;
                files = rewritten;
                fixed += count;
            }
            for file in &files {
                found.extend(registry.check(file));
            }
            found.extend(registry.check_workspace(&files));

            // Deduplicated per package, because a package is the widest scope in
            // which two identical offences are certainly one finding.
            //
            // The workspace question is asked once per package root, so a rule
            // whose subject is the workspace rather than the package can state
            // the same finding twice while walking one member. Those collapse.
            //
            // Across members they must not. A path is relative to its package,
            // so `src/lib.rs` in one member and `src/lib.rs` in another are two
            // real files rendered as one string, and collapsing by content alone
            // threw the second away. Measured on `etheram-embassy`, whose 31
            // members repeat `src/lib.rs` and `tests/all_tests.rs` throughout:
            // 390 offences reported as 364, across four rules, with the summary
            // and the exit code both counting the smaller number. A checker that
            // quietly reports less than it found is the failure this tool exists
            // to refuse, and it was doing it to itself.
            let mut seen = HashSet::new();
            found.retain(|offence| seen.insert(offence.clone()));
            offences.extend(found);
            excluded.push(outcome.excluded);
        }

        // Rules run in registration order and the tree-wide pass runs last, so
        // without this the report jumps between files. Sorting is the report's
        // business rather than any rule's -- a rule states facts, and their
        // order on the page is not one of them.
        offences.sort_by(|left, right| left.sort_key().cmp(&right.sort_key()));

        if config.write_baseline {
            return Self::record(&config, offences).map(|outcome| (outcome, String::new()));
        }
        let baselined = Self::baselined(&config, offences)?;
        let offences = baselined.kept;
        let report = Self::report(
            &config,
            &registry,
            ScanTotals::new(files_scanned, fixed),
            &Self::merged(excluded),
            &BaselineOutcome::new(Vec::new(), baselined.suppressed, baselined.stale),
            &rosters,
            &offences,
        );
        Ok((RunOutcome::of(offences.len()), report))
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
    fn config_from(args: &Args, section: Option<&PackageConfig>) -> Result<Config> {
        let directory = Self::manifest_directory(&args.manifest_path);
        let file = ConfigFile::load(&directory)?;
        let found = file.as_ref();
        let header_file = args
            .header_file
            .clone()
            .or_else(|| section.and_then(|s| s.header_file_from(&directory)))
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
            // Filled in by `run` once the manifest has been read; the command
            // line has nothing to say about it.
            manifest_license: None,
            workspace_dependencies: None,
            // Discovered beside the manifest when nobody named one, the same
            // way stern4rust.toml is. Implicit suppression would be
            // unacceptable if it were invisible; every report that used a
            // baseline names it and states how many offences it hid, so a
            // reader can always see that one is in force.
            baseline: args
                .baseline
                .clone()
                .or_else(|| found.and_then(|file| file.baseline_from(&directory)))
                .or_else(|| Self::discovered_baseline(&directory, args.write_baseline)),
            write_baseline: args.write_baseline,
            fix: args.fix,
            config_file: found.map(|_| directory.join(ConfigFile::NAME)),
            manifest_path: args.manifest_path.clone(),
            max_files_per_directory: section
                .and_then(|s| s.max_files_per_directory)
                .or_else(|| found.and_then(|file| file.max_files_per_directory)),
            max_subfolders_per_directory: section
                .and_then(|s| s.max_subfolders_per_directory)
                .or_else(|| found.and_then(|file| file.max_subfolders_per_directory)),
            packages: args.packages.clone(),
            excludes: Self::preferred(
                args.excludes.clone(),
                Self::section_or_root(section.map(|s| &s.exclude), found.map(|file| &file.exclude)),
            ),
            expected_header,
            format: args.format,
            offence_threshold: OffenceThreshold::new(threshold),
            selection: RuleSelection::new(
                Self::preferred(
                    args.rules.clone(),
                    Self::section_or_root(section.map(|s| &s.rules), found.map(|file| &file.rules)),
                ),
                Self::preferred(
                    args.skipped_rules.clone(),
                    Self::section_or_root(section.map(|s| &s.skip), found.map(|file| &file.skip)),
                ),
            ),
        })
    }

    // Every rule the registry can hold, not the subset this run selected: the
    // reader asking what a rule wants has not chosen one yet.
    //
    // Two rules stay out of a registry until something configures them -- the
    // header rule until it is told what the header says, and
    // spdx-matches-manifest until a manifest declares a licence. Both are
    // handed a stand-in that nothing ever reads, because a listing missing a
    // rule reads as a tool that does not have it. The first draft supplied only
    // the header and quietly listed twenty.
    fn rule_listing(args: &Args) -> String {
        let registry = RuleRegistry::from_config(&Config {
            expected_header: vec![String::new()],
            manifest_license: Some(String::new()),
            ..Config::default()
        });
        RuleListing::new(&registry.explanations()).render(args.format)
    }

    // When writing, the default path is the destination whether or not it
    // exists yet. When reading, only an existing file counts -- otherwise every
    // run without a baseline would fail trying to load one.
    fn discovered_baseline(directory: &Path, writing: bool) -> Option<PathBuf> {
        let path = directory.join(Self::BASELINE_NAME);
        (writing || path.exists()).then_some(path)
    }

    // A section states its whole list rather than adding to the root's, which is
    // the argument already made for the command line against the file, one level
    // down: a reader who wants to know what a package skips reads one list.
    fn section_or_root<'a>(
        section: Option<&'a Vec<String>>,
        root: Option<&'a Vec<String>>,
    ) -> Option<&'a Vec<String>> {
        match section {
            Some(values) if !values.is_empty() => Some(values),
            _ => root,
        }
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
        totals: ScanTotals,
        excluded: &ExclusionOutcome,
        baselined: &BaselineOutcome,
        rosters: &[PackageRoster],
        offences: &[Offence],
    ) -> String {
        let threshold = config.offence_threshold;
        let applied = Self::owned(&registry.names());
        let skipped = Self::owned(&RuleRegistry::skipped_names(&config.selection));
        let unconfigured: Vec<(String, String)> = registry
            .unconfigured(config)
            .into_iter()
            .map(|(name, requirement)| (name.to_string(), requirement.to_string()))
            .collect();
        let unconfigured_names = Self::owned(&registry.unconfigured_names(config));
        match config.format {
            OutputFormat::Text => ReportPrinter::new(totals.files_scanned)
                .with_threshold(threshold)
                .with_rules(applied.clone(), skipped.clone(), unconfigured.clone())
                .with_package_rosters(rosters.to_vec())
                .with_exclusions(excluded.excluded.clone())
                .with_config_file(Self::shown(config))
                .with_baseline(
                    Self::baseline_shown(config),
                    baselined.suppressed,
                    baselined.stale,
                )
                .with_fixed(totals.fixed)
                .render(offences),
            OutputFormat::Json => JsonPrinter::new(totals.files_scanned)
                .with_threshold(threshold)
                .with_rules(applied, skipped, unconfigured_names)
                .with_package_rosters(rosters.to_vec())
                .with_exclusions(excluded.excluded.clone())
                .with_config_file(Self::shown(config))
                .with_baseline(
                    Self::baseline_shown(config),
                    baselined.suppressed,
                    baselined.stale,
                )
                .with_fixed(totals.fixed)
                .render(offences),
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
