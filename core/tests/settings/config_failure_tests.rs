// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::support::in_memory_source_tree::NotHeld;
use std::error::Error;
use stern4rust::settings::config_failure::ConfigFailure;
use stern4rust::source_path::SourcePath;

fn path() -> SourcePath {
    SourcePath::new(String::from("/repo/stern4rust.toml"))
}

#[test]
fn source_returns_the_cause() {
    // Arrange
    let failure = ConfigFailure::Invalid {
        path: path(),
        cause: Box::new(NotHeld),
    };

    // Act
    let cause = failure.source().map(ToString::to_string);

    // Assert
    assert_eq!(cause, Some(String::from("not held")));
}

#[test]
fn to_string_of_a_file_that_cannot_be_read_says_so() {
    // Arrange & Act
    let text = ConfigFailure::Unreadable {
        path: path(),
        cause: Box::new(NotHeld),
    }
    .to_string();

    // Assert
    assert_eq!(text, "/repo/stern4rust.toml could not be read");
}

#[test]
fn to_string_of_a_file_that_is_not_configuration_says_so() {
    // Arrange & Act
    let text = ConfigFailure::Invalid {
        path: path(),
        cause: Box::new(NotHeld),
    }
    .to_string();

    // Assert
    assert_eq!(
        text,
        "/repo/stern4rust.toml is not valid stern4rust configuration"
    );
}
