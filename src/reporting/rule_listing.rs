// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::reporting::output_format::OutputFormat;
use crate::reporting::rule_explanation::RuleExplanation;
use serde_json::Value;
use serde_json::json;
use serde_json::to_string_pretty;

// The rule set as a document, in whichever form was asked for.
//
// This is not a report and it does not live on either printer, though it began
// there. A printer holds what one run found -- files scanned, offences kept,
// which rules were applied -- and renders that. A listing has no run behind it:
// nothing was scanned, nothing was counted, and every field a printer carries
// would be empty. The give-away was that both rendering functions took no
// `self` and read no field; they were free functions wearing a printer's name.
//
// So the listing is its own subject, and it takes the format rather than the
// caller choosing a printer by hand. That is also what keeps the two forms
// honest with each other -- one type renders both, from one list, which is
// ADR-MachineReadableReport's requirement that the two must not give different
// pictures.
pub struct RuleListing<'a> {
    explanations: &'a [RuleExplanation],
}

impl<'a> RuleListing<'a> {
    pub fn new(explanations: &'a [RuleExplanation]) -> Self {
        Self { explanations }
    }

    pub fn render(&self, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => self.json(),
            OutputFormat::Text => self.text(),
        }
    }

    // One section per rule, in registry order, so the listing reads in the same
    // order as the roster a report prints.
    fn text(&self) -> String {
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

    fn json(&self) -> String {
        let entries: Vec<Value> = self
            .explanations
            .iter()
            .map(|entry| {
                json!({
                    "name": entry.name,
                    "summary": entry.summary,
                    "breaks": entry.breaks,
                    "instead": entry.instead,
                })
            })
            .collect();
        to_string_pretty(&json!({ "rules": entries }))
            .unwrap_or_else(|_| String::from("{\"rules\":[]}"))
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
