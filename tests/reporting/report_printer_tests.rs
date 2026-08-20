// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Rendering the report a person reads.
//
// render returns the document instead of writing it, which is what lets these
// assert on what comes out rather than only that nothing panicked. The print_
// tests below still pin that the writing path survives the shapes it will meet:
// no offences, one, many, and a path far wider than its column heading.

use stern4rust::reporting::offence::Offence;
use stern4rust::reporting::offence_threshold::OffenceThreshold;
use stern4rust::reporting::report_printer::ReportPrinter;

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

// The case a bare total would hide: a pattern naming a tree that has moved or
// been deleted goes on looking like it is doing work.
#[test]
fn render_names_a_pattern_that_matched_nothing_as_such() {
    // Arrange
    let printer = ReportPrinter::new(4)
        .with_exclusions(vec![("gone/**".to_string(), 0), ("live/**".to_string(), 3)]);

    // Act
    let report = printer.render(&[]);

    // Assert
    assert!(report.contains("matched nothing: gone/**"), "{report}");
    assert!(!report.contains("matched nothing: live/**"), "{report}");
}

// A count answers "how many", which is only useful to a reader who already
// knows how many there are. The names answer what was actually checked.
// An exclusion is only acceptable if the reader can see it. A tree removed
// from the report with no number beside it is the silent skip the walker had
// until 0.4.0.
#[test]
fn render_names_each_exclusion_with_the_files_it_removed() {
    // Arrange
    let printer = ReportPrinter::new(4).with_exclusions(vec![("fixture/**".to_string(), 27)]);

    // Act
    let report = printer.render(&[]);

    // Assert
    assert!(
        report.contains("excluded: fixture/** (27 files)"),
        "{report}"
    );
    assert!(report.contains("files_excluded=27"), "{report}");
}

#[test]
fn render_names_the_rules_that_were_applied() {
    // Arrange & Act
    let report = ReportPrinter::new(1)
        .with_rules(
            vec!["header".to_string(), "tests-layout".to_string()],
            Vec::new(),
            Vec::new(),
        )
        .render(&[]);

    // Assert
    assert!(
        report.contains("applied: header, tests-layout"),
        "got {report}"
    );
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
            Vec::new(),
        )
        .render(&[offence("src/a.rs", 1, "header")]);

    // Assert
    assert!(
        report.contains("not applied: tests-layout (skipped), test-free-source (skipped)"),
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
    assert!(report.contains(
        "files_scanned=9 files_excluded=0 offences=0 baselined=0 fixed=0 rules_broken=0"
    ));
}

// "All rules are satisfied" is only true when all of them ran.
#[test]
fn render_of_no_offences_with_a_skipped_rule_says_applied_rules_are_satisfied() {
    // Arrange & Act
    let report = ReportPrinter::new(9)
        .with_rules(
            vec!["header".to_string()],
            vec!["tests-layout".to_string()],
            Vec::new(),
        )
        .render(&[]);

    // Assert
    assert!(
        report.contains("All applied rules are satisfied."),
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

// Skipped and unconfigured are both "did not run" and are not the same thing.
// One is a choice the reader made; the other is a flag they did not pass, and
// saying which is the difference between a note and an instruction.
#[test]
fn render_says_why_each_rule_was_not_applied() {
    // Arrange & Act
    let report = ReportPrinter::new(1)
        .with_rules(
            vec!["readable-source".to_string()],
            vec!["tests-layout".to_string()],
            vec!["header".to_string()],
        )
        .render(&[]);

    // Assert
    assert!(report.contains("tests-layout (skipped)"), "got {report}");
    // Not "(needs --header-file)": two rules can go unconfigured now, and the
    // header rule is not the one the other is waiting on.
    assert!(report.contains("header (not configured)"), "got {report}");
    assert!(
        report.contains("rules_applied=1 rules_skipped=1 rules_unconfigured=1"),
        "got {report}"
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
    assert!(report.contains(
        "files_scanned=5 files_excluded=0 offences=2 baselined=0 fixed=0 rules_broken=1"
    ));
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
        report.contains(
            "files_scanned=1 files_excluded=0 offences=10 baselined=0 fixed=0 rules_broken=1"
        ),
        "got {report}"
    );
}

// These three builders shipped in 0.4.0 with no test touching them directly.
// tested-public-api found all six -- three here and three on the JSON printer.
#[test]
fn with_baseline_puts_the_suppressed_count_in_the_report() {
    // Arrange
    let printer = ReportPrinter::new(1).with_baseline(Some("bl.json".to_string()), 7, 2);

    // Act
    let report = printer.render(&[]);

    // Assert
    assert!(
        report.contains("baseline: bl.json (7 suppressed)"),
        "{report}"
    );
    assert!(
        report.contains("2 baseline entries matched nothing"),
        "{report}"
    );
    assert!(report.contains("baselined=7"), "{report}");
}

#[test]
fn with_config_file_names_the_config_the_run_used() {
    // Arrange
    let printer = ReportPrinter::new(1).with_config_file(Some("stern4rust.toml".to_string()));

    // Act
    let report = printer.render(&[]);

    // Assert
    assert!(report.contains("config: stern4rust.toml"), "{report}");
}

#[test]
fn with_fixed_reports_how_many_files_were_rewritten() {
    // Arrange
    let printer = ReportPrinter::new(1).with_fixed(12);

    // Act
    let report = printer.render(&[]);

    // Assert
    assert!(report.contains("fixed: 12 file(s) rewritten"), "{report}");
    assert!(report.contains("fixed=12"), "{report}");
}
