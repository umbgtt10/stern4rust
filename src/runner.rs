// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::fs;
use std::path::Path;

use anyhow::Context;
use anyhow::Result;

use crate::args::Args;
use crate::config::Config;
use crate::header_source::HeaderSource;
use crate::manifest_resolver::ManifestResolver;
use crate::offence::Offence;
use crate::report_printer::ReportPrinter;
use crate::rule_registry::RuleRegistry;
use crate::run_outcome::RunOutcome;
use crate::source_file::SourceFile;
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
pub struct Runner;

impl Runner {
    pub fn run(args: Args) -> Result<RunOutcome> {
        let config = Self::config_from(args)?;
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
            for path in SourceWalker::walk(root) {
                let file = Self::read(root, &path)?;
                files_scanned += 1;
                offences.extend(registry.check(&file));
            }
        }

        Self::report(files_scanned, &offences);
        Ok(RunOutcome::of(offences.len()))
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
        })
    }

    fn read(root: &Path, path: &Path) -> Result<SourceFile> {
        let contents = fs::read_to_string(path)
            .with_context(|| format!("failed to read source file {}", path.display()))?;
        Ok(SourceFile::new(
            &ManifestResolver::relative_to(root, path),
            &contents,
        ))
    }

    fn report(files_scanned: usize, offences: &[Offence]) {
        ReportPrinter::new(files_scanned).print(offences);
    }
}
