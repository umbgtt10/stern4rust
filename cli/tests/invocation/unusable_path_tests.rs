// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use stern4rust_cli::invocation::unusable_path::UnusablePath;

#[test]
fn to_string_names_the_flag_and_the_path() {
    // Arrange
    let unusable = UnusablePath {
        flag: String::from("--manifest-path"),
        shown: String::from("/repo/bad\u{fffd}"),
    };

    // Act
    let text = unusable.to_string();

    // Assert
    assert_eq!(
        text,
        "--manifest-path /repo/bad\u{fffd} is not valid UTF-8, which stern4rust needs to read beside it"
    );
}
