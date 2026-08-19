// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use anyhow::Result;

use crate::args::Args;
use crate::config::Config;
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
use crate::source_reader::SourceReader;
use crate::source_walker::SourceWalker;

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
        let mut offences = Vec::new();
        let mut files_scanned = 0usize;

        for root in &roots {
            // Read the whole package before judging it. Rules whose subject is
            // the tree -- "there is exactly one all_tests.rs" -- cannot be
            // answered a file at a time, and the file that carries the offence
            // is often the one that does not exist.
            let mut files = Vec::new();
            for path in SourceWalker::walk(root) {
                files_scanned += 1;
                match SourceReader::read(root, &path) {
                    Ok(file) => files.push(file),
                    Err(offence) => offences.push(*offence),
                }
            }
            for file in &files {
                offences.extend(registry.check(file));
            }
            offences.extend(registry.check_workspace(&files));
        }

        // Rules run in registration order and the tree-wide pass runs last, so
        // without this the report jumps between files. Sorting is the report's
        // business rather than any rule's -- a rule states facts, and their
        // order on the page is not one of them.
        offences.sort_by(|left, right| left.sort_key().cmp(&right.sort_key()));
        Self::report(&config, &registry, files_scanned, &offences);
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

    fn config_from(args: Args) -> Result<Config> {
        let expected_header = match &args.header_file {
            Some(path) => HeaderSource::read(path)?,
            None => Vec::new(),
        };
        Ok(Config {
            manifest_path: args.manifest_path,
            packages: args.packages,
            expected_header,
            format: args.format,
            offence_threshold: OffenceThreshold::new(args.offence_threshold),
            selection: RuleSelection::new(args.rules, args.skipped_rules),
        })
    }

    fn report(
        config: &Config,
        registry: &RuleRegistry,
        files_scanned: usize,
        offences: &[Offence],
    ) {
        let threshold = config.offence_threshold;
        let applied = Self::owned(&registry.names());
        let skipped = Self::owned(&RuleRegistry::skipped_names(&config.selection));
        match config.format {
            OutputFormat::Text => ReportPrinter::new(files_scanned)
                .with_threshold(threshold)
                .with_rules(applied, skipped)
                .print(offences),
            OutputFormat::Json => JsonPrinter::new(files_scanned)
                .with_threshold(threshold)
                .with_rules(applied, skipped)
                .print(offences),
        }
    }

    fn owned(names: &[&str]) -> Vec<String> {
        names.iter().map(|name| (*name).to_string()).collect()
    }
}
