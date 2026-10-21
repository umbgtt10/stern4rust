// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Why crap4rust's output yielded no report: none was there, or the one there
// did not parse.

use xtask::crap::report_failure::ReportFailure;

#[test]
fn to_string_of_a_missing_report_says_none_was_found() {
    // Arrange & Act
    let text = ReportFailure::Missing.to_string();

    // Assert
    assert_eq!(text, "could not find a JSON report in crap4rust's output");
}

#[test]
fn to_string_of_an_unparseable_report_carries_the_reason() {
    // Arrange & Act
    let text = ReportFailure::Unparseable(String::from("EOF while parsing")).to_string();

    // Assert
    assert_eq!(text, "could not parse crap4rust JSON: EOF while parsing");
}
