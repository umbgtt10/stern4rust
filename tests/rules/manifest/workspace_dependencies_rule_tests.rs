// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A workspace declares its dependencies once, in the root, and every member
// takes them from there.
//
// Only one direction needs checking. `cargo` already refuses to build a
// `{ workspace = true }` the root does not declare, so requiring the root to
// hold every reference costs no code -- the same split R009, R014 and R016 made.

use stern4rust::finding::model::manifest_dependency::ManifestDependency;
use stern4rust::reporting::offence::Offence;
use stern4rust::rule::Rule;
use stern4rust::rules::manifest::workspace_dependencies_rule::WorkspaceDependenciesRule;
use stern4rust::source_file::SourceFile;

const RULE: &str = "workspace-dependencies";

fn check(declared: Option<Vec<ManifestDependency>>) -> Vec<Offence> {
    WorkspaceDependenciesRule::new(declared).check_workspace(&[SourceFile::new("src/lib.rs", "")])
}

fn dependency(manifest: &str, name: &str, takes: bool) -> ManifestDependency {
    ManifestDependency::new(manifest, name, "dependencies", takes)
}

// The subject is the manifest, not any source file, so the per-file door stays
// shut.
#[test]
fn check_of_a_single_file_reports_nothing() {
    // Arrange
    let file = SourceFile::new("src/lib.rs", "");

    // Act
    let offences =
        WorkspaceDependenciesRule::new(Some(vec![dependency("a/Cargo.toml", "serde", false)]))
            .check(&file);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// The etheram-ibft shape: an intra-workspace path dependency spelled out in
// four members while the root already centralises two others.
#[test]
fn check_workspace_of_a_dependency_declared_in_a_member_reports_it() {
    // Arrange & Act
    let offences = check(Some(vec![dependency(
        "validation/Cargo.toml",
        "node",
        false,
    )]));

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].rule, RULE);
    assert_eq!(offences[0].file, "validation/Cargo.toml");
    assert_eq!(offences[0].subject.as_deref(), Some("node"));
    assert_eq!(
        offences[0].correction,
        "add `node` to [workspace.dependencies] in the root manifest, and write \
         `node = { workspace = true }` here"
    );
}

#[test]
fn check_workspace_of_a_member_taking_from_the_workspace_reports_nothing() {
    // Arrange & Act
    let offences = check(Some(vec![dependency("node/Cargo.toml", "anyhow", true)]));

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// A single-crate repository has no root to centralise into, so the rule has
// nothing to say rather than something wrong to report.
#[test]
fn check_workspace_of_a_package_that_is_not_a_workspace_reports_nothing() {
    // Arrange & Act
    let offences = check(None);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// A dev-dependency pinned in a member splits the workspace exactly as a runtime
// one does.
#[test]
fn check_workspace_of_a_pinned_dev_dependency_reports_it() {
    // Arrange & Act
    let offences = check(Some(vec![ManifestDependency::new(
        "node/Cargo.toml",
        "proptest",
        "dev-dependencies",
        false,
    )]));

    // Assert
    assert_eq!(offences.len(), 1);
    assert!(
        offences[0].description.contains("dev-dependencies"),
        "the table is named, got {}",
        offences[0].description
    );
}

#[test]
fn check_workspace_reports_every_member_declaration() {
    // Arrange & Act
    let offences = check(Some(vec![
        dependency("a/Cargo.toml", "serde", false),
        dependency("b/Cargo.toml", "anyhow", true),
        dependency("b/Cargo.toml", "toml", false),
    ]));

    // Assert
    assert_eq!(offences.len(), 2);
    assert_eq!(offences[0].subject.as_deref(), Some("serde"));
    assert_eq!(offences[1].subject.as_deref(), Some("toml"));
}

#[test]
fn is_configured_for_a_rule_that_needs_nothing_returns_true() {
    // Arrange & Act
    let configured = WorkspaceDependenciesRule::new(None).is_configured();

    // Assert
    assert!(configured);
}

#[test]
fn name_is_the_kebab_case_rule_name_used_in_the_report() {
    // Arrange & Act
    let name = WorkspaceDependenciesRule::new(None).name();

    // Assert
    assert_eq!(name, RULE);
}
