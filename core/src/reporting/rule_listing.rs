// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::format;
use alloc::string::String;

use crate::reporting::rule_explanation::RuleExplanation;

// The rule set as the text a person reads.
//
// This is not a report and it does not live on a printer, though it began
// there. A printer holds what one run found -- files scanned, offences kept,
// which rules were applied -- and renders that. A listing has no run behind it:
// nothing was scanned, nothing was counted, and every field a printer carries
// would be empty.
//
// The same list as a document is the command line's `JsonRuleListing`; both
// read the rules' own `RuleExplanation`s, so the two forms cannot give
// different pictures, which is ADR-MachineReadableReport's requirement.
pub struct RuleListing<'a> {
    explanations: &'a [RuleExplanation],
}

impl<'a> RuleListing<'a> {
    pub fn new(explanations: &'a [RuleExplanation]) -> Self {
        Self { explanations }
    }

    // One section per rule, in registry order, so the listing reads in the same
    // order as the roster a report prints.
    pub fn render(&self) -> String {
        let mut out = String::from(
            "stern4rust rules
",
        );
        for entry in self.explanations {
            out.push_str(&format!(
                "
{}
  {}
",
                entry.name, entry.summary
            ));
            out.push_str(&Self::block("breaks", entry.breaks));
            out.push_str(&Self::block("instead", entry.instead));
        }
        out
    }

    // Indented so a multi-line example stays one block rather than running into
    // the next label.
    fn block(label: &str, body: &str) -> String {
        let mut out = format!(
            "
  {label}:
"
        );
        for line in body.lines() {
            out.push_str(&format!(
                "      {line}
"
            ));
        }
        out
    }
}
