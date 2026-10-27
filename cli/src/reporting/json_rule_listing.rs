// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use serde_json::Value;
use serde_json::json;
use serde_json::to_string_pretty;
use stern4rust::reporting::rule_explanation::RuleExplanation;

// The rule set as a document: the same four fields the text listing shows,
// named rather than laid out, so an agent asking what a rule wants gets it
// without parsing indentation. It reads the same `RuleExplanation`s the text
// does, so the two cannot give different pictures -- see
// ADR-MachineReadableReport.
pub struct JsonRuleListing<'a> {
    explanations: &'a [RuleExplanation],
}

impl<'a> JsonRuleListing<'a> {
    pub fn new(explanations: &'a [RuleExplanation]) -> Self {
        Self { explanations }
    }

    pub fn render(&self) -> String {
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
}
