// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A baseline as the JSON file a repository checks in.
//
// A missing or unreadable file is an error rather than an empty baseline: a
// gate whose baseline path has a typo would otherwise report every existing
// offence and look like a regression.

use std::env;
use std::fs;
use stern4rust::reporting::offence::Offence;
use stern4rust::settings::adoption::baseline::Baseline;
use stern4rust::source_path::SourcePath;
use stern4rust::traits::baseline_store::BaselineStore;
use stern4rust_cli::system::json_baseline_store::JsonBaselineStore;
use stern4rust_cli::system::portable_path::PortablePath;

fn offence(file: &str, line: usize, rule: &'static str, description: &str) -> Offence {
    Offence::new(file, line, rule, description.to_string(), "fix".to_string())
}

fn temporary(name: &str) -> SourcePath {
    let path = env::temp_dir().join(format!("stern4rust_baseline_{name}.json"));
    let _ = fs::remove_file(&path);
    PortablePath::of(&path).expect("a UTF-8 temporary directory")
}

#[test]
fn load_of_a_file_that_is_not_a_baseline_is_an_error() {
    // Arrange
    let path = temporary("invalid");
    fs::write(path.as_str(), "not json at all").expect("write");

    // Act
    let loaded = JsonBaselineStore::new().load(&path);

    // Assert
    assert!(loaded.is_err());
}

#[test]
fn load_of_a_missing_file_is_an_error() {
    // Arrange
    let path = temporary("absent");

    // Act
    let loaded = JsonBaselineStore::new().load(&path);

    // Assert
    assert!(loaded.is_err());
}

// Written and read back unchanged, because the file is checked in and reviewed.
#[test]
fn save_then_load_round_trips() {
    // Arrange
    let path = temporary("roundtrip");
    let baseline = Baseline::of(&[
        offence("src/a.rs", 1, "header", "wrong"),
        offence("src/a.rs", 2, "header", "wrong"),
    ]);

    // Act
    JsonBaselineStore::new()
        .save(&path, &baseline)
        .expect("saves");

    // Assert
    assert_eq!(
        JsonBaselineStore::new().load(&path).expect("loads"),
        baseline
    );
}
