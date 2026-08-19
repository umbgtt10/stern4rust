// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The offences a repository has already agreed to live with.
//
// --rule lets a codebase enforce one rule at a time. What it cannot express is
// "every rule, against new code only", which is what a codebase with six
// hundred existing offences needs -- otherwise the choice is between a gate
// that fails forever and no gate at all.
//
// Counts, not a set: fixing one of two identical offences and introducing
// another must still pass, while introducing a third must not.

use std::env;
use std::fs;
use std::path::PathBuf;
use stern4rust::adoption::baseline::Baseline;
use stern4rust::reporting::offence::Offence;

fn offence(file: &str, line: usize, rule: &'static str, description: &str) -> Offence {
    Offence::new(file, line, rule, description.to_string(), "fix".to_string())
}

fn temporary(name: &str) -> PathBuf {
    let path = env::temp_dir().join(format!("stern4rust_baseline_{name}.json"));
    let _ = fs::remove_file(&path);
    path
}

#[test]
fn apply_forgives_an_offence_that_moved_to_a_new_line() {
    // Arrange
    let baseline = Baseline::of(&[offence("src/a.rs", 12, "header", "wrong")]);

    // Act
    let outcome = baseline.apply(vec![offence("src/a.rs", 400, "header", "wrong")]);

    // Assert
    assert!(outcome.kept.is_empty());
    assert_eq!(outcome.suppressed, 1);
}

#[test]
fn apply_forgives_each_recorded_offence_once() {
    // Arrange
    let baseline = Baseline::of(&[
        offence("src/a.rs", 1, "header", "wrong"),
        offence("src/a.rs", 2, "header", "wrong"),
    ]);

    // Act
    let outcome = baseline.apply(vec![
        offence("src/a.rs", 1, "header", "wrong"),
        offence("src/a.rs", 2, "header", "wrong"),
        offence("src/a.rs", 3, "header", "wrong"),
    ]);

    // Assert
    assert_eq!(outcome.kept.len(), 1);
    assert_eq!(outcome.suppressed, 2);
}

#[test]
fn apply_of_an_empty_baseline_keeps_everything() {
    // Arrange
    let baseline = Baseline::default();

    // Act
    let outcome = baseline.apply(vec![offence("src/a.rs", 1, "header", "wrong")]);

    // Assert
    assert_eq!(outcome.kept.len(), 1);
    assert_eq!(outcome.suppressed, 0);
}

#[test]
fn apply_reports_an_offence_the_baseline_never_saw() {
    // Arrange
    let baseline = Baseline::of(&[offence("src/a.rs", 1, "header", "wrong")]);

    // Act
    let outcome = baseline.apply(vec![offence("src/b.rs", 1, "header", "wrong")]);

    // Assert
    assert_eq!(outcome.kept.len(), 1);
    assert_eq!(outcome.kept[0].file, "src/b.rs");
    assert_eq!(outcome.suppressed, 0);
}

// An entry matching nothing describes an offence somebody has since fixed. Dead
// weight that makes the file look like it is still holding something back.
#[test]
fn apply_reports_how_many_entries_matched_nothing() {
    // Arrange
    let baseline = Baseline::of(&[
        offence("src/a.rs", 1, "header", "wrong"),
        offence("src/gone.rs", 1, "header", "wrong"),
    ]);

    // Act
    let outcome = baseline.apply(vec![offence("src/a.rs", 1, "header", "wrong")]);

    // Assert
    assert_eq!(outcome.stale, 1);
    assert!(outcome.is_stale());
}

#[test]
fn len_counts_every_recorded_occurrence() {
    // Arrange & Act
    let baseline = Baseline::of(&[
        offence("src/a.rs", 1, "header", "wrong"),
        offence("src/a.rs", 2, "header", "wrong"),
        offence("src/b.rs", 1, "header", "wrong"),
    ]);

    // Assert
    assert_eq!(baseline.len(), 3);
    assert!(!baseline.is_empty());
}

#[test]
fn load_of_a_file_that_is_not_a_baseline_is_an_error() {
    // Arrange
    let path = temporary("invalid");
    fs::write(&path, "not json at all").expect("write");

    // Act
    let loaded = Baseline::load(&path);

    // Assert
    assert!(loaded.is_err());
}

#[test]
fn load_of_a_missing_file_is_an_error() {
    // Arrange
    let path = temporary("absent");

    // Act
    let loaded = Baseline::load(&path);

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
    baseline.save(&path).expect("saves");

    // Assert
    assert_eq!(Baseline::load(&path).expect("loads"), baseline);
}
