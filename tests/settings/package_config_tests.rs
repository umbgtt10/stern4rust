// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// What one package in a workspace says about itself.
//
// The keys it does *not* carry are the load-bearing part: `baseline` and
// `offence-threshold` are about a run rather than a package, and leaving them
// off the struct is what makes `deny_unknown_fields` reject them. Those cases
// live in `config_file_tests`, where a section can be parsed from real TOML.
//
// Here the question is the one thing this type does on its own: resolve a header
// path against the directory the config file sits in.

use std::path::Path;
use std::path::PathBuf;
use stern4rust::settings::package_config::PackageConfig;

#[test]
fn default_carries_no_header_file() {
    // Arrange & Act
    let config = PackageConfig::default();

    // Assert
    assert!(config.header_file_from(Path::new("/repo")).is_none());
}

#[test]
fn default_selects_and_skips_nothing() {
    // Arrange & Act
    let config = PackageConfig::default();

    // Assert
    assert!(config.rules.is_empty());
    assert!(config.skip.is_empty());
    assert!(config.exclude.is_empty());
}

// An absolute path is already an answer, and joining leaves it alone.
#[test]
fn header_file_from_leaves_an_absolute_path_alone() {
    // Arrange
    let absolute = Path::new("/etc").join("header.txt");
    let config = PackageConfig {
        header_file: Some(absolute.clone()),
        ..PackageConfig::default()
    };

    // Act
    let resolved = config.header_file_from(Path::new("/repo"));

    // Assert
    assert_eq!(resolved, Some(absolute));
}

// Resolved against the config's own directory, which for a root file is the
// workspace root -- so a section writes `docs/header.txt` rather than the
// `../docs/header.txt` a file beside a member manifest needed.
#[test]
fn header_file_from_resolves_against_the_directory_it_is_given() {
    // Arrange
    let config = PackageConfig {
        header_file: Some(PathBuf::from("docs/header.txt")),
        ..PackageConfig::default()
    };

    // Act
    let resolved = config.header_file_from(Path::new("/repo"));

    // Assert
    assert_eq!(resolved, Some(Path::new("/repo").join("docs/header.txt")));
}
