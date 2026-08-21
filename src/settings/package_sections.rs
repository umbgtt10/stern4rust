// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use anyhow::Result;
use std::collections::BTreeMap;
use std::path::Path;

use crate::settings::config_file::ConfigFile;
use crate::settings::package_config::PackageConfig;

// The `[package.<name>]` sections of the root `stern4rust.toml`, and the two
// questions asked of them: which section applies to the package about to be
// walked, and does every section name a package this run actually scans.
//
// A type rather than a pair of functions on the runner, because the second
// question only means anything beside the first: a section is either the rule
// set for a member or a name that matches nothing, and nothing else.
//
// See [ADR-PerPackageConfiguration](../../docs/ADRs/ADR-PerPackageConfiguration.md).
pub struct PackageSections {
    sections: BTreeMap<String, PackageConfig>,
}

impl PackageSections {
    pub fn new(sections: BTreeMap<String, PackageConfig>) -> Self {
        Self { sections }
    }

    // No file and a file with no sections are the same answer here. They differ
    // only for `ConfigFile::load`, which has to tell a missing file from an
    // unreadable one.
    pub fn load(directory: &Path) -> Result<Self> {
        Ok(Self::new(
            ConfigFile::load(directory)?
                .map(|file| file.packages)
                .unwrap_or_default(),
        ))
    }

    pub fn of(&self, name: &str) -> Option<&PackageConfig> {
        self.sections.get(name)
    }

    // Every rule any section stands down on.
    //
    // The report answers for the run as a whole, and a rule that did not apply
    // to one package did not apply to the run. Reporting it as applied would be
    // the overstatement this tool exists to refuse: a stand-down is only
    // acceptable while the report names it. Until the report speaks per package,
    // this is what keeps it honest -- it understates, naming a rule as skipped
    // even where most packages applied it.
    //
    // Only the packages this run walks. Counting a section for one it does not
    // made a scoped run contradict itself: the roster listed a rule as applied
    // while the summary beneath it called the same rule skipped.
    pub fn skipped_anywhere(&self, scanned: &[&str]) -> Vec<String> {
        let mut skipped: Vec<String> = self
            .sections
            .iter()
            .filter(|(name, _)| scanned.contains(&name.as_str()))
            .flat_map(|(_, section)| section.skip.iter().cloned())
            .collect();
        skipped.sort();
        skipped.dedup();
        skipped
    }

    pub fn is_empty(&self) -> bool {
        self.sections.is_empty()
    }

    // A section naming no package in the *workspace* is an error, for the reason
    // a misspelled `--rule` name is: it reads as a rule set being applied, and
    // `deny_unknown_fields` cannot catch it because the section name is data
    // rather than a key.
    //
    // The workspace, not the scan. Scoping a run to one package is an ordinary
    // thing to do, and the sections for the others are not typos -- checking
    // against the scan made `--package node` an error in any repository whose
    // root config had sections, which is every repository this exists for.
    pub fn validate(&self, workspace: &[&str]) -> Result<()> {
        let unknown: Vec<&str> = self
            .sections
            .keys()
            .map(String::as_str)
            .filter(|name| !workspace.contains(name))
            .collect();
        if unknown.is_empty() {
            return Ok(());
        }
        Err(anyhow::anyhow!(
            "{} configures package(s) that are not in this workspace: {} -- it holds: {}",
            ConfigFile::NAME,
            unknown.join(", "),
            workspace.join(", ")
        ))
    }
}
