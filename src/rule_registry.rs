// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::config::Config;
use crate::offence::Offence;
use crate::rule::Rule;
use crate::rules::header_rule::HeaderRule;
use crate::source_file::SourceFile;

// The one place that knows which rules exist. Adding a rule is a line here and a
// file under rules/, and nothing else in the tool changes.
//
// A rule with nothing to work from is left out rather than registered and
// silently passing: a run that reports "all rules satisfied" while a rule was
// never configured is worse than one that says which rules it actually applied.
pub struct RuleRegistry {
    rules: Vec<Box<dyn Rule>>,
}

impl RuleRegistry {
    pub fn from_config(config: &Config) -> Self {
        let mut rules: Vec<Box<dyn Rule>> = Vec::new();
        if !config.expected_header.is_empty() {
            rules.push(Box::new(HeaderRule::new(config.expected_header.clone())));
        }
        Self { rules }
    }

    pub fn new(rules: Vec<Box<dyn Rule>>) -> Self {
        Self { rules }
    }

    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    pub fn names(&self) -> Vec<&'static str> {
        self.rules.iter().map(|rule| rule.name()).collect()
    }

    pub fn check(&self, file: &SourceFile) -> Vec<Offence> {
        self.rules
            .iter()
            .flat_map(|rule| rule.check(file))
            .collect()
    }
}
