// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::support::in_memory_source_tree::NotHeld;
use std::error::Error;
use stern4rust::settings::adoption::baseline_failure::BaselineFailure;
use stern4rust::source_path::SourcePath;

fn path() -> SourcePath {
    SourcePath::new(String::from("/repo/stern4rust-baseline.json"))
}

#[test]
fn source_returns_the_cause() {
    // Arrange
    let failure = BaselineFailure::Unwritable {
        path: path(),
        cause: Box::new(NotHeld),
    };

    // Act
    let cause = failure.source().map(ToString::to_string);

    // Assert
    assert_eq!(cause, Some(String::from("not held")));
}

#[test]
fn to_string_names_the_file_and_what_went_wrong_with_it() {
    // Arrange
    let failures = [
        BaselineFailure::Unreadable {
            path: path(),
            cause: Box::new(NotHeld),
        },
        BaselineFailure::Invalid {
            path: path(),
            cause: Box::new(NotHeld),
        },
        BaselineFailure::Unrenderable {
            path: path(),
            cause: Box::new(NotHeld),
        },
        BaselineFailure::Unwritable {
            path: path(),
            cause: Box::new(NotHeld),
        },
    ];

    // Act
    let texts: Vec<String> = failures.iter().map(ToString::to_string).collect();

    // Assert
    assert_eq!(
        texts,
        [
            "/repo/stern4rust-baseline.json could not be read",
            "/repo/stern4rust-baseline.json is not a valid baseline",
            "/repo/stern4rust-baseline.json could not be rendered",
            "/repo/stern4rust-baseline.json could not be written",
        ]
    );
}
