// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// One dependency as a member manifest declares it, read from the TOML rather
// than from resolved metadata -- the question is how it was written, and
// resolution erases that.

use stern4rust::finding::manifest_dependency::ManifestDependency;

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
