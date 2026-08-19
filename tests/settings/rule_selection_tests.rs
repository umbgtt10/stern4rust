// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Which rules a run applies.
//
// The default is everything, because a tool that does nothing until it is
// configured is a tool nobody switches on. The switches exist for adoption: a
// repository facing two hundred offences cannot gate on all five rules today,
// but it can gate on one of them today and the rest as it goes.
//
// A misspelled name is an error rather than a rule that quietly matches
// nothing. `--skip test-file-strucutre` that silently skipped nothing would
// look exactly like a run that worked.

use stern4rust::settings::rule_selection::RuleSelection;

const KNOWN: [&str; 3] = ["header", "test-file-structure", "tests-layout"];

fn selection(selected: &[&str], skipped: &[&str]) -> RuleSelection {
    RuleSelection::new(
        selected.iter().map(|name| (*name).to_string()).collect(),
        skipped.iter().map(|name| (*name).to_string()).collect(),
    )
}

#[test]
fn default_includes_every_rule() {
    // Arrange & Act
    let selection = RuleSelection::default();

    // Assert
    assert!(selection.includes("header"));
    assert!(selection.includes("tests-layout"));
}

#[test]
fn includes_a_rule_that_was_selected() {
    // Arrange & Act
    let selection = selection(&["header"], &[]);

    // Assert
    assert!(selection.includes("header"));
}

// Skipping wins. Asking for a rule and excluding it in the same breath is a
// contradiction, and the safer reading of a contradiction is the narrower one.
#[test]
fn includes_of_a_rule_both_selected_and_skipped_is_false() {
    // Arrange & Act
    let selection = selection(&["header"], &["header"]);

    // Assert
    assert!(!selection.includes("header"));
}

// An explicit selection is a whitelist: naming one rule excludes the rest.
#[test]
fn includes_of_a_rule_outside_an_explicit_selection_is_false() {
    // Arrange & Act
    let selection = selection(&["header"], &[]);

    // Assert
    assert!(!selection.includes("tests-layout"));
}

#[test]
fn includes_of_a_skipped_rule_is_false() {
    // Arrange & Act
    let selection = selection(&[], &["tests-layout"]);

    // Assert
    assert!(!selection.includes("tests-layout"));
    assert!(selection.includes("header"));
}

// Distinct from includes(): the header rule needs a header file, and asking for
// it without one has to be an error rather than an empty run.
#[test]
fn selects_explicitly_of_a_named_rule_is_true() {
    // Arrange & Act
    let selection = selection(&["header"], &[]);

    // Assert
    assert!(selection.selects_explicitly("header"));
}

#[test]
fn selects_explicitly_with_no_selection_is_false() {
    // Arrange & Act
    let selection = RuleSelection::default();

    // Assert
    assert!(!selection.selects_explicitly("header"));
}

#[test]
fn unknown_in_checks_both_switches() {
    // Arrange & Act
    let selection = selection(&["heder"], &["tets-layout"]);

    // Assert
    assert_eq!(selection.unknown_in(&KNOWN), ["heder", "tets-layout"]);
}

#[test]
fn unknown_in_names_a_misspelled_rule() {
    // Arrange & Act
    let selection = selection(&[], &["test-file-strucutre"]);

    // Assert
    assert_eq!(selection.unknown_in(&KNOWN), ["test-file-strucutre"]);
}

#[test]
fn unknown_in_of_valid_names_is_empty() {
    // Arrange & Act
    let selection = selection(&["header"], &["tests-layout"]);

    // Assert
    assert!(selection.unknown_in(&KNOWN).is_empty());
}
