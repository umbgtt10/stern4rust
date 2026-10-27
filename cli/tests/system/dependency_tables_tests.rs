// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Every dependency a manifest declares, as it is written: the three plain tables
// and each of them under `[target.<cfg>]`, with what an entry asks of the crate
// it names -- the workspace, a version, features, default features, optional.
//
// Read from the TOML rather than from `cargo metadata`, because resolution
// erases exactly what the manifest rules ask about. These run on strings, with
// no manifest on disk.

use stern4rust::finding::model::manifest_dependency::ManifestDependency;
use stern4rust_cli::system::dependency_tables::DependencyTables;
use toml::Table;

const MANIFEST: &str = "core/Cargo.toml";

fn declared(manifest: &str) -> Vec<ManifestDependency> {
    DependencyTables::declared(&manifest.parse::<Table>().expect("valid TOML"), MANIFEST)
}

fn only(manifest: &str) -> ManifestDependency {
    let declared = declared(manifest);
    assert_eq!(declared.len(), 1, "{declared:?}");
    declared[0].clone()
}

#[test]
fn declared_of_a_dotted_workspace_key_takes_from_the_workspace() {
    // Arrange & Act
    let dependency = only("[dependencies]\nclap.workspace = true\n");

    // Assert
    assert!(dependency.takes_from_workspace());
}

#[test]
fn declared_of_a_manifest_without_dependency_tables_is_empty() {
    // Arrange & Act
    let declared = declared("[package]\nname = \"probe\"\n");

    // Assert
    assert!(declared.is_empty(), "{declared:?}");
}

// Unset, an entry keeps its default features, is not optional and turns on
// nothing; and a target-free entry has no target.
#[test]
fn declared_of_a_plain_entry_asks_for_nothing_beyond_the_crate() {
    // Arrange & Act
    let dependency = only("[dependencies]\nanyhow = { version = \"1\" }\n");

    // Assert
    assert!(dependency.default_features());
    assert!(!dependency.optional());
    assert!(dependency.features().is_empty());
    assert_eq!(dependency.target(), None);
}

// Cargo forbids naming one package under two keys, so the key is what a repeat
// is matched on, and it is what is kept.
#[test]
fn declared_of_a_renamed_entry_keeps_the_key_it_is_given() {
    // Arrange & Act
    let dependency = only("[dependencies]\nfoo = { package = \"bar\", version = \"1\" }\n");

    // Assert
    assert_eq!(dependency.name(), "foo");
}

#[test]
fn declared_of_a_target_table_carries_its_target() {
    // Arrange & Act
    let dependency = only("[target.'cfg(unix)'.dev-dependencies]\nnix = \"0.29\"\n");

    // Assert
    assert_eq!(dependency.section(), "dev-dependencies");
    assert_eq!(dependency.target(), Some("cfg(unix)"));
}

#[test]
fn declared_of_a_version_string_carries_it_and_takes_nothing_from_the_workspace() {
    // Arrange & Act
    let dependency = only("[dependencies]\nserde = \"1.0\"\n");

    // Assert
    assert_eq!(dependency.version(), Some("1.0"));
    assert!(!dependency.takes_from_workspace());
}

#[test]
fn declared_of_an_entry_spelling_default_features_with_an_underscore_reads_it() {
    // Arrange & Act
    let dependency = only("[dependencies]\nsyn = { version = \"2\", default_features = false }\n");

    // Assert
    assert!(!dependency.default_features());
}

#[test]
fn declared_of_an_entry_taking_from_the_workspace_says_so() {
    // Arrange & Act
    let dependency = only("[dependencies]\nclap = { workspace = true }\n");

    // Assert
    assert!(dependency.takes_from_workspace());
    assert_eq!(dependency.version(), None);
}

#[test]
fn declared_of_an_entry_turning_off_default_features_reads_it() {
    // Arrange & Act
    let dependency = only("[dependencies]\nsyn = { version = \"2\", default-features = false }\n");

    // Assert
    assert!(!dependency.default_features());
}

#[test]
fn declared_of_an_entry_with_features_carries_them() {
    // Arrange & Act
    let dependency = only(
        "[dev-dependencies]\ntokio = { workspace = true, features = [\"test-util\", \"macros\"] }\n",
    );

    // Assert
    assert_eq!(dependency.features(), ["test-util", "macros"]);
}

#[test]
fn declared_of_an_optional_entry_carries_it() {
    // Arrange & Act
    let dependency = only("[dependencies]\nserde = { version = \"1\", optional = true }\n");

    // Assert
    assert!(dependency.optional());
}

// A manifest's plain tables and its target tables are both read, the plain
// ones first.
#[test]
fn declared_of_both_a_plain_and_a_target_table_reads_both() {
    // Arrange & Act
    let declared = declared(
        "[dependencies]\nanyhow = \"1\"\n\n[target.'cfg(windows)'.dependencies]\nwinapi = \"0.3\"\n",
    );

    // Assert
    let found: Vec<(&str, Option<&str>)> = declared
        .iter()
        .map(|dependency| (dependency.name(), dependency.target()))
        .collect();
    assert_eq!(found, [("anyhow", None), ("winapi", Some("cfg(windows)"))]);
}

// Every table names its entries with the section they sit in and the manifest
// they were read from.
#[test]
fn declared_of_each_plain_table_names_its_entries_with_their_section() {
    // Arrange & Act
    let declared = declared(
        "[dependencies]\nanyhow = \"1\"\n\n[dev-dependencies]\ntempfile = \"3\"\n\n[build-dependencies]\ncc = \"1\"\n",
    );

    // Assert
    let found: Vec<(&str, &str, &str)> = declared
        .iter()
        .map(|dependency| {
            (
                dependency.manifest(),
                dependency.section(),
                dependency.name(),
            )
        })
        .collect();
    assert_eq!(
        found,
        [
            (MANIFEST, "dependencies", "anyhow"),
            (MANIFEST, "dev-dependencies", "tempfile"),
            (MANIFEST, "build-dependencies", "cc"),
        ]
    );
}
