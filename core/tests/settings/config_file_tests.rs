// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// `stern4rust.toml` as the core holds it, once read. Reading and parsing the
// file is the command line's `TomlConfigSource`, tested there.

use std::collections::BTreeMap;
use stern4rust::settings::config_file::ConfigFile;
use stern4rust::settings::package_config::PackageConfig;
use stern4rust::source_path::SourcePath;

fn file() -> ConfigFile {
    ConfigFile::new(
        Some(path("stern4rust-baseline.json")),
        Some(25),
        PackageConfig::new(
            Some(path("docs/header.txt")),
            Some(30),
            Some(8),
            vec![String::from("header")],
            Some(vec![String::from("test-file-structure")]),
            vec![String::from("fixture/**")],
        ),
        BTreeMap::from([(String::from("node"), PackageConfig::default())]),
    )
}

fn path(text: &str) -> SourcePath {
    SourcePath::new(text.to_string())
}

#[test]
fn baseline_from_resolves_against_the_directory_it_is_given() {
    // Arrange
    let file = file();

    // Act
    let baseline = file.baseline_from(&path("/repo"));

    // Assert
    assert_eq!(baseline, Some(path("/repo/stern4rust-baseline.json")));
}

#[test]
fn baseline_is_the_path_the_file_names() {
    // Arrange
    let file = file();

    // Act
    let baseline = file.baseline();

    // Assert
    assert_eq!(baseline, Some(&path("stern4rust-baseline.json")));
}

#[test]
fn header_file_from_resolves_against_the_directory_it_is_given() {
    // Arrange
    let file = file();

    // Act
    let header = file.header_file_from(&path("/repo"));

    // Assert
    assert_eq!(header, Some(path("/repo/docs/header.txt")));
}

#[test]
fn into_packages_gives_up_every_section() {
    // Arrange
    let file = file();

    // Act
    let sections = file.into_packages();

    // Assert
    assert_eq!(sections.keys().collect::<Vec<_>>(), ["node"]);
}

#[test]
fn new_of_a_root_stating_no_skip_skips_nothing() {
    // Arrange & Act
    let file = ConfigFile::new(None, None, PackageConfig::default(), BTreeMap::new());

    // Assert
    assert!(file.skip().is_empty());
    assert_eq!(file, ConfigFile::default());
}

// The keys a package section may also set are the root's own, so they come in
// as one; the root's `skip` is its whole list.
#[test]
fn new_takes_the_root_keys_from_the_root_section() {
    // Arrange & Act
    let file = file();

    // Assert
    assert_eq!(file.header_file(), Some(&path("docs/header.txt")));
    assert_eq!(file.max_files_per_directory(), Some(30));
    assert_eq!(file.max_subfolders_per_directory(), Some(8));
    assert_eq!(file.offence_threshold(), Some(25));
    assert_eq!(file.rules(), ["header"]);
    assert_eq!(file.skip(), ["test-file-structure"]);
    assert_eq!(file.exclude(), ["fixture/**"]);
    assert_eq!(file.packages().len(), 1);
}
