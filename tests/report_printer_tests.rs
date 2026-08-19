// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Rendering the report a person reads.
//
// render returns the document instead of writing it, which is what lets these
// assert on what comes out rather than only that nothing panicked. The print_
// tests below still pin that the writing path survives the shapes it will meet:
// no offences, one, many, and a path far wider than its column heading.

use stern4rust::offence::Offence;
use stern4rust::offence_threshold::OffenceThreshold;
use stern4rust::report_printer::ReportPrinter;

fn many(count: usize) -> Vec<Offence> {
    (1..=count)
        .map(|line| offence("src/a.rs", line, "header"))
        .collect()
}

fn offence(file: &str, line: usize, rule: &'static str) -> Offence {
    Offence::new(
        file,
        line,
        rule,
        "something is wrong".to_string(),
        "do the thing that makes it right".to_string(),
    )
}

// Columns are sized to their contents, so a path longer than the heading must
// widen the column rather than overflow it.
#[test]
fn print_with_a_path_far_wider_than_its_heading_does_not_panic() {
    // Arrange
    let printer = ReportPrinter::new(1);
    let long = "crates/deeply/nested/package/src/module/submodule/subject.rs";

    // Act & Assert
    printer.print(&[offence(long, 1234, "header")]);
}

#[test]
fn print_with_no_offences_does_not_panic() {
    // Arrange
    let printer = ReportPrinter::new(0);

    // Act & Assert
    printer.print(&[]);
}

#[test]
fn print_with_offences_from_several_rules_does_not_panic() {
    // Arrange
    let printer = ReportPrinter::new(3);

    // Act & Assert
    printer.print(&[
        offence("src/a.rs", 1, "header"),
        offence("tests/b_tests.rs", 42, "test-file-structure"),
        offence("src/c.rs", 7, "header"),
    ]);
}

#[test]
fn print_with_one_offence_does_not_panic() {
    // Arrange
    let printer = ReportPrinter::new(1);

    // Act & Assert
    printer.print(&[offence("src/a.rs", 1, "header")]);
}

// A run with rules switched off must never read as a run that checked
// everything. This is the same refusal as the omitted-offence note, one level
// up: what was not looked at is part of the finding.
#[test]
fn render_names_the_rules_that_were_not_applied() {
    // Arrange & Act
    let report = ReportPrinter::new(1)
        .with_rules(
            vec!["header".to_string()],
            vec!["tests-layout".to_string(), "test-free-source".to_string()],
        )
        .render(&[offence("src/a.rs", 1, "header")]);

    // Assert
    assert!(
        report.contains("2 rule(s) were not applied"),
        "got {report}"
    );
    assert!(
        report.contains("tests-layout, test-free-source"),
        "got {report}"
    );
    assert!(
        report.contains("rules_applied=1 rules_skipped=2"),
        "got {report}"
    );
}

// Loudly, and naming the flag that raises it. A cap nobody was told about
// reads as "that was all of them".
#[test]
fn render_of_more_offences_than_the_threshold_says_how_many_are_not_shown() {
    // Arrange & Act
    let report = ReportPrinter::new(1)
        .with_threshold(OffenceThreshold::new(3))
        .render(&many(10));

    // Assert
    assert!(report.contains("7 more"), "got {report}");
    assert!(report.contains("--offence-threshold"), "got {report}");
}

#[test]
fn render_of_more_offences_than_the_threshold_shows_only_the_threshold() {
    // Arrange & Act
    let report = ReportPrinter::new(1)
        .with_threshold(OffenceThreshold::new(3))
        .render(&many(10));

    // Assert
    assert_eq!(report.matches("something is wrong").count(), 3);
}

#[test]
fn render_of_no_offences_says_every_rule_is_satisfied() {
    // Arrange & Act
    let report = ReportPrinter::new(9).render(&[]);

    // Assert
    assert!(report.contains("All rules are satisfied."));
    assert!(report.contains("files_scanned=9 offences=0 rules_broken=0"));
}

// "All rules are satisfied" is only true when all of them ran.
#[test]
fn render_of_no_offences_with_a_skipped_rule_says_selected_rules_are_satisfied() {
    // Arrange & Act
    let report = ReportPrinter::new(9)
        .with_rules(vec!["header".to_string()], vec!["tests-layout".to_string()])
        .render(&[]);

    // Assert
    assert!(
        report.contains("All selected rules are satisfied."),
        "got {report}"
    );
    assert!(!report.contains("All rules are satisfied."), "got {report}");
    assert!(report.contains("not applied: tests-layout"), "got {report}");
}

// Every offence carries what to do about it, so every row is followed by one.
#[test]
fn render_puts_a_correction_under_every_offence() {
    // Arrange
    let offences = [
        offence("src/a.rs", 1, "header"),
        offence("src/b.rs", 2, "header"),
    ];

    // Act
    let report = ReportPrinter::new(2).render(&offences);

    // Assert
    assert_eq!(
        report
            .matches("fix: do the thing that makes it right")
            .count(),
        2
    );
}

// Beneath the row it belongs to, indented past the columns, so the table stays
// scannable and the correction is unambiguously attached to one offence.
#[test]
fn render_puts_the_correction_on_its_own_line_after_the_offence() {
    // Arrange & Act
    let report = ReportPrinter::new(1).render(&[offence("src/a.rs", 1, "header")]);
    let lines: Vec<&str> = report.lines().collect();

    // Assert
    let row = lines
        .iter()
        .position(|line| line.contains("something is wrong"))
        .expect("the offence row");
    assert!(lines[row + 1].trim_start().starts_with("fix: "));
    assert!(
        lines[row + 1].starts_with("    "),
        "got {:?}",
        lines[row + 1]
    );
}

#[test]
fn render_shows_the_file_line_and_rule_of_every_offence() {
    // Arrange & Act
    let report = ReportPrinter::new(2).render(&[
        offence("src/a.rs", 1, "header"),
        offence("tests/b_tests.rs", 42, "test-file-structure"),
    ]);

    // Assert
    assert!(report.contains("src/a.rs"));
    assert!(report.contains("tests/b_tests.rs"));
    assert!(report.contains("42"));
    assert!(report.contains("test-file-structure"));
}

#[test]
fn render_summarises_two_offences_of_one_rule_as_one_broken_rule() {
    // Arrange & Act
    let report = ReportPrinter::new(5).render(&[
        offence("src/a.rs", 1, "header"),
        offence("src/b.rs", 2, "header"),
    ]);

    // Assert
    assert!(report.contains("files_scanned=5 offences=2 rules_broken=1"));
}

// The cap is on what is shown, never on what is counted. A summary that said
// 3 when the tree holds 10 would be this tool doing the exact thing it exists
// to catch.
#[test]
fn render_summary_counts_every_offence_even_when_some_are_not_shown() {
    // Arrange & Act
    let report = ReportPrinter::new(1)
        .with_threshold(OffenceThreshold::new(3))
        .render(&many(10));

    // Assert
    assert!(
        report.contains("files_scanned=1 offences=10 rules_broken=1"),
        "got {report}"
    );
}
