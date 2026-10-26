// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A dev-dependency adds to the dependencies. An entry in `[dev-dependencies]`
// that names a crate `[dependencies]` already gives the tests, written the same
// way and asking for nothing more, is a line that does nothing -- and nothing
// else says so: cargo accepts it, and cargo-shear looks for unused
// dependencies, not repeated ones.

use stern4rust::finding::model::manifest_dependency::ManifestDependency;
use stern4rust::reporting::offence::Offence;
use stern4rust::rule::Rule;
use stern4rust::rules::manifest::distinct_dev_dependencies_rule::DistinctDevDependenciesRule;
use stern4rust::source_file::SourceFile;

const MANIFEST: &str = "core/Cargo.toml";
const RULE: &str = "distinct-dev-dependencies";

fn check(declared: Vec<ManifestDependency>) -> Vec<Offence> {
    DistinctDevDependenciesRule::new(declared).check_workspace(&[SourceFile::new("src/lib.rs", "")])
}

fn dev(name: &str) -> ManifestDependency {
    ManifestDependency::new(MANIFEST, name, "dev-dependencies", true)
}

fn normal(name: &str) -> ManifestDependency {
    ManifestDependency::new(MANIFEST, name, "dependencies", true)
}

// The subject is the manifest, not any source file, so the per-file door stays
// shut.
#[test]
fn check_of_a_single_file_reports_nothing() {
    // Arrange
    let file = SourceFile::new("src/lib.rs", "");

    // Act
    let offences = DistinctDevDependenciesRule::new(vec![normal("clap"), dev("clap")]).check(&file);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_of_a_build_dependency_repeating_a_dependency_reports_nothing() {
    // Arrange & Act
    let offences = check(vec![
        normal("cc"),
        ManifestDependency::new(MANIFEST, "cc", "build-dependencies", true),
    ]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// slotgate's shape: the tests turn on a feature production must not ship, and a
// dev-dependency is the only way cargo gives them one.
#[test]
fn check_workspace_of_a_dev_dependency_adding_a_feature_reports_nothing() {
    // Arrange & Act
    let offences = check(vec![
        normal("tokio"),
        dev("tokio").with_features(vec!["test-util".to_string()]),
    ]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_of_a_dev_dependency_alone_reports_nothing() {
    // Arrange & Act
    let offences = check(vec![normal("anyhow"), dev("tempfile")]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// The dev entry gives the tests the crate on every target the dependency leaves
// out.
#[test]
fn check_workspace_of_a_dev_dependency_beside_a_targeted_dependency_reports_nothing() {
    // Arrange & Act
    let offences = check(vec![
        normal("nix").with_target(Some("cfg(unix)")),
        dev("nix"),
    ]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_of_a_dev_dependency_beside_an_optional_dependency_reports_nothing() {
    // Arrange & Act
    let offences = check(vec![normal("serde").with_optional(true), dev("serde")]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// The shape twin4rust, dry4rust and this repository had: the same workspace
// spec in both tables.
#[test]
fn check_workspace_of_a_dev_dependency_repeating_a_dependency_reports_it() {
    // Arrange & Act
    let offences = check(vec![normal("clap"), dev("clap")]);

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].rule(), RULE);
    assert_eq!(offences[0].file(), MANIFEST);
    assert_eq!(offences[0].line(), 1);
    assert_eq!(offences[0].subject(), Some("clap"));
    assert_eq!(
        offences[0].description(),
        "core/Cargo.toml declares `clap` in [dev-dependencies] as well as in [dependencies], \
         and adds nothing to it"
    );
    assert_eq!(
        offences[0].correction(),
        "remove `clap` from [dev-dependencies]: the tests already have it from [dependencies]"
    );
}

// One taken from the workspace, the other spelled out, is left to
// `workspace-dependencies`, which reports the spelled-out one in a workspace.
#[test]
fn check_workspace_of_a_dev_dependency_written_unlike_its_dependency_reports_nothing() {
    // Arrange & Act
    let offences = check(vec![
        normal("clap"),
        ManifestDependency::new(MANIFEST, "clap", "dev-dependencies", false),
    ]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_of_a_targeted_dev_dependency_repeating_a_dependency_names_its_table() {
    // Arrange & Act
    let offences = check(vec![
        normal("nix"),
        dev("nix").with_target(Some("cfg(unix)")),
    ]);

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(
        offences[0].description(),
        "core/Cargo.toml declares `nix` in [target.'cfg(unix)'.dev-dependencies] as well as in \
         [dependencies], and adds nothing to it"
    );
}

#[test]
fn check_workspace_of_nothing_declared_reports_nothing() {
    // Arrange & Act
    let offences = check(Vec::new());

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// twin4rust's core repeated two, and each is a line of its own to delete.
#[test]
fn check_workspace_of_two_repeats_reports_each() {
    // Arrange & Act
    let offences = check(vec![normal("clap"), normal("syn"), dev("clap"), dev("syn")]);

    // Assert
    let subjects: Vec<Option<&str>> = offences.iter().map(Offence::subject).collect();
    assert_eq!(subjects, [Some("clap"), Some("syn")]);
}

#[test]
fn explanation_shows_the_repeated_entry_removed() {
    // Arrange & Act
    let explanation = DistinctDevDependenciesRule::new(Vec::new()).explanation();

    // Assert
    assert_eq!(explanation.name, RULE);
    assert!(explanation.breaks.contains("[dev-dependencies]"));
    assert!(!explanation.instead.contains("[dev-dependencies]"));
}

#[test]
fn is_configured_for_a_rule_that_needs_nothing_returns_true() {
    // Arrange & Act
    let configured = DistinctDevDependenciesRule::new(Vec::new()).is_configured();

    // Assert
    assert!(configured);
}

#[test]
fn name_is_the_kebab_case_rule_name_used_in_the_report() {
    // Arrange & Act
    let name = DistinctDevDependenciesRule::new(Vec::new()).name();

    // Assert
    assert_eq!(name, RULE);
}

#[test]
fn requirement_of_a_rule_that_needs_nothing_is_none() {
    // Arrange & Act
    let requirement = DistinctDevDependenciesRule::new(Vec::new()).requirement();

    // Assert
    assert!(requirement.is_none());
}
