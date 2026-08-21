// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// `stern4rust.toml`, beside the manifest it configures.
//
// Two absences must not look alike. No file at all is the ordinary case and
// means "use the defaults". A file that exists and cannot be parsed is an
// error, because it was written on purpose and running as though it were absent
// would apply a configuration nobody chose.
//
// Unknown keys are rejected for the same reason an unknown `--rule` name is: a
// misspelled `exclude` that silently did nothing would look exactly like one
// that worked.

use std::env;
use std::fs;
use std::path::PathBuf;
use stern4rust::settings::config_file::ConfigFile;

fn directory(name: &str, contents: Option<&str>) -> PathBuf {
    let path = env::temp_dir().join(format!("stern4rust_cfg_{name}"));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("create the directory");
    if let Some(contents) = contents {
        fs::write(path.join(ConfigFile::NAME), contents).expect("write the config");
    }
    path
}

#[test]
fn baseline_from_resolves_against_the_directory_of_the_config() {
    // Arrange
    let path = directory(
        "baseline",
        Some(
            "baseline = \"stern4rust-baseline.json\"
",
        ),
    );
    let loaded = ConfigFile::load(&path).expect("loads").expect("present");

    // Act
    let baseline = loaded.baseline_from(&path);

    // Assert
    assert_eq!(baseline, Some(path.join("stern4rust-baseline.json")));
}

#[test]
fn header_file_from_resolves_against_the_directory_of_the_config() {
    // Arrange
    let path = directory("header", Some("header-file = \"docs/header.txt\"\n"));
    let loaded = ConfigFile::load(&path).expect("loads").expect("present");

    // Act
    let header = loaded.header_file_from(&path);

    // Assert
    assert_eq!(header, Some(path.join("docs/header.txt")));
}

#[test]
fn load_of_a_config_reads_every_field() {
    // Arrange
    let path = directory(
        "full",
        Some(
            "header-file = \"docs/header.txt\"\n\
             offence-threshold = 25\n\
             rules = [\"header\", \"tests-layout\"]\n\
             skip = [\"test-file-structure\"]\n\
             exclude = [\"fixture/**\"]\n",
        ),
    );

    // Act
    let loaded = ConfigFile::load(&path).expect("loads").expect("present");

    // Assert
    assert_eq!(loaded.offence_threshold, Some(25));
    assert_eq!(loaded.rules, ["header", "tests-layout"]);
    assert_eq!(loaded.skip, ["test-file-structure"]);
    assert_eq!(loaded.exclude, ["fixture/**"]);
}

#[test]
fn load_of_a_directory_without_a_config_is_none() {
    // Arrange
    let path = directory("absent", None);

    // Act
    let loaded = ConfigFile::load(&path).expect("no file is not a failure");

    // Assert
    assert!(loaded.is_none());
}

// A package with nothing to say needs no section, which is the honest rendering
// of "this one applies everything".
#[test]
fn load_of_a_file_with_no_sections_reads_no_packages() {
    // Arrange
    let path = directory("no_sections", Some("header-file = \"docs/header.txt\"\n"));

    // Act
    let file = ConfigFile::load(&path).expect("read").expect("a file");

    // Assert
    assert!(file.packages.is_empty());
}

// baseline is a set of fingerprints for one run and offence-threshold is about
// how the report ends. Neither is a property of a package, so neither is a key
// a section has -- and deny_unknown_fields says so without a check of its own.
#[test]
fn load_of_a_section_naming_a_run_level_key_is_an_error() {
    // Arrange
    let path = directory(
        "section_baseline",
        Some("[package.node]\nbaseline = \"stern4rust-baseline.json\"\n"),
    );

    // Act
    let result = ConfigFile::load(&path);

    // Assert
    assert!(result.is_err());
}

#[test]
fn load_of_a_section_naming_an_unknown_key_is_an_error() {
    // Arrange
    let path = directory(
        "section_typo",
        Some("[package.node]\nskkip = [\"header\"]\n"),
    );

    // Act
    let result = ConfigFile::load(&path);

    // Assert
    assert!(result.is_err());
}

// Every key a section may carry, so that adding one to the struct without
// meaning to is caught here rather than in a repository.
#[test]
fn load_of_a_section_reads_every_key_a_package_may_set() {
    // Arrange
    let path = directory(
        "section_full",
        Some(
            r#"
[package.node]
rules = ["header"]
skip = ["paired-test-file"]
exclude = ["vendor/**"]
header-file = "docs/other.txt"
max-files-per-directory = 30
max-subfolders-per-directory = 8
"#,
        ),
    );

    // Act
    let file = ConfigFile::load(&path).expect("read").expect("a file");

    // Assert
    let node = &file.packages["node"];
    assert_eq!(node.rules, vec!["header"]);
    assert_eq!(node.skip, vec!["paired-test-file"]);
    assert_eq!(node.exclude, vec!["vendor/**"]);
    assert_eq!(node.max_files_per_directory, Some(30));
    assert_eq!(node.max_subfolders_per_directory, Some(8));
    assert!(node.header_file.is_some());
}

// Every key is optional, so a repository can set one thing without restating
// the defaults for everything else.
#[test]
fn load_of_an_empty_config_is_all_defaults() {
    // Arrange
    let path = directory("empty", Some("\n"));

    // Act
    let loaded = ConfigFile::load(&path).expect("loads").expect("present");

    // Assert
    assert_eq!(loaded, ConfigFile::default());
}

// A file written on purpose that cannot be understood must not be treated as
// absent.
#[test]
fn load_of_an_invalid_config_is_an_error() {
    // Arrange
    let path = directory("invalid", Some("offence-threshold = \"not a number\"\n"));

    // Act
    let loaded = ConfigFile::load(&path);

    // Assert
    assert!(loaded.is_err());
}

// The misspelling that would otherwise be silent.
#[test]
fn load_of_an_unknown_key_is_an_error() {
    // Arrange
    let path = directory("unknown", Some("excludes = [\"fixture/**\"]\n"));

    // Act
    let loaded = ConfigFile::load(&path);

    // Assert
    assert!(loaded.is_err());
}

// A workspace whose members want different rules says so in one file, in a
// section each, rather than in a stern4rust.toml beside every manifest.
#[test]
fn load_reads_a_section_for_each_named_package() {
    // Arrange
    let path = directory(
        "sections",
        Some(
            r#"
header-file = "docs/header.txt"

[package.validation]
skip = ["paired-test-file"]

[package.system-tests]
skip = ["paired-test-file"]
exclude = ["**/generated/**"]
"#,
        ),
    );

    // Act
    let file = ConfigFile::load(&path).expect("read").expect("a file");

    // Assert
    assert_eq!(file.packages.len(), 2);
    assert_eq!(file.packages["validation"].skip, vec!["paired-test-file"]);
    assert_eq!(
        file.packages["system-tests"].exclude,
        vec!["**/generated/**"]
    );
}
