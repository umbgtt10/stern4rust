// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Every .rs file opens with the repository's header.
//
// The interesting cases are all about what counts as "the same header". A rule
// this simple earns its keep only if it is exact -- a wrong year or a swapped
// licence line has to fail -- while not failing on the things that differ
// between one checkout and another: line endings, a byte order mark, and a
// trailing newline in the header file itself.

use stern4rust::rule::Rule;
use stern4rust::rules::header_rule::HeaderRule;
use stern4rust::source_file::SourceFile;

const HEADER: &str = "// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>\n\
                      // Licensed under the MIT License\n\
                      // SPDX-License-Identifier: MIT";

fn rule() -> HeaderRule {
    HeaderRule::new(HEADER.lines().map(str::to_string).collect())
}

fn file(contents: &str) -> SourceFile {
    SourceFile::new("src/subject.rs", contents)
}

fn with_body(header: &str) -> String {
    format!("{header}\n\npub struct Subject;\n")
}

#[test]
fn check_a_file_opening_with_the_header_reports_nothing() {
    // Arrange
    let subject = file(&with_body(HEADER));

    // Act
    let offences = rule().check(&subject);

    // Assert
    assert!(offences.is_empty());
}

#[test]
fn check_a_file_whose_first_line_differs_reports_that_line() {
    // Arrange
    let wrong_year = HEADER.replace("2025", "2024");
    let subject = file(&with_body(&wrong_year));

    // Act
    let offences = rule().check(&subject);

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].line, 1);
    assert_eq!(offences[0].rule, "header");
    assert_eq!(offences[0].file, "src/subject.rs");
}

// A wrong licence is the failure this rule exists to catch, and it sits on the
// second line rather than the first.
#[test]
fn check_a_file_carrying_the_wrong_licence_reports_the_licence_line() {
    // Arrange
    let wrong_licence = HEADER.replace("MIT License", "Apache License, Version 2.0");
    let subject = file(&with_body(&wrong_licence));

    // Act
    let offences = rule().check(&subject);

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].line, 2);
}

#[test]
fn check_a_file_missing_the_last_header_line_reports_it() {
    // Arrange
    let truncated = "// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>\n\
                     // Licensed under the MIT License\n\
                     \n\
                     pub struct Subject;\n";
    let subject = file(truncated);

    // Act
    let offences = rule().check(&subject);

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].line, 3);
}

#[test]
fn check_a_file_with_no_header_at_all_reports_the_first_line() {
    // Arrange
    let subject = file("pub struct Subject;\n");

    // Act
    let offences = rule().check(&subject);

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].line, 1);
}

// One offence per file, not one per header line. A file with no header would
// otherwise bury every other file in the report behind its own three rows.
#[test]
fn check_reports_only_the_first_divergence() {
    // Arrange
    let subject = file("// wrong\n// also wrong\n// wrong again\n\npub struct Subject;\n");

    // Act
    let offences = rule().check(&subject);

    // Assert
    assert_eq!(offences.len(), 1);
}

#[test]
fn check_an_empty_file_reports_that_it_carries_no_header() {
    // Arrange
    let subject = file("");

    // Act
    let offences = rule().check(&subject);

    // Assert
    assert_eq!(offences.len(), 1);
    assert!(offences[0].description.contains("empty"));
}

// A file that runs out before the header does, with no trailing newline to
// stand in for the missing lines. This is the only way to reach the
// too-short report: with a trailing newline the file has a blank line where a
// header line should be, which is a divergence and is reported as one.
#[test]
fn check_a_file_that_ends_before_the_header_does_reports_its_length() {
    // Arrange
    let subject = file("// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>");

    // Act
    let offences = rule().check(&subject);

    // Assert
    assert_eq!(offences.len(), 1);
    assert!(offences[0].description.contains("header is 3"));
}

// The same file with a trailing newline is a different case: it has a second
// line, and that line is blank where the licence should be.
#[test]
fn check_a_file_whose_second_line_is_blank_reports_the_divergence_not_the_length() {
    // Arrange
    let subject = file("// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>\n");

    // Act
    let offences = rule().check(&subject);

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].line, 2);
    assert!(offences[0].description.contains("Licensed under"));
}

// git rewrites line endings on checkout, so a byte-for-byte comparison would
// fail on every line of every file on a Windows working copy.
#[test]
fn check_a_file_saved_with_windows_line_endings_reports_nothing() {
    // Arrange
    let crlf = with_body(HEADER).replace('\n', "\r\n");
    let subject = file(&crlf);

    // Act
    let offences = rule().check(&subject);

    // Assert
    assert!(offences.is_empty());
}

// A byte order mark is invisible in an editor and would otherwise sit in front
// of the first character of line 1.
#[test]
fn check_a_file_carrying_a_byte_order_mark_reports_nothing() {
    // Arrange
    let with_bom = format!("\u{feff}{}", with_body(HEADER));
    let subject = file(&with_bom);

    // Act
    let offences = rule().check(&subject);

    // Assert
    assert!(offences.is_empty());
}

// The header is the opening of the file, so a file that carries it further down
// does not satisfy the rule.
#[test]
fn check_a_file_carrying_the_header_below_other_code_reports_the_first_line() {
    // Arrange
    let subject = file(&format!("pub struct Subject;\n\n{HEADER}\n"));

    // Act
    let offences = rule().check(&subject);

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].line, 1);
}

// With nothing to compare against the rule has no opinion, rather than failing
// every file in the workspace.
#[test]
fn check_with_an_empty_expected_header_reports_nothing() {
    // Arrange
    let subject = file("pub struct Subject;\n");

    // Act
    let offences = HeaderRule::new(Vec::new()).check(&subject);

    // Assert
    assert!(offences.is_empty());
}

#[test]
fn name_is_the_kebab_case_rule_name_used_in_the_report() {
    // Arrange & Act
    let name = rule().name();

    // Assert
    assert_eq!(name, "header");
}

#[test]
fn expected_returns_the_header_the_rule_was_built_with() {
    // Arrange & Act
    let expected = rule().expected().to_vec();

    // Assert
    assert_eq!(expected.len(), 3);
    assert_eq!(expected[2], "// SPDX-License-Identifier: MIT");
}
