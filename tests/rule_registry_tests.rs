// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The one place that knows which rules exist.
//
// The property worth stating is that a rule with nothing to work from is left
// out rather than registered and silently passing. A run that reports "all rules
// satisfied" while a rule was never configured is worse than one that says so,
// because the green is indistinguishable from a real pass.

use stern4rust::config::Config;
use stern4rust::offence::Offence;
use stern4rust::rule::Rule;
use stern4rust::rule_registry::RuleRegistry;
use stern4rust::source_file::SourceFile;

struct AlwaysOffends;

impl Rule for AlwaysOffends {
    fn name(&self) -> &'static str {
        "always"
    }

    fn check(&self, file: &SourceFile) -> Vec<Offence> {
        vec![Offence::new(file.relative_path(), 1, "always", "no".into())]
    }
}

fn config_with_header(lines: &[&str]) -> Config {
    Config {
        expected_header: lines.iter().map(|line| (*line).to_string()).collect(),
        ..Config::default()
    }
}

fn file() -> SourceFile {
    SourceFile::new("src/a.rs", "pub struct A;")
}

struct NeverOffends;

impl Rule for NeverOffends {
    fn name(&self) -> &'static str {
        "never"
    }

    fn check(&self, _file: &SourceFile) -> Vec<Offence> {
        Vec::new()
    }
}

#[test]
fn check_collects_the_offences_of_every_registered_rule() {
    // Arrange
    let registry = RuleRegistry::new(vec![Box::new(AlwaysOffends), Box::new(AlwaysOffends)]);

    // Act
    let offences = registry.check(&file());

    // Assert
    assert_eq!(offences.len(), 2);
}

#[test]
fn check_reports_nothing_when_no_rule_objects() {
    // Arrange
    let registry = RuleRegistry::new(vec![Box::new(NeverOffends)]);

    // Act
    let offences = registry.check(&file());

    // Assert
    assert!(offences.is_empty());
}

// A satisfied rule beside a broken one must not mask it.
#[test]
fn check_reports_the_broken_rule_when_another_is_satisfied() {
    // Arrange
    let registry = RuleRegistry::new(vec![Box::new(NeverOffends), Box::new(AlwaysOffends)]);

    // Act
    let offences = registry.check(&file());

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].rule, "always");
}

// The header rule cannot hold until it is told what the header says, so it joins
// the set only once there is one to compare against.
#[test]
fn from_config_with_a_header_registers_the_header_rule_alongside_the_others() {
    // Arrange & Act
    let registry = RuleRegistry::from_config(&config_with_header(&["// header"]));

    // Assert
    assert_eq!(registry.names(), ["test-file-structure", "header"]);
}

// The structure rule needs nothing configured, so it holds from the first run.
// A tool that does nothing until it is given a flag is a tool nobody switches on.
#[test]
fn from_config_without_a_header_registers_the_rules_that_need_no_configuration() {
    // Arrange & Act
    let registry = RuleRegistry::from_config(&Config::default());

    // Assert
    assert_eq!(registry.names(), ["test-file-structure"]);
}

#[test]
fn names_lists_every_registered_rule() {
    // Arrange
    let registry = RuleRegistry::new(vec![Box::new(AlwaysOffends), Box::new(NeverOffends)]);

    // Act
    let names = registry.names();

    // Assert
    assert_eq!(names, ["always", "never"]);
}
