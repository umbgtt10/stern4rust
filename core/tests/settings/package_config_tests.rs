// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
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

use stern4rust::settings::package_config::PackageConfig;
use stern4rust::source_path::SourcePath;

fn path(text: &str) -> SourcePath {
    SourcePath::new(text.to_string())
}

#[test]
fn default_carries_no_header_file() {
    // Arrange & Act
    let config = PackageConfig::default();

    // Assert
    assert!(config.header_file_from(&path("/repo")).is_none());
}

#[test]
fn default_selects_and_skips_nothing() {
    // Arrange & Act
    let config = PackageConfig::default();

    // Assert
    assert!(config.rules().is_empty());
    assert_eq!(config.skip(), None);
    assert!(config.exclude().is_empty());
}

// An absolute path is already an answer, and joining leaves it alone.
#[test]
fn header_file_from_leaves_an_absolute_path_alone() {
    // Arrange
    let absolute = path("/etc").join("header.txt");
    let config = PackageConfig::default().with_header_file(absolute.clone());

    // Act
    let resolved = config.header_file_from(&path("/repo"));

    // Assert
    assert_eq!(resolved, Some(absolute));
}

// Resolved against the config's own directory, which for a root file is the
// workspace root -- so a section writes `docs/header.txt` rather than the
// `../docs/header.txt` a file beside a member manifest needed.
#[test]
fn header_file_from_resolves_against_the_directory_it_is_given() {
    // Arrange
    let config = PackageConfig::default().with_header_file(path("docs/header.txt"));

    // Act
    let resolved = config.header_file_from(&path("/repo"));

    // Assert
    assert_eq!(resolved, Some(path("/repo").join("docs/header.txt")));
}

#[test]
fn with_header_file_names_the_header_and_keeps_the_rest() {
    // Arrange & Act
    let config = PackageConfig::default().with_header_file(path("docs/header.txt"));

    // Assert
    assert_eq!(config.header_file(), Some(&path("docs/header.txt")));
    assert!(config.skip().is_none());
}

#[test]
fn with_skip_stands_the_rules_down_and_keeps_the_rest() {
    // Arrange & Act
    let config = PackageConfig::default().with_skip(vec![String::from("paired-test-file")]);

    // Assert
    assert_eq!(config.skip(), Some(&[String::from("paired-test-file")][..]));
    assert!(config.header_file().is_none());
}
