// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The settings one run is checked against, built over its Default a field at a time: a record of sixteen is past what
// one constructor may take, so each field has a step of its own and every other keeps its default.

use std::path::Path;
use std::path::PathBuf;
use stern4rust::finding::model::name_readers::NameReaders;
use stern4rust::reporting::offence_threshold::OffenceThreshold;
use stern4rust::reporting::output_format::OutputFormat;
use stern4rust::settings::config::Config;
use stern4rust::settings::rule_selection::RuleSelection;

#[test]
fn with_baseline_sets_it_and_keeps_the_rest() {
    // Arrange & Act
    let config = Config::default().with_baseline(Some(PathBuf::from("stern4rust-baseline.json")));

    // Assert
    assert_eq!(
        config.baseline(),
        Some(Path::new("stern4rust-baseline.json"))
    );
    assert!(!config.fix());
}

#[test]
fn with_config_file_sets_it_and_keeps_the_rest() {
    // Arrange & Act
    let config = Config::default().with_config_file(Some(PathBuf::from("stern4rust.toml")));

    // Assert
    assert_eq!(config.config_file(), Some(Path::new("stern4rust.toml")));
    assert!(config.baseline().is_none());
}

#[test]
fn with_excludes_sets_it_and_keeps_the_rest() {
    // Arrange & Act
    let config = Config::default().with_excludes(vec![String::from("fixture/**")]);

    // Assert
    assert_eq!(config.excludes(), [String::from("fixture/**")]);
    assert!(config.packages().is_empty());
}

#[test]
fn with_expected_header_sets_it_and_keeps_the_rest() {
    // Arrange & Act
    let config = Config::default().with_expected_header(vec![String::from("// Copyright")]);

    // Assert
    assert_eq!(config.expected_header(), [String::from("// Copyright")]);
    assert!(config.excludes().is_empty());
}

#[test]
fn with_fix_sets_it_and_keeps_the_rest() {
    // Arrange & Act
    let config = Config::default().with_fix(true);

    // Assert
    assert!(config.fix());
    assert!(!config.write_baseline());
}

#[test]
fn with_format_sets_it_and_keeps_the_rest() {
    // Arrange & Act
    let config = Config::default().with_format(OutputFormat::Json);

    // Assert
    assert_eq!(config.format(), OutputFormat::Json);
    assert!(!config.fix());
}

#[test]
fn with_manifest_license_sets_it_and_keeps_the_rest() {
    // Arrange & Act
    let config = Config::default().with_manifest_license(Some(String::from("MIT")));

    // Assert
    assert_eq!(config.manifest_license(), Some("MIT"));
    assert!(config.manifest_path().is_none());
}

#[test]
fn with_manifest_path_sets_it_and_keeps_the_rest() {
    // Arrange & Act
    let config = Config::default().with_manifest_path(Some(PathBuf::from("Cargo.toml")));

    // Assert
    assert_eq!(config.manifest_path(), Some(Path::new("Cargo.toml")));
    assert!(config.manifest_license().is_none());
}

#[test]
fn with_max_files_per_directory_sets_it_and_keeps_the_rest() {
    // Arrange & Act
    let config = Config::default().with_max_files_per_directory(Some(20));

    // Assert
    assert_eq!(config.max_files_per_directory(), Some(20));
    assert_eq!(config.max_subfolders_per_directory(), None);
}

#[test]
fn with_max_subfolders_per_directory_sets_it_and_keeps_the_rest() {
    // Arrange & Act
    let config = Config::default().with_max_subfolders_per_directory(Some(5));

    // Assert
    assert_eq!(config.max_subfolders_per_directory(), Some(5));
    assert_eq!(config.max_files_per_directory(), None);
}

#[test]
fn with_name_readers_sets_it_and_keeps_the_rest() {
    // Arrange & Act
    let config = Config::default().with_name_readers(NameReaders::default());

    // Assert
    assert_eq!(config.name_readers(), &NameReaders::default());
    assert!(config.packages().is_empty());
}

#[test]
fn with_offence_threshold_sets_it_and_keeps_the_rest() {
    // Arrange & Act
    let config = Config::default().with_offence_threshold(OffenceThreshold::new(7));

    // Assert
    assert_eq!(config.offence_threshold(), OffenceThreshold::new(7));
    assert!(!config.fix());
}

#[test]
fn with_packages_sets_it_and_keeps_the_rest() {
    // Arrange & Act
    let config = Config::default().with_packages(vec![String::from("node")]);

    // Assert
    assert_eq!(config.packages(), [String::from("node")]);
    assert!(config.excludes().is_empty());
}

#[test]
fn with_selection_sets_it_and_keeps_the_rest() {
    // Arrange & Act
    let config = Config::default()
        .with_selection(RuleSelection::new(vec![String::from("header")], Vec::new()));

    // Assert
    assert_eq!(
        config.selection(),
        &RuleSelection::new(vec![String::from("header")], Vec::new())
    );
    assert!(config.packages().is_empty());
}

#[test]
fn with_workspace_dependencies_sets_it_and_keeps_the_rest() {
    // Arrange & Act
    let config = Config::default().with_workspace_dependencies(Some(Vec::new()));

    // Assert
    assert_eq!(config.workspace_dependencies(), Some(&[][..]));
    assert!(config.manifest_path().is_none());
}

#[test]
fn with_write_baseline_sets_it_and_keeps_the_rest() {
    // Arrange & Act
    let config = Config::default().with_write_baseline(true);

    // Assert
    assert!(config.write_baseline());
    assert!(!config.fix());
}
