// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Which rules a run applies.
//
// The default is everything, because a tool that does nothing until it is
// configured is a tool nobody switches on. The switches exist for adoption: a
// repository facing two hundred offences cannot gate on all five rules today,
// but it can gate on one of them today and the rest as it goes.
//
// Naming a rule with --rule makes the selection a whitelist; --skip subtracts
// from whatever is left. That is the shape clippy, ruff and eslint converge on,
// so nobody has to learn this one.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RuleSelection {
    selected: Vec<String>,
    skipped: Vec<String>,
}

impl RuleSelection {
    pub fn new(selected: Vec<String>, skipped: Vec<String>) -> Self {
        Self { selected, skipped }
    }

    // The same selection, standing down on these as well.
    //
    // Used to fold every package section's skips into the run-level answer, so
    // the report cannot claim a rule applied when one package stood it down.
    pub fn also_skipping(&self, more: &[String]) -> Self {
        let mut skipped = self.skipped.clone();
        skipped.extend(more.iter().cloned());
        skipped.sort();
        skipped.dedup();
        Self {
            selected: self.selected.clone(),
            skipped,
        }
    }

    // Skipping wins over selecting. Asking for a rule and excluding it in the
    // same breath is a contradiction, and the safer reading of a contradiction
    // is the narrower one.
    pub fn includes(&self, name: &str) -> bool {
        if self.skipped.iter().any(|skipped| skipped == name) {
            return false;
        }
        self.selected.is_empty() || self.selected.iter().any(|selected| selected == name)
    }

    // Distinct from includes(), because "this rule was asked for by name" and
    // "this rule is in the set" differ for a rule that cannot run without
    // configuration. Asking for the header rule without a header file is an
    // error; not asking for it is an omission.
    pub fn selects_explicitly(&self, name: &str) -> bool {
        self.selected.iter().any(|selected| selected == name)
    }

    // A misspelled name is an error rather than a rule that quietly matches
    // nothing. `--skip test-file-strucutre` that silently skipped nothing would
    // look exactly like a run that worked.
    pub fn unknown_in(&self, known: &[&str]) -> Vec<String> {
        self.selected
            .iter()
            .chain(self.skipped.iter())
            .filter(|name| !known.contains(&name.as_str()))
            .cloned()
            .collect()
    }
}
