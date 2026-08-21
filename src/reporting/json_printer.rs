// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::reporting::offence::Offence;
use crate::reporting::offence_threshold::OffenceThreshold;
use crate::reporting::package_roster::PackageRoster;
use serde_json::Value;
use serde_json::json;
use serde_json::to_string_pretty;
use std::collections::BTreeSet;

// The same run as data rather than as a table.
//
// The table is sized to its contents and meant for a person. Nothing can parse
// it reliably -- paths and descriptions both contain spaces, and descriptions
// carry backticks, quotes and semicolons, so splitting on whitespace is
// guesswork. A gate script or an agent reads this instead and never has to
// infer where one column ends and the next begins.
pub struct JsonPrinter {
    files_scanned: usize,
    threshold: OffenceThreshold,
    applied: Vec<String>,
    rosters: Vec<PackageRoster>,
    skipped: Vec<String>,
    unconfigured: Vec<String>,
    exclusions: Vec<(String, usize)>,
    config_file: Option<String>,
    baseline: Option<String>,
    suppressed: usize,
    stale: usize,
    fixed: usize,
}

impl JsonPrinter {
    pub fn new(files_scanned: usize) -> Self {
        Self {
            files_scanned,
            threshold: OffenceThreshold::default(),
            applied: Vec::new(),
            rosters: Vec::new(),
            skipped: Vec::new(),
            unconfigured: Vec::new(),
            exclusions: Vec::new(),
            config_file: None,
            baseline: None,
            suppressed: 0,
            stale: 0,
            fixed: 0,
        }
    }

    // How many files --fix repaired. Stated alongside what is left, because a
    // fixer reporting only its successes would be the same silence this tool
    // refuses everywhere else.
    pub fn with_fixed(self, fixed: usize) -> Self {
        Self { fixed, ..self }
    }

    pub fn with_baseline(self, baseline: Option<String>, suppressed: usize, stale: usize) -> Self {
        Self {
            baseline,
            suppressed,
            stale,
            ..self
        }
    }

    pub fn with_config_file(self, config_file: Option<String>) -> Self {
        Self {
            config_file,
            ..self
        }
    }

    // Each pattern with the number of files it removed, so a consumer can tell
    // a run that looked at everything from one that was told not to -- and can
    // see a pattern sitting at zero, which is a stale exclusion rather than a
    // working one.
    pub fn with_exclusions(self, exclusions: Vec<(String, usize)>) -> Self {
        Self { exclusions, ..self }
    }

    // A consumer that could not tell an all-rules run from a one-rule run would
    // read "no offences" as "nothing wrong", which is only true of the rules
    // that were actually applied.
    // A roster per package walked. The text report collapses these where they
    // agree; a document does not, because nothing is reading it for brevity and
    // a consumer that has to tell absent from empty has been given a puzzle.
    pub fn with_package_rosters(self, rosters: Vec<PackageRoster>) -> Self {
        Self { rosters, ..self }
    }

    pub fn with_rules(
        self,
        applied: Vec<String>,
        skipped: Vec<String>,
        unconfigured: Vec<String>,
    ) -> Self {
        Self {
            applied,
            skipped,
            unconfigured,
            ..self
        }
    }

    pub fn with_threshold(self, threshold: OffenceThreshold) -> Self {
        Self { threshold, ..self }
    }

    // Returns the document rather than printing it, so the shape is assertable
    // in a test instead of being checked for not panicking.
    // offences_found is the true total and offences is what survived the
    // threshold, so a consumer reading only the array can still see that it is
    // not the whole story. rules_broken counts every rule that was broken, not
    // only those whose offences fitted.
    fn excluded_total(&self) -> usize {
        self.exclusions.iter().map(|(_, count)| count).sum()
    }

    pub fn render(&self, offences: &[Offence]) -> String {
        let broken: BTreeSet<&str> = offences.iter().map(|offence| offence.rule).collect();
        let shown = self.threshold.kept(offences);
        let document = json!({
            "baseline": self.baseline,
            "files_fixed": self.fixed,
            "baselined": self.suppressed,
            "baseline_stale_entries": self.stale,
            "config_file": self.config_file,
            "files_scanned": self.files_scanned,
            "files_excluded": self.excluded_total(),
            "exclusions": self.exclusions.iter().map(|(pattern, count)| json!({
                "pattern": pattern,
                "files_excluded": count,
            })).collect::<Vec<_>>(),
            "offences_found": offences.len(),
            "offences_reported": shown.len(),
            "offences_omitted": self.threshold.omitted(offences),
            "offence_threshold": self.threshold.limit(),
            "rules_broken": broken.len(),
            "rules_applied": self.applied,
            "rules_skipped": self.skipped,
            "rules_unconfigured": self.unconfigured,
            "packages": self.package_documents(),
            "offences": shown,
        });
        to_string_pretty(&document).unwrap_or_default()
    }

    fn package_documents(&self) -> Vec<Value> {
        self.rosters
            .iter()
            .map(|roster| {
                json!({
                    "package": roster.package,
                    "rules_applied": roster.applied,
                    "rules_skipped": roster.skipped,
                    "rules_unconfigured": roster
                        .unconfigured
                        .iter()
                        .map(|(rule, requirement)| json!({
                            "rule": rule,
                            "requirement": requirement,
                        }))
                        .collect::<Vec<_>>(),
                })
            })
            .collect()
    }

    pub fn print(&self, offences: &[Offence]) {
        println!("{}", self.render(offences));
    }

    // The same content the text listing carries, as a document. The two must not
    // give different pictures -- see
    // [ADR-MachineReadableReport](../../docs/ADRs/ADR-MachineReadableReport.md).
}
