// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A source file has one subject: at most one type that carries behaviour.
//
// Plain data declarations are not subjects, so a file may hold as many structs
// and enums without impl blocks as its subject needs. What it may not hold is a
// second type with an impl block, because then the file has two things to be
// named after and its name can only describe one of them.
//
// The first implemented type is taken as the subject and every later one is
// reported, so the offence names the type to move rather than complaining that
// the file has too many.

use stern4rust::reporting::offence::Offence;
use stern4rust::rule::Rule;
use stern4rust::rules::source::single_implemented_type_rule::SingleImplementedTypeRule;
use stern4rust::source_file::SourceFile;

const HEADER: &str = "// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>\n\
                      // Licensed under the MIT License\n\
                      // SPDX-License-Identifier: MIT\n";
const RULE: &str = "single-implemented-type";

fn check(path: &str, body: &str) -> Vec<Offence> {
    SingleImplementedTypeRule::new().check(&SourceFile::new(path, &format!("{HEADER}\n{body}")))
}

// Data has no behaviour to be the subject of.
#[test]
fn check_a_file_of_plain_data_declarations_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/types.rs",
        "pub struct Request;\npub struct Response;\npub enum Kind {\n    One,\n}\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_a_file_that_does_not_parse_reports_nothing() {
    // Arrange & Act
    let offences = check("src/subject.rs", "struct Broken {\n");

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_a_file_with_one_implemented_type_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/widget.rs",
        "pub struct Widget;\n\nimpl Widget {\n    pub fn new() {}\n}\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// The subject may be accompanied by all the data it needs.
#[test]
fn check_a_file_with_one_subject_and_plain_data_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/widget.rs",
        "pub struct Options;\npub struct Widget;\n\nimpl Widget {}\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_a_file_with_two_implemented_types_reports_the_second() {
    // Arrange & Act
    let offences = check(
        "src/report_printer.rs",
        "pub struct ReportPrinter;\nimpl ReportPrinter {}\n\nstruct ColumnWidths;\nimpl ColumnWidths {}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].rule, RULE);
    assert!(
        offences[0].description.contains("ColumnWidths"),
        "got {}",
        offences[0].description
    );
    assert!(
        offences[0].correction.contains("column_widths.rs"),
        "got {}",
        offences[0].correction
    );
}

// An enum carrying behaviour is a subject exactly as a struct is.
#[test]
fn check_a_file_with_two_subjects_one_of_them_an_enum_reports_the_second() {
    // Arrange & Act
    let offences = check(
        "src/subject.rs",
        "pub struct A;\nimpl A {}\n\npub enum B {\n    One,\n}\nimpl B {}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
    assert!(offences[0].description.contains("B"));
}

// Tests have their own shape, and a test file legitimately holds several fakes
// that each carry an impl block.
#[test]
fn check_a_test_file_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "tests/subject_tests.rs",
        "struct FakeOne;\nimpl FakeOne {}\n\nstruct FakeTwo;\nimpl FakeTwo {}\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_reports_every_type_after_the_first() {
    // Arrange & Act
    let offences = check(
        "src/subject.rs",
        "pub struct A;\nimpl A {}\n\npub struct B;\nimpl B {}\n\npub struct C;\nimpl C {}\n",
    );

    // Assert
    assert_eq!(offences.len(), 2);
    assert!(offences[0].description.contains("B"));
    assert!(offences[1].description.contains("C"));
}

#[test]
fn name_is_the_kebab_case_rule_name_used_in_the_report() {
    // Arrange & Act
    let name = SingleImplementedTypeRule::new().name();

    // Assert
    assert_eq!(name, RULE);
}
