// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Normalising a file once so no rule has to. Both normalisations exist because
// of how a file reaches the disk rather than what it says: git rewrites line
// endings on checkout, and editors add a byte order mark invisibly. A rule that
// compared raw bytes would fail every file on a Windows working copy and never
// on the maintainer's.

use stern4rust::source_file::SourceFile;

// A file of blank lines carries no header either, and treating it as content
// would report a confusing mismatch against whitespace.
#[test]
fn is_empty_of_a_file_holding_only_whitespace_returns_true() {
    // Arrange & Act
    let file = SourceFile::new("src/a.rs", "\n  \n\t\n");

    // Assert
    assert!(file.is_empty());
}

#[test]
fn is_empty_of_a_file_with_no_contents_returns_true() {
    // Arrange & Act
    let file = SourceFile::new("src/a.rs", "");

    // Assert
    assert!(file.is_empty());
}

#[test]
fn is_empty_of_a_file_with_one_line_of_content_returns_false() {
    // Arrange & Act
    let file = SourceFile::new("src/a.rs", "// header\n");

    // Assert
    assert!(!file.is_empty());
}

// The mark is only a mark at the very start; the same bytes further in are
// content and must survive.
#[test]
fn new_keeps_a_byte_order_mark_that_is_not_at_the_start() {
    // Arrange & Act
    let file = SourceFile::new("src/a.rs", "// header\n\u{feff}second");

    // Assert
    assert_eq!(file.lines()[1], "\u{feff}second");
}

// Reports are read by people and pasted into scripts, so paths are printed the
// same way whichever platform produced them.
#[test]
fn new_reports_a_windows_path_with_forward_slashes() {
    // Arrange & Act
    let file = SourceFile::new("src\\rules\\header_rule.rs", "");

    // Assert
    assert_eq!(file.relative_path(), "src/rules/header_rule.rs");
}

#[test]
fn new_splits_the_contents_into_lines() {
    // Arrange & Act
    let file = SourceFile::new("src/a.rs", "one\ntwo\nthree");

    // Assert
    assert_eq!(file.lines(), ["one", "two", "three"]);
}

#[test]
fn new_strips_a_leading_byte_order_mark() {
    // Arrange & Act
    let file = SourceFile::new("src/a.rs", "\u{feff}// header");

    // Assert
    assert_eq!(file.lines()[0], "// header");
}

#[test]
fn new_strips_the_carriage_return_from_windows_line_endings() {
    // Arrange & Act
    let file = SourceFile::new("src/a.rs", "one\r\ntwo\r\n");

    // Assert
    assert_eq!(file.lines()[0], "one");
    assert_eq!(file.lines()[1], "two");
}
