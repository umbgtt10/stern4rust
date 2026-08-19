// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::collections::BTreeSet;

use serde_json::json;

use crate::offence::Offence;
use crate::offence_threshold::OffenceThreshold;
use serde_json::to_string_pretty;

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
    skipped: Vec<String>,
    unconfigured: Vec<String>,
    exclusions: Vec<(String, usize)>,
}

impl JsonPrinter {
    pub fn new(files_scanned: usize) -> Self {
        Self {
            files_scanned,
            threshold: OffenceThreshold::default(),
            applied: Vec::new(),
            skipped: Vec::new(),
            unconfigured: Vec::new(),
            exclusions: Vec::new(),
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
            "offences": shown,
        });
        to_string_pretty(&document).unwrap_or_default()
    }

    pub fn print(&self, offences: &[Offence]) {
        println!("{}", self.render(offences));
    }
}
