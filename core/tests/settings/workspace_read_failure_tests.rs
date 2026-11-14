// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::support::in_memory_source_tree::NotHeld;
use crate::support::in_memory_source_tree::Wrapped;
use std::error::Error;
use stern4rust::settings::workspace_read_failure::WorkspaceReadFailure;

// It says what the reader said, so cargo's own words reach the reader
// unchanged and nothing is stated twice.
// The reader's cause speaks for itself: what it names beneath it is the
// failure's source.
#[test]
fn source_of_a_cause_naming_a_reason_of_its_own_is_that_reason() {
    // Arrange
    let failure = WorkspaceReadFailure {
        cause: Box::new(Wrapped { cause: NotHeld }),
    };

    // Act
    let source = failure.source().map(ToString::to_string);

    // Assert
    assert_eq!(source.as_deref(), Some("not held"));
}

#[test]
fn source_of_a_cause_with_no_reason_of_its_own_is_none() {
    // Arrange
    let failure = WorkspaceReadFailure {
        cause: Box::new(NotHeld),
    };

    // Act
    let source = failure.source();

    // Assert
    assert!(source.is_none());
}

#[test]
fn to_string_is_the_readers_own_words() {
    // Arrange & Act
    let text = WorkspaceReadFailure {
        cause: Box::new(NotHeld),
    }
    .to_string();

    // Assert
    assert_eq!(text, "not held");
}
