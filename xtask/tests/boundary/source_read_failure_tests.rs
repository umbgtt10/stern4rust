// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::path::PathBuf;
use xtask::boundary::source_read_failure::SourceReadFailure;

#[test]
fn to_string_names_the_path_and_the_reason() {
    // Arrange
    let failure = SourceReadFailure {
        path: PathBuf::from("/w/core/src"),
        reason: String::from("No such file or directory"),
    };

    // Act
    let text = failure.to_string();

    // Assert
    assert_eq!(
        text,
        "could not read /w/core/src: No such file or directory"
    );
}
