// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::collections::BTreeSet;

use serde_json::json;

use crate::offence::Offence;
use crate::offence_threshold::OffenceThreshold;

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
}

impl JsonPrinter {
    pub fn new(files_scanned: usize) -> Self {
        Self {
            files_scanned,
            threshold: OffenceThreshold::default(),
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
    pub fn render(&self, offences: &[Offence]) -> String {
        let broken: BTreeSet<&str> = offences.iter().map(|offence| offence.rule).collect();
        let shown = self.threshold.kept(offences);
        let document = json!({
            "files_scanned": self.files_scanned,
            "offences_found": offences.len(),
            "offences_reported": shown.len(),
            "offences_omitted": self.threshold.omitted(offences),
            "offence_threshold": self.threshold.limit(),
            "rules_broken": broken.len(),
            "offences": shown,
        });
        serde_json::to_string_pretty(&document).unwrap_or_default()
    }

    pub fn print(&self, offences: &[Offence]) {
        println!("{}", self.render(offences));
    }
}
