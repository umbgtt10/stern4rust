// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
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
use serde_json::from_str;
use stern4rust::reporting::json_printer::JsonPrinter;
use stern4rust::reporting::offence::Offence;
use stern4rust::reporting::offence_threshold::OffenceThreshold;
use stern4rust::reporting::package_roster::PackageRoster;

fn parsed(files_scanned: usize, offences: &[Offence]) -> Value {
    from_str(&JsonPrinter::new(files_scanned).render(offences)).expect("valid json")
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

// The same run as a document, which is what ADR-MachineReadableReport asks of
// this printer. The text report gained a roster per package and this did not,
// so for one afternoon a gate script reading JSON got a different picture from a
// developer reading the terminal -- the conservative aggregate, with no way to
// see which member stood a rule down.
#[test]
fn render_carries_a_roster_for_each_package() {
    // Arrange
    let printer = JsonPrinter::new(9).with_package_rosters(vec![
        PackageRoster::new("node", vec!["header".to_string()], Vec::new(), Vec::new()),
        PackageRoster::new(
            "validation",
            Vec::new(),
            vec!["header".to_string()],
            Vec::new(),
        ),
    ]);

    // Act
    let document: Value = from_str(&printer.render(&[])).expect("valid json");

    // Assert
    let packages = document["packages"].as_array().expect("a packages array");
    assert_eq!(packages.len(), 2);
    assert_eq!(packages[0]["package"], "node");
    assert_eq!(packages[0]["rules_applied"][0], "header");
    assert_eq!(packages[1]["rules_skipped"][0], "header");
}

// Two offences from one rule are one broken rule, the same count the table's
// summary line reports.
// A consumer that could not see the exclusions would read "no offences" as
// "nothing wrong" about a tree the run was told not to look at. The zero-count
// pattern is carried too, since that is a stale exclusion rather than a
// working one.
#[test]
fn render_carries_every_exclusion_with_its_count() {
    // Arrange
    let printer = JsonPrinter::new(4).with_exclusions(vec![
        ("fixture/**".to_string(), 27),
        ("gone/**".to_string(), 0),
    ]);

    // Act
    let document: Value = from_str(&printer.render(&[])).expect("valid json");

    // Assert
    assert_eq!(document["files_excluded"], 27);
    assert_eq!(document["exclusions"][0]["pattern"], "fixture/**");
    assert_eq!(document["exclusions"][0]["files_excluded"], 27);
    assert_eq!(document["exclusions"][1]["pattern"], "gone/**");
    assert_eq!(document["exclusions"][1]["files_excluded"], 0);
}

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

// A consumer that could not tell an all-rules run from a one-rule run would
// read "no offences" as "nothing wrong", which is only true of the rules that
// were actually applied.
#[test]
fn render_lists_the_rules_applied_skipped_and_unconfigured() {
    // Arrange & Act
    let found = from_str::<Value>(
        &JsonPrinter::new(1)
            .with_rules(
                vec!["header".to_string()],
                vec!["tests-layout".to_string()],
                vec!["readable-source".to_string()],
            )
            .render(&[]),
    )
    .expect("valid json");

    // Assert
    assert_eq!(found["rules_applied"][0], "header");
    assert_eq!(found["rules_skipped"][0], "tests-layout");
    assert_eq!(found["rules_unconfigured"][0], "readable-source");
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

// Why a rule is missing travels into the document too, or a machine reading it
// cannot tell a rule nobody asked for from one that could not run.
#[test]
fn render_names_why_a_package_rule_is_unconfigured() {
    // Arrange
    let printer = JsonPrinter::new(9).with_package_rosters(vec![PackageRoster::new(
        "node",
        Vec::new(),
        Vec::new(),
        vec![(
            "spdx-matches-manifest".to_string(),
            "needs a `license` field in Cargo.toml".to_string(),
        )],
    )]);

    // Act
    let document: Value = from_str(&printer.render(&[])).expect("valid json");

    // Assert
    let unconfigured = &document["packages"][0]["rules_unconfigured"][0];
    assert_eq!(unconfigured["rule"], "spdx-matches-manifest");
    assert_eq!(
        unconfigured["requirement"],
        "needs a `license` field in Cargo.toml"
    );
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

// offences_found stays the true total; the array is what was truncated. A
// consumer that read only the array would otherwise believe it had them all.
#[test]
fn render_of_more_offences_than_the_threshold_reports_both_counts() {
    // Arrange
    let offences: Vec<Offence> = (1..=10).map(|_| plain()).collect();

    // Act
    let found = from_str::<Value>(
        &JsonPrinter::new(1)
            .with_threshold(OffenceThreshold::new(3))
            .render(&offences),
    )
    .expect("valid json");

    // Assert
    assert_eq!(found["offences_found"], 10);
    assert_eq!(found["offences_reported"], 3);
    assert_eq!(found["offences_omitted"], 7);
    assert_eq!(found["offence_threshold"], 3);
    assert_eq!(found["offences"].as_array().expect("array").len(), 3);
}

#[test]
fn render_of_no_offences_reports_an_empty_list() {
    // Arrange & Act
    let found = parsed(43, &[]);

    // Assert
    assert_eq!(found["offences"], Value::Array(vec![]));
    assert_eq!(found["rules_broken"], 0);
}

// A single-package run still carries its one roster, because a machine has no
// terminal to collapse things for -- the array is the shape either way.
#[test]
fn render_of_one_package_carries_one_roster() {
    // Arrange
    let printer = JsonPrinter::new(9).with_package_rosters(vec![PackageRoster::new(
        "cargo-stern4rust",
        vec!["header".to_string()],
        Vec::new(),
        Vec::new(),
    )]);

    // Act
    let document: Value = from_str(&printer.render(&[])).expect("valid json");

    // Assert
    assert_eq!(document["packages"].as_array().expect("array").len(), 1);
    assert_eq!(document["packages"][0]["package"], "cargo-stern4rust");
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

// The run-level keys every gate script in this family already reads are
// untouched, so adding the detail does not cost the readers who do not want it.
#[test]
fn render_with_rosters_keeps_the_run_level_keys() {
    // Arrange
    let printer = JsonPrinter::new(9)
        .with_rules(vec!["header".to_string()], Vec::new(), Vec::new())
        .with_package_rosters(vec![PackageRoster::new(
            "node",
            vec!["header".to_string()],
            Vec::new(),
            Vec::new(),
        )]);

    // Act
    let document: Value = from_str(&printer.render(&[])).expect("valid json");

    // Assert
    assert_eq!(document["rules_applied"][0], "header");
    assert_eq!(document["files_scanned"], 9);
}

// A run with no rosters at all -- nothing walked -- still renders a document
// rather than omitting the key, so a reader never has to tell absent from empty.
#[test]
fn render_without_rosters_carries_an_empty_packages_array() {
    // Arrange
    let printer = JsonPrinter::new(0);

    // Act
    let document: Value = from_str(&printer.render(&[])).expect("valid json");

    // Assert
    assert_eq!(document["packages"].as_array().expect("array").len(), 0);
}

#[test]
fn with_baseline_carries_the_suppressed_and_stale_counts() {
    // Arrange
    let printer = JsonPrinter::new(1).with_baseline(Some("bl.json".to_string()), 7, 2);

    // Act
    let document: Value = from_str(&printer.render(&[])).expect("valid json");

    // Assert
    assert_eq!(document["baseline"], "bl.json");
    assert_eq!(document["baselined"], 7);
    assert_eq!(document["baseline_stale_entries"], 2);
}

#[test]
fn with_config_file_carries_the_config_the_run_used() {
    // Arrange
    let printer = JsonPrinter::new(1).with_config_file(Some("stern4rust.toml".to_string()));

    // Act
    let document: Value = from_str(&printer.render(&[])).expect("valid json");

    // Assert
    assert_eq!(document["config_file"], "stern4rust.toml");
}

#[test]
fn with_fixed_carries_the_number_of_files_rewritten() {
    // Arrange
    let printer = JsonPrinter::new(1).with_fixed(12);

    // Act
    let document: Value = from_str(&printer.render(&[])).expect("valid json");

    // Assert
    assert_eq!(document["files_fixed"], 12);
}
