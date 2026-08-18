// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::collections::BTreeSet;

use serde_json::json;

use crate::offence::Offence;

// The same run as data rather than as a table.
//
// The table is sized to its contents and meant for a person. Nothing can parse
// it reliably -- paths and descriptions both contain spaces, and descriptions
// carry backticks, quotes and semicolons, so splitting on whitespace is
// guesswork. A gate script or an agent reads this instead and never has to
// infer where one column ends and the next begins.
pub struct JsonPrinter {
    files_scanned: usize,
}

impl JsonPrinter {
    pub fn new(files_scanned: usize) -> Self {
        Self { files_scanned }
    }

    // Returns the document rather than printing it, so the shape is assertable
    // in a test instead of being checked for not panicking.
    pub fn render(&self, offences: &[Offence]) -> String {
        let broken: BTreeSet<&str> = offences.iter().map(|offence| offence.rule).collect();
        let document = json!({
            "files_scanned": self.files_scanned,
            "offences_found": offences.len(),
            "rules_broken": broken.len(),
            "offences": offences,
        });
        serde_json::to_string_pretty(&document).unwrap_or_default()
    }

    pub fn print(&self, offences: &[Offence]) {
        println!("{}", self.render(offences));
    }
}
