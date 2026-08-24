// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The one place that knows which rules exist.
//
// The property worth stating is that a rule with nothing to work from is left
// out rather than registered and silently passing. A run that reports "all rules
// satisfied" while a rule was never configured is worse than one that says so,
// because the green is indistinguishable from a real pass.

use stern4rust::reporting::offence::Offence;
use stern4rust::reporting::rule_explanation::RuleExplanation;
use stern4rust::rule::Rule;
use stern4rust::rule_registry::RuleRegistry;
use stern4rust::settings::config::Config;
use stern4rust::settings::rule_selection::RuleSelection;
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
        RuleExplanation::new(self.name(), "a stub", "before", "after")
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
        RuleExplanation::new(self.name(), "a stub", "before", "after")
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
        RuleExplanation::new(self.name(), "a stub", "before", "after")
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
    assert_eq!(offences[0].rule, "always");
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
    assert_eq!(offences[0].rule, "workspace");
    assert_eq!(offences[0].description, "saw 2 files");
}

#[test]
fn explanations_covers_every_registered_rule() {
    // Arrange
    let registry = RuleRegistry::from_config(&fully_configured(&["// header"]));

    // Act
    let explained: Vec<&'static str> = registry
        .explanations()
        .iter()
        .map(|entry| entry.name)
        .collect();

    // Assert
    assert_eq!(explained, registry.names());
}

// The example has to break the rule it illustrates, or it teaches the wrong
// thing. Each is fed back through its own rule and must be reported.
#[test]
fn explanations_offer_examples_that_their_own_rule_reports() {
    // Arrange
    let registry = RuleRegistry::from_config(&fully_configured(&["// header"]));

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

// --rules is only worth having if every rule can answer for itself, so the
// invariant is asserted over the registered set rather than rule by rule. A
// rule added without an example would otherwise print a blank section and read
// as a rule with nothing to say.
#[test]
fn from_config_registers_rules_that_each_offer_an_example_and_a_remedy() {
    // Arrange
    let registry = RuleRegistry::from_config(&fully_configured(&["// header"]));

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

// The header rule cannot hold until it is told what the header says, so it joins
// the set only once there is one to compare against.
#[test]
fn from_config_with_a_header_registers_the_header_rule_alongside_the_others() {
    // Arrange & Act
    let registry = RuleRegistry::from_config(&fully_configured(&["// header"]));

    // Assert
    assert_eq!(
        registry.names(),
        [
            "readable-source",
            "arrange-act-assert",
            "declared-by-name",
            "directory-file-count",
            "directory-subfolder-count",
            "imported-paths",
            "module-registry",
            "ordered-imports",
            "paired-test-file",
            "pure-traits",
            "registry-completeness",
            "single-implemented-type",
            "spdx-matches-manifest",
            "test-file-name-postfix",
            "test-file-structure",
            "test-free-source",
            "test-naming",
            "tested-public-api",
            "tests-layout",
            "workspace-dependencies",
            "header"
        ]
    );
}

// Naming one rule is what lets a codebase facing hundreds of offences gate on
// something today rather than on nothing.
#[test]
fn from_config_with_a_selection_registers_only_the_selected_rules() {
    // Arrange
    let config = Config {
        selection: RuleSelection::new(vec!["tests-layout".to_string()], Vec::new()),
        ..Config::default()
    };

    // Act
    let registry = RuleRegistry::from_config(&config);

    // Assert
    assert_eq!(registry.names(), ["tests-layout"]);
}

#[test]
fn from_config_with_a_skip_leaves_that_rule_out() {
    // Arrange
    let config = Config {
        selection: RuleSelection::new(Vec::new(), vec!["test-file-structure".to_string()]),
        ..Config::default()
    };

    // Act
    let registry = RuleRegistry::from_config(&config);

    // Assert
    assert_eq!(
        registry.names(),
        [
            "readable-source",
            "arrange-act-assert",
            "declared-by-name",
            "directory-file-count",
            "directory-subfolder-count",
            "imported-paths",
            "module-registry",
            "ordered-imports",
            "paired-test-file",
            "pure-traits",
            "registry-completeness",
            "single-implemented-type",
            "test-file-name-postfix",
            "test-free-source",
            "test-naming",
            "tested-public-api",
            "tests-layout",
            "workspace-dependencies"
        ]
    );
}

// The structure rule needs nothing configured, so it holds from the first run.
// A tool that does nothing until it is given a flag is a tool nobody switches on.
#[test]
fn from_config_without_a_header_registers_the_rules_that_need_no_configuration() {
    // Arrange & Act
    let registry = RuleRegistry::from_config(&Config::default());

    // Assert
    assert_eq!(
        registry.names(),
        [
            "readable-source",
            "arrange-act-assert",
            "declared-by-name",
            "directory-file-count",
            "directory-subfolder-count",
            "imported-paths",
            "module-registry",
            "ordered-imports",
            "paired-test-file",
            "pure-traits",
            "registry-completeness",
            "single-implemented-type",
            "test-file-name-postfix",
            "test-file-structure",
            "test-free-source",
            "test-naming",
            "tested-public-api",
            "tests-layout",
            "workspace-dependencies"
        ]
    );
}

// Built by asking each rule its own name, so the list cannot drift from the
// rules themselves -- which is what the switches validate against.
#[test]
fn known_names_lists_every_rule_the_tool_has() {
    // Arrange & Act
    let known = RuleRegistry::known_names();

    // Assert
    assert_eq!(
        known,
        [
            "readable-source",
            "arrange-act-assert",
            "declared-by-name",
            "directory-file-count",
            "directory-subfolder-count",
            "imported-paths",
            "module-registry",
            "ordered-imports",
            "paired-test-file",
            "pure-traits",
            "registry-completeness",
            "single-implemented-type",
            "spdx-matches-manifest",
            "test-file-name-postfix",
            "test-file-structure",
            "test-free-source",
            "test-naming",
            "tested-public-api",
            "tests-layout",
            "workspace-dependencies",
            "header"
        ]
    );
}

// The regression this closes. known_names once kept its own list of rules
// beside the one from_config builds, so a rule added to only one of them was
// applied by a default run while `--rule <name>` rejected it as unknown. Both
// now read the same list, and this fails if they stop doing so.
#[test]
fn known_names_matches_what_a_fully_configured_run_applies() {
    // Arrange
    let config = fully_configured(&["// Copyright"]);

    // Act
    let applied = RuleRegistry::from_config(&config).names();

    // Assert
    assert_eq!(applied, RuleRegistry::known_names());
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

// Not the same as what went unregistered: a header rule left out for want of a
// header file was never deselected by anybody, and saying it was would blame
// the user for an omission they did not make.
#[test]
fn skipped_names_lists_what_the_switches_turned_off() {
    // Arrange
    let selection = RuleSelection::new(Vec::new(), vec!["tests-layout".to_string()]);

    // Act
    let skipped = RuleRegistry::skipped_names(&selection);

    // Assert
    assert_eq!(skipped, ["tests-layout"]);
}

#[test]
fn unconfigured_names_does_not_list_a_skipped_rule() {
    // Arrange
    // A licence, so that the header rule is the only one that could be
    // unconfigured -- and it is the one being skipped.
    let config = Config {
        selection: RuleSelection::new(Vec::new(), vec!["header".to_string()]),
        manifest_license: Some("MIT".to_string()),
        ..Config::default()
    };
    let registry = RuleRegistry::from_config(&config);

    // Act
    let unconfigured = registry.unconfigured_names(&config);

    // Assert
    assert!(unconfigured.is_empty(), "got {unconfigured:?}");
}

// The requirement comes from the rule, so the report can say what to pass
// rather than only that something is unset.
#[test]
fn unconfigured_names_each_rule_with_what_it_is_waiting_for() {
    // Arrange
    let registry = RuleRegistry::from_config(&Config::default());

    // Act
    let unconfigured = registry.unconfigured(&Config::default());

    // Assert
    assert_eq!(
        unconfigured,
        [
            (
                "spdx-matches-manifest",
                "needs a `license` field in Cargo.toml"
            ),
            ("header", "needs --header-file")
        ]
    );
}

// A rule dropped for want of configuration is not a rule anybody skipped, and
// reporting it as skipped would blame the user for a choice they did not make.
// Reporting it as nothing at all is worse: the run silently checks less than it
// appears to.
#[test]
fn unconfigured_names_lists_a_rule_that_could_not_run() {
    // Arrange
    let registry = RuleRegistry::from_config(&Config::default());

    // Act
    let unconfigured = registry.unconfigured_names(&Config::default());

    // Assert
    assert_eq!(unconfigured, ["spdx-matches-manifest", "header"]);
}
