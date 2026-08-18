// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The report as data rather than as a table.
//
// The table is sized to its contents and read by a person. Nothing can parse it
// reliably: paths and descriptions both contain spaces, and descriptions carry
// backticks, quotes and semicolons, so splitting on whitespace is guesswork.
// This renders the same run as JSON so a gate script or an agent consumes the
// findings without inferring where one column ends and the next begins.
//
// render returns the document instead of printing it, which is what lets these
// tests parse it back and assert on the structure rather than on a substring.

use serde_json::Value;
use stern4rust::json_printer::JsonPrinter;
use stern4rust::offence::Offence;

fn parsed(files_scanned: usize, offences: &[Offence]) -> Value {
    serde_json::from_str(&JsonPrinter::new(files_scanned).render(offences)).expect("valid json")
}

fn plain() -> Offence {
    Offence::new(
        "src/a.rs",
        12,
        "header",
        "expected something".to_string(),
        "fix it".to_string(),
    )
}

// Two offences from one rule are one broken rule, the same count the table's
// summary line reports.
#[test]
fn render_counts_one_broken_rule_for_two_offences_of_the_same_rule() {
    // Arrange & Act
    let found = parsed(1, &[plain(), plain()]);

    // Assert
    assert_eq!(found["offences_found"], 2);
    assert_eq!(found["rules_broken"], 1);
}

// The header rule's descriptions embed the offending text with {:?}, so quotes
// reach the report as a matter of course rather than as an edge case.
#[test]
fn render_escapes_a_description_containing_quotes() {
    // Arrange
    let offence = Offence::new(
        "src/a.rs",
        1,
        "header",
        r#"expected "// Copyright" but found "// nope""#.to_string(),
        r#"make line 1 read "// Copyright""#.to_string(),
    );

    // Act
    let found = parsed(1, &[offence]);

    // Assert
    assert_eq!(
        found["offences"][0]["description"],
        Value::String(r#"expected "// Copyright" but found "// nope""#.to_string())
    );
}

#[test]
fn render_includes_the_summary_counts() {
    // Arrange & Act
    let found = parsed(43, &[plain()]);

    // Assert
    assert_eq!(found["files_scanned"], 43);
    assert_eq!(found["offences_found"], 1);
    assert_eq!(found["rules_broken"], 1);
}

#[test]
fn render_names_every_field_of_an_offence() {
    // Arrange
    let offence = plain()
        .with_subject("the constant `LIMIT`")
        .with_expected("// Copyright");

    // Act
    let found = parsed(1, &[offence]);

    // Assert
    assert_eq!(found["offences"][0]["file"], "src/a.rs");
    assert_eq!(found["offences"][0]["line"], 12);
    assert_eq!(found["offences"][0]["rule"], "header");
    assert_eq!(found["offences"][0]["correction"], "fix it");
    assert_eq!(found["offences"][0]["subject"], "the constant `LIMIT`");
    assert_eq!(found["offences"][0]["expected"], "// Copyright");
}

// The keys are present even when the rule had nothing to put in them, so a
// consumer reads the same shape on every offence rather than testing for the
// existence of a field.
#[test]
fn render_of_an_offence_without_extras_still_carries_both_keys() {
    // Arrange & Act
    let found = parsed(1, &[plain()]);

    // Assert
    assert_eq!(found["offences"][0]["subject"], Value::Null);
    assert_eq!(found["offences"][0]["expected"], Value::Null);
}

#[test]
fn render_of_no_offences_reports_an_empty_list() {
    // Arrange & Act
    let found = parsed(43, &[]);

    // Assert
    assert_eq!(found["offences"], Value::Array(vec![]));
    assert_eq!(found["rules_broken"], 0);
}

#[test]
fn render_of_several_offences_keeps_the_order_it_was_given() {
    // Arrange
    let first = Offence::new(
        "src/a.rs",
        1,
        "header",
        "first".to_string(),
        "fix it".to_string(),
    );
    let second = Offence::new(
        "src/b.rs",
        2,
        "header",
        "second".to_string(),
        "fix it".to_string(),
    );

    // Act
    let found = parsed(2, &[first, second]);

    // Assert
    assert_eq!(found["offences"][0]["description"], "first");
    assert_eq!(found["offences"][1]["description"], "second");
}
