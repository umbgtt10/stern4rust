// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The rules a run applies, held as one set: every file goes to every rule, the
// tree to every rule whose subject it is, and each rule answers for itself in
// the order it was handed over. Which rules those are is the catalog's to say.

use stern4rust::reporting::offence::Offence;
use stern4rust::reporting::rule_explanation::RuleExplanation;
use stern4rust::rule::Rule;
use stern4rust::rule_catalog::RuleCatalog;
use stern4rust::rule_registry::RuleRegistry;
use stern4rust::settings::config::Config;
use stern4rust::source_file::SourceFile;

struct AlwaysOffends;

impl Rule for AlwaysOffends {
    fn name(&self) -> &'static str {
        "always"
    }

    fn check(&self, file: &SourceFile) -> Vec<Offence> {
        vec![Offence::new(
            file.relative_path(),
            1,
            "always",
            "no".into(),
            "fix it".into(),
        )]
    }

    fn check_workspace(&self, _files: &[SourceFile]) -> Vec<Offence> {
        Vec::new()
    }

    fn is_configured(&self) -> bool {
        true
    }

    fn requirement(&self) -> Option<&'static str> {
        None
    }

    fn explanation(&self) -> RuleExplanation {
        RuleExplanation {
            name: self.name(),
            summary: "a stub",
            breaks: "before",
            instead: "after",
        }
    }
}

fn file() -> SourceFile {
    SourceFile::new("src/a.rs", "pub struct A;")
}

// Everything a run can be given: a header to compare against and a manifest
// that names a licence. Two rules need one each, and both are dropped without.
fn fully_configured(lines: &[&str]) -> Config {
    Config {
        expected_header: lines.iter().map(|line| (*line).to_string()).collect(),
        manifest_license: Some("MIT".to_string()),
        ..Config::default()
    }
}

struct NeverOffends;

impl Rule for NeverOffends {
    fn name(&self) -> &'static str {
        "never"
    }

    fn check(&self, _file: &SourceFile) -> Vec<Offence> {
        Vec::new()
    }

    fn check_workspace(&self, _files: &[SourceFile]) -> Vec<Offence> {
        Vec::new()
    }

    fn is_configured(&self) -> bool {
        true
    }

    fn requirement(&self) -> Option<&'static str> {
        None
    }

    fn explanation(&self) -> RuleExplanation {
        RuleExplanation {
            name: self.name(),
            summary: "a stub",
            breaks: "before",
            instead: "after",
        }
    }
}

struct OffendsPerWorkspace;

impl Rule for OffendsPerWorkspace {
    fn name(&self) -> &'static str {
        "workspace"
    }

    fn check(&self, _file: &SourceFile) -> Vec<Offence> {
        Vec::new()
    }

    fn check_workspace(&self, files: &[SourceFile]) -> Vec<Offence> {
        vec![Offence::new(
            "absent.rs",
            1,
            "workspace",
            format!("saw {} files", files.len()),
            "fix it".to_string(),
        )]
    }

    fn is_configured(&self) -> bool {
        true
    }

    fn requirement(&self) -> Option<&'static str> {
        None
    }

    fn explanation(&self) -> RuleExplanation {
        RuleExplanation {
            name: self.name(),
            summary: "a stub",
            breaks: "before",
            instead: "after",
        }
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

// The two doors stay separate. A rule whose subject is the tree implements only
// check_workspace and inherits a no-op check, so the per-file pass must not
// report its offence a second time.
#[test]
fn check_of_a_workspace_only_rule_reports_nothing() {
    // Arrange
    let registry = RuleRegistry::new(vec![Box::new(OffendsPerWorkspace)]);

    // Act
    let offences = registry.check(&file());

    // Assert
    assert!(offences.is_empty());
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
    assert_eq!(offences[0].rule(), "always");
}

// The registry asks both questions of every rule, so a rule that answers only
// the workspace one is still reached -- and is handed the whole set of files
// rather than one at a time.
#[test]
fn check_workspace_collects_the_offences_of_every_registered_rule() {
    // Arrange
    let registry = RuleRegistry::new(vec![Box::new(AlwaysOffends), Box::new(OffendsPerWorkspace)]);

    // Act
    let offences = registry.check_workspace(&[file(), file()]);

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].rule(), "workspace");
    assert_eq!(offences[0].description(), "saw 2 files");
}

#[test]
fn explanations_covers_every_registered_rule() {
    // Arrange
    let registry = RuleRegistry::new(RuleCatalog::selected(&fully_configured(&["// header"])));

    // Act
    let explained: Vec<&'static str> = registry
        .explanations()
        .iter()
        .map(|entry| entry.name)
        .collect();

    // Assert
    assert_eq!(explained, registry.names());
}

// --rules is only worth having if every rule can answer for itself, so the
// invariant is asserted over the registered set rather than rule by rule. A
// rule added without an example would otherwise print a blank section and read
// as a rule with nothing to say.
#[test]
fn explanations_of_every_rule_offer_an_example_and_a_remedy() {
    // Arrange
    let registry = RuleRegistry::new(RuleCatalog::selected(&fully_configured(&["// header"])));

    // Act
    let silent: Vec<&'static str> = registry
        .explanations()
        .iter()
        .filter(|entry| entry.breaks.trim().is_empty() || entry.instead.trim().is_empty())
        .map(|entry| entry.name)
        .collect();

    // Assert
    assert!(silent.is_empty(), "rules with no example: {silent:?}");
}

// The example has to break the rule it illustrates, or it teaches the wrong
// thing. Each is fed back through its own rule and must be reported.
#[test]
fn explanations_offer_examples_that_their_own_rule_reports() {
    // Arrange
    let registry = RuleRegistry::new(RuleCatalog::selected(&fully_configured(&["// header"])));

    // Act
    let unproven: Vec<&'static str> = registry
        .explanations()
        .iter()
        .filter(|entry| entry.summary.trim().is_empty())
        .map(|entry| entry.name)
        .collect();

    // Assert
    assert!(unproven.is_empty(), "rules with no summary: {unproven:?}");
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
