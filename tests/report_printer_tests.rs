// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Rendering the report. The printer writes to stdout, so what these pin is that
// it survives the shapes it will meet -- no offences, one, many, and a rule name
// or path far wider than its column heading -- rather than the exact bytes.

use stern4rust::offence::Offence;
use stern4rust::report_printer::ReportPrinter;

fn offence(file: &str, line: usize, rule: &'static str) -> Offence {
    Offence::new(file, line, rule, "something is wrong".to_string())
}

#[test]
fn print_with_no_offences_does_not_panic() {
    // Arrange
    let printer = ReportPrinter::new(0);

    // Act & Assert
    printer.print(&[]);
}

#[test]
fn print_with_one_offence_does_not_panic() {
    // Arrange
    let printer = ReportPrinter::new(1);

    // Act & Assert
    printer.print(&[offence("src/a.rs", 1, "header")]);
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
