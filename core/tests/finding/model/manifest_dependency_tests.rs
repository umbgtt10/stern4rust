// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// One dependency as a member manifest declares it, read from the TOML rather
// than from resolved metadata -- the question is how it was written, and
// resolution erases that.

use stern4rust::finding::model::manifest_dependency::ManifestDependency;

const MANIFEST: &str = "core/Cargo.toml";

fn dev(name: &str) -> ManifestDependency {
    ManifestDependency::new(MANIFEST, name, "dev-dependencies", true)
}

fn normal(name: &str) -> ManifestDependency {
    ManifestDependency::new(MANIFEST, name, "dependencies", true)
}

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
    let kept = ManifestDependency::in_manifest(all.as_deref(), "alpha/Cargo.toml");

    // Assert
    let kept = kept.expect("a workspace yields a list");
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].name(), "serde");
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
    let kept = ManifestDependency::in_manifest(all.as_deref(), "beta/Cargo.toml");

    // Assert
    assert_eq!(kept, Some(Vec::new()));
}

// `None` means the package is not in a workspace, which is how the rule is
// told to stand down. Filtering must not turn that into an empty list.
#[test]
fn in_manifest_of_no_workspace_stays_none() {
    // Arrange & Act
    let kept = ManifestDependency::in_manifest(None, "alpha/Cargo.toml");

    // Assert
    assert!(kept.is_none());
}

// Built from its four facts alone, an entry asks for the crate and nothing more:
// default features on, nothing optional, no features, no target, no version.
#[test]
fn new_asks_for_the_crate_and_nothing_more() {
    // Arrange & Act
    let dependency = normal("anyhow");

    // Assert
    assert!(dependency.default_features());
    assert!(!dependency.optional());
    assert!(dependency.features().is_empty());
    assert_eq!(dependency.target(), None);
    assert_eq!(dependency.version(), None);
}

#[test]
fn new_keeps_every_field_it_was_given() {
    // Arrange & Act
    let dependency = ManifestDependency::new("node/Cargo.toml", "anyhow", "dependencies", true);

    // Assert
    assert_eq!(dependency.manifest(), "node/Cargo.toml");
    assert_eq!(dependency.name(), "anyhow");
    assert_eq!(dependency.section(), "dependencies");
    assert!(dependency.takes_from_workspace());
}

// Build scripts are a separate graph: a build dependency is not the tests'.
#[test]
fn repeats_of_a_build_entry_beside_its_dependency_is_false() {
    // Arrange & Act
    let repeats =
        ManifestDependency::new(MANIFEST, "cc", "build-dependencies", true).repeats(&normal("cc"));

    // Assert
    assert!(!repeats);
}

#[test]
fn repeats_of_a_dev_entry_adding_a_feature_is_false() {
    // Arrange & Act
    let repeats = dev("tokio")
        .with_features(vec!["test-util".to_string()])
        .repeats(&normal("tokio"));

    // Assert
    assert!(!repeats);
}

#[test]
fn repeats_of_a_dev_entry_beside_a_dependency_of_another_name_is_false() {
    // Arrange & Act
    let repeats = dev("clap").repeats(&normal("syn"));

    // Assert
    assert!(!repeats);
}

#[test]
fn repeats_of_a_dev_entry_beside_a_dependency_of_another_target_is_false() {
    // Arrange & Act
    let repeats = dev("nix")
        .with_target(Some("cfg(unix)"))
        .repeats(&normal("nix").with_target(Some("cfg(windows)")));

    // Assert
    assert!(!repeats);
}

// The dev entry gives the tests the crate on every target the dependency does
// not cover.
#[test]
fn repeats_of_a_dev_entry_beside_a_targeted_dependency_is_false() {
    // Arrange & Act
    let repeats = dev("nix").repeats(&normal("nix").with_target(Some("cfg(unix)")));

    // Assert
    assert!(!repeats);
}

// Without the feature that turns it on, the tests would not have it.
#[test]
fn repeats_of_a_dev_entry_beside_an_optional_dependency_is_false() {
    // Arrange & Act
    let repeats = dev("serde").repeats(&normal("serde").with_optional(true));

    // Assert
    assert!(!repeats);
}

#[test]
fn repeats_of_a_dev_entry_beside_another_dev_entry_is_false() {
    // Arrange & Act
    let repeats = dev("clap").repeats(&dev("clap"));

    // Assert
    assert!(!repeats);
}

// A dev-dependency repeats a dependency when it names the same crate in the same
// manifest, for a target the dependency covers, written the same way, and asks
// for nothing the dependency does not already give the tests.
#[test]
fn repeats_of_a_dev_entry_beside_its_dependency_is_true() {
    // Arrange & Act
    let repeats = dev("clap").repeats(&normal("clap"));

    // Assert
    assert!(repeats);
}

#[test]
fn repeats_of_a_dev_entry_in_another_manifest_is_false() {
    // Arrange & Act
    let repeats = dev("clap").repeats(&ManifestDependency::new(
        "xtask/Cargo.toml",
        "clap",
        "dependencies",
        true,
    ));

    // Assert
    assert!(!repeats);
}

#[test]
fn repeats_of_a_dev_entry_keeping_default_features_the_dependency_turns_off_is_false() {
    // Arrange & Act
    let repeats = dev("syn").repeats(&normal("syn").with_default_features(false));

    // Assert
    assert!(!repeats);
}

#[test]
fn repeats_of_a_dev_entry_listing_some_of_the_dependency_features_is_true() {
    // Arrange & Act
    let repeats = dev("syn")
        .with_features(vec!["full".to_string()])
        .repeats(&normal("syn").with_features(vec!["full".to_string(), "visit".to_string()]));

    // Assert
    assert!(repeats);
}

#[test]
fn repeats_of_a_dev_entry_of_another_version_requirement_is_false() {
    // Arrange & Act
    let repeats = ManifestDependency::new(MANIFEST, "serde", "dev-dependencies", false)
        .with_version(Some("1.0.200"))
        .repeats(
            &ManifestDependency::new(MANIFEST, "serde", "dependencies", false)
                .with_version(Some("1")),
        );

    // Assert
    assert!(!repeats);
}

#[test]
fn repeats_of_a_dev_entry_of_its_dependency_target_is_true() {
    // Arrange & Act
    let repeats = dev("nix")
        .with_target(Some("cfg(unix)"))
        .repeats(&normal("nix").with_target(Some("cfg(unix)")));

    // Assert
    assert!(repeats);
}

#[test]
fn repeats_of_a_dev_entry_turning_off_default_features_the_dependency_keeps_is_true() {
    // Arrange & Act
    let repeats = dev("syn")
        .with_default_features(false)
        .repeats(&normal("syn"));

    // Assert
    assert!(repeats);
}

// One taken from the workspace, the other spelled out: what the workspace entry
// enables is not in this manifest, so the pair is not compared.
// `workspace-dependencies` reports the spelled-out one in a workspace.
#[test]
fn repeats_of_a_dev_entry_written_unlike_its_dependency_is_false() {
    // Arrange & Act
    let repeats = dev("clap").repeats(&ManifestDependency::new(
        MANIFEST,
        "clap",
        "dependencies",
        false,
    ));

    // Assert
    assert!(!repeats);
}

// The dependency covers every target, the one the dev entry names included.
#[test]
fn repeats_of_a_targeted_dev_entry_beside_an_untargeted_dependency_is_true() {
    // Arrange & Act
    let repeats = dev("nix")
        .with_target(Some("cfg(unix)"))
        .repeats(&normal("nix"));

    // Assert
    assert!(repeats);
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

#[test]
fn table_of_a_targeted_entry_names_its_target() {
    // Arrange & Act
    let table = dev("nix").with_target(Some("cfg(unix)")).table();

    // Assert
    assert_eq!(table, "target.'cfg(unix)'.dev-dependencies");
}

#[test]
fn table_of_an_untargeted_entry_is_its_section() {
    // Arrange & Act
    let table = dev("clap").table();

    // Assert
    assert_eq!(table, "dev-dependencies");
}

#[test]
fn with_default_features_records_them() {
    // Arrange & Act
    let dependency = normal("syn").with_default_features(false);

    // Assert
    assert!(!dependency.default_features());
}

#[test]
fn with_features_records_them() {
    // Arrange & Act
    let dependency = normal("syn").with_features(vec!["full".to_string()]);

    // Assert
    assert_eq!(dependency.features(), ["full"]);
}

#[test]
fn with_optional_records_it() {
    // Arrange & Act
    let dependency = normal("serde").with_optional(true);

    // Assert
    assert!(dependency.optional());
}

#[test]
fn with_target_records_it() {
    // Arrange & Act
    let dependency = normal("nix").with_target(Some("cfg(unix)"));

    // Assert
    assert_eq!(dependency.target(), Some("cfg(unix)"));
}

#[test]
fn with_version_records_it() {
    // Arrange & Act
    let dependency = normal("serde").with_version(Some("1"));

    // Assert
    assert_eq!(dependency.version(), Some("1"));
}
