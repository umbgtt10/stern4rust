// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The names a file reads -- through a path, an import, a pattern or a macro's
// tokens -- and not the names it declares, so a file declaring a constant of
// its own is not taken for a reader of another file's.

use std::collections::BTreeSet;
use stern4rust::finding::parsing::read_names::ReadNames;
use stern4rust::source_file::SourceFile;

fn names(contents: &str) -> BTreeSet<String> {
    ReadNames::of(&SourceFile::new("src/reader.rs", contents))
}

#[test]
fn index_of_several_files_names_each_reader_by_its_path() {
    // Arrange
    let files = [
        SourceFile::new("node/src/limits.rs", "pub const LIMIT: u32 = 3;\n"),
        SourceFile::new("node/src/pool.rs", "use crate::limits::LIMIT;\n"),
        SourceFile::new(
            "types/tests/pool_tests.rs",
            "fn t() {\n    assert_eq!(size(), LIMIT);\n}\n",
        ),
    ];

    // Act
    let index = ReadNames::index(&files);

    // Assert
    assert_eq!(
        index.get("LIMIT"),
        Some(&BTreeSet::from([
            String::from("node/src/pool.rs"),
            String::from("types/tests/pool_tests.rs"),
        ]))
    );
    assert_eq!(index.get("UNREAD"), None::<&BTreeSet<String>>);
}

#[test]
fn of_a_file_declaring_a_constant_leaves_its_name_out() {
    // Arrange & Act
    let read = names("const LIMIT: u32 = 3;\n");

    // Assert
    assert!(!read.contains("LIMIT"), "{read:?}");
}

#[test]
fn of_a_file_importing_a_constant_reads_it() {
    // Arrange & Act
    let read = names("use crate::limits::LIMIT;\nuse crate::sizes::{PAGE as PAGE_SIZE, ROW};\n");

    // Assert
    assert!(read.contains("LIMIT"), "{read:?}");
    assert!(read.contains("PAGE"), "{read:?}");
    assert!(read.contains("ROW"), "{read:?}");
}

#[test]
fn of_a_file_matching_on_a_constant_reads_it() {
    // Arrange & Act
    let read = names(
        "fn kind(tag: u8) -> bool {\n    match tag {\n        VOTE_TAG => true,\n        _ => false,\n    }\n}\n",
    );

    // Assert
    assert!(read.contains("VOTE_TAG"), "{read:?}");
}

#[test]
fn of_a_file_reading_a_constant_in_a_macro_reads_it() {
    // Arrange & Act
    let read = names("fn t() {\n    assert_eq!(count(), LIMIT);\n}\n");

    // Assert
    assert!(read.contains("LIMIT"), "{read:?}");
}

#[test]
fn of_a_file_that_does_not_parse_is_empty() {
    // Arrange & Act
    let read = names("fn broken( {");

    // Assert
    assert!(read.is_empty(), "{read:?}");
}

#[test]
fn of_a_file_using_a_constant_by_path_reads_it() {
    // Arrange & Act
    let read = names("fn buffer() -> [u8; sizes::PAGE] {\n    [0; sizes::PAGE]\n}\n");

    // Assert
    assert!(read.contains("PAGE"), "{read:?}");
    assert!(read.contains("sizes"), "{read:?}");
}
