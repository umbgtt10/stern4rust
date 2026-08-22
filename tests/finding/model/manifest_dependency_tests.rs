// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// One dependency as a member manifest declares it, read from the TOML rather
// than from resolved metadata -- the question is how it was written, and
// resolution erases that.

use stern4rust::finding::model::manifest_dependency::ManifestDependency;

// A finding belongs to the manifest that states it, and every package is
// handed only its own. Without this, `check_workspace` running once per
// package stated each finding once per member.
#[test]
fn in_manifest_keeps_only_the_declarations_of_that_manifest() {
    // Arrange
    let all = Some(vec![
        ManifestDependency::new("alpha/Cargo.toml", "serde", "dependencies", false),
        ManifestDependency::new("beta/Cargo.toml", "anyhow", "dependencies", false),
    ]);

    // Act
    let kept = ManifestDependency::in_manifest(&all, "alpha/Cargo.toml");

    // Assert
    let kept = kept.expect("a workspace yields a list");
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].name, "serde");
}

#[test]
fn in_manifest_of_a_manifest_declaring_nothing_is_empty() {
    // Arrange
    let all = Some(vec![ManifestDependency::new(
        "alpha/Cargo.toml",
        "serde",
        "dependencies",
        false,
    )]);

    // Act
    let kept = ManifestDependency::in_manifest(&all, "beta/Cargo.toml");

    // Assert
    assert_eq!(kept, Some(Vec::new()));
}

// `None` means the package is not in a workspace, which is how the rule is
// told to stand down. Filtering must not turn that into an empty list.
#[test]
fn in_manifest_of_no_workspace_stays_none() {
    // Arrange & Act
    let kept = ManifestDependency::in_manifest(&None, "alpha/Cargo.toml");

    // Assert
    assert!(kept.is_none());
}

#[test]
fn new_keeps_every_field_it_was_given() {
    // Arrange & Act
    let dependency = ManifestDependency::new("node/Cargo.toml", "anyhow", "dependencies", true);

    // Assert
    assert_eq!(dependency.manifest, "node/Cargo.toml");
    assert_eq!(dependency.name, "anyhow");
    assert_eq!(dependency.section, "dependencies");
    assert!(dependency.takes_from_workspace);
}

// The three tables a manifest can declare a dependency in. Dev and build
// dependencies are dependencies: a member pinning its own `proptest` version
// splits the workspace exactly as a runtime one does.
#[test]
fn sections_covers_every_table_a_dependency_can_sit_in() {
    // Arrange & Act
    let sections = ManifestDependency::SECTIONS;

    // Assert
    assert_eq!(
        sections,
        ["dependencies", "dev-dependencies", "build-dependencies"]
    );
}
