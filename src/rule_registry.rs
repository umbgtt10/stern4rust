// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::config::Config;
use crate::offence::Offence;
use crate::rule::Rule;
use crate::rules::header_rule::HeaderRule;
use crate::rules::readable_source_rule::ReadableSourceRule;
use crate::rules::test_file_structure_rule::TestFileStructureRule;
use crate::rules::tests_layout_rule::TestsLayoutRule;
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
        // The structure rule needs nothing configured, so it holds from the
        // first run. The header rule cannot: it has no idea what your header
        // says until you tell it.
        // readable-source comes first because it is the one rule whose failure
        // explains every other rule's silence on the same file.
        let mut rules: Vec<Box<dyn Rule>> = vec![
            Box::new(ReadableSourceRule::new()),
            Box::new(TestFileStructureRule::new()),
            Box::new(TestsLayoutRule::new()),
        ];
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

    // Asked once, after every file has been read, for the rules whose subject is
    // the tree rather than a file in it.
    pub fn check_workspace(&self, files: &[SourceFile]) -> Vec<Offence> {
        self.rules
            .iter()
            .flat_map(|rule| rule.check_workspace(files))
            .collect()
    }
}
