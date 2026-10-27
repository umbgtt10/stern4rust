// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::error::Error;
use stern4rust::settings::adoption::exclusion::Exclusion;

#[test]
fn source_returns_the_reason_the_glob_gave() {
    // Arrange
    let failure = Exclusion::new("fixture/[")
        .err()
        .expect("an unusable pattern");

    // Act
    let cause = failure.source().map(ToString::to_string);

    // Assert
    assert_eq!(
        cause,
        Some(String::from(
            "error parsing glob 'fixture/[': unclosed character class; missing ']'"
        ))
    );
}

#[test]
fn to_string_names_the_pattern() {
    // Arrange & Act
    let text = Exclusion::new("fixture/[")
        .err()
        .expect("an unusable pattern")
        .to_string();

    // Assert
    assert_eq!(text, "`fixture/[` is not a usable exclude pattern");
}
