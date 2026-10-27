// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::support::in_memory_source_tree::NotHeld;
use std::error::Error;
use stern4rust::source_path::SourcePath;
use stern4rust::source_write_failure::SourceWriteFailure;

fn failure() -> SourceWriteFailure {
    SourceWriteFailure {
        path: SourcePath::new(String::from("/p/tests/a_tests.rs")),
        cause: Box::new(NotHeld),
    }
}

#[test]
fn source_returns_the_cause() {
    // Arrange
    let failure = failure();

    // Act
    let cause = failure.source().map(ToString::to_string);

    // Assert
    assert_eq!(cause, Some(String::from("not held")));
}

#[test]
fn to_string_names_the_file_that_could_not_be_rewritten() {
    // Arrange & Act
    let text = failure().to_string();

    // Assert
    assert_eq!(text, "/p/tests/a_tests.rs could not be rewritten");
}
