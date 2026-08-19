// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Reading the expected header off disk. Every editor ends a file with a newline,
// so without trimming it the rule would demand a blank line at the top of every
// source file -- and the failure would be reported against line 4 of every file
// in the workspace, which is a confusing way to learn about a trailing newline.

use std::env;
use std::fs;
use stern4rust::settings::header_source::HeaderSource;

#[test]
fn parse_drops_several_trailing_blank_lines() {
    // Arrange & Act
    let header = HeaderSource::parse("// one\n\n\n   \n");

    // Assert
    assert_eq!(header, ["// one"]);
}

#[test]
fn parse_drops_the_trailing_newline_every_editor_adds() {
    // Arrange & Act
    let header = HeaderSource::parse("// one\n// two\n");

    // Assert
    assert_eq!(header.len(), 2);
}

// A blank line inside the header is part of it, unlike one at the end.
#[test]
fn parse_keeps_a_blank_line_between_header_lines() {
    // Arrange & Act
    let header = HeaderSource::parse("// one\n\n// two\n");

    // Assert
    assert_eq!(header, ["// one", "", "// two"]);
}

#[test]
fn parse_normalises_windows_line_endings() {
    // Arrange & Act
    let header = HeaderSource::parse("// one\r\n// two\r\n");

    // Assert
    assert_eq!(header, ["// one", "// two"]);
}

#[test]
fn parse_of_an_empty_file_returns_no_lines() {
    // Arrange & Act
    let header = HeaderSource::parse("\n\n");

    // Assert
    assert!(header.is_empty());
}

#[test]
fn parse_splits_the_header_into_lines() {
    // Arrange & Act
    let header = HeaderSource::parse("// one\n// two");

    // Assert
    assert_eq!(header, ["// one", "// two"]);
}

#[test]
fn parse_strips_a_leading_byte_order_mark() {
    // Arrange & Act
    let header = HeaderSource::parse("\u{feff}// one\n");

    // Assert
    assert_eq!(header, ["// one"]);
}

// parse is the half that shapes the text; read is the half that reaches the
// disk, and a missing file has to be an error rather than an empty header --
// otherwise a typo in --header-file would silently disable the rule.
#[test]
fn read_of_a_file_returns_its_lines() {
    // Arrange
    let path = env::temp_dir().join("stern4rust_header_read.txt");
    fs::write(
        &path,
        "// one
// two
",
    )
    .expect("write the header");

    // Act
    let header = HeaderSource::read(&path).expect("reads");

    // Assert
    assert_eq!(header, ["// one", "// two"]);
}

#[test]
fn read_of_a_missing_file_is_an_error() {
    // Arrange
    let path = env::temp_dir().join("stern4rust_header_absent.txt");
    let _ = fs::remove_file(&path);

    // Act
    let header = HeaderSource::read(&path);

    // Assert
    assert!(header.is_err());
}
