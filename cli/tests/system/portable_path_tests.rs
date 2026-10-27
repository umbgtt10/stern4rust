// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::path::MAIN_SEPARATOR;
use std::path::Path;
use stern4rust_cli::system::portable_path::PortablePath;

#[test]
fn of_a_utf8_path_keeps_its_text() {
    // Arrange
    let path = Path::new("/p/src/lib.rs");

    // Act
    let source = PortablePath::of(path);

    // Assert
    assert_eq!(
        source.map(|source| source.to_string()),
        Some(String::from("/p/src/lib.rs"))
    );
}

// `\` separates only on Windows; elsewhere it is part of a name and must stay.
#[test]
fn of_text_turns_a_backslash_into_a_slash_only_where_it_separates() {
    // Arrange
    let text = "project\\src\\lib.rs";

    // Act
    let source = PortablePath::of_text(text);

    // Assert
    let expected = if MAIN_SEPARATOR == '\\' {
        "project/src/lib.rs"
    } else {
        "project\\src\\lib.rs"
    };
    assert_eq!(source.as_str(), expected);
}
