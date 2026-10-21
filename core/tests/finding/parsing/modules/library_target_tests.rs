// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The files of a package's library target: `src/lib.rs` and every file its
// module declarations reach, as `name.rs` or `name/mod.rs`. A binary's files --
// `src/main.rs`, `src/bin/`, and what only they declare -- are not among them.

use stern4rust::finding::parsing::modules::library_target::LibraryTarget;
use stern4rust::source_file::SourceFile;

fn target(files: &[(&str, &str)]) -> LibraryTarget {
    let files: Vec<SourceFile> = files
        .iter()
        .map(|(file, contents)| SourceFile::new(file, contents))
        .collect();
    LibraryTarget::of(&files)
}

#[test]
fn contains_a_file_a_declared_module_declares_returns_true() {
    // Arrange
    let library = target(&[
        ("src/lib.rs", "pub mod store;\n"),
        ("src/store/mod.rs", "pub mod shelf;\n"),
        ("src/store/shelf.rs", ""),
    ]);

    // Act
    let reached = library.contains("src/store/shelf.rs");

    // Assert
    assert!(reached);
}

#[test]
fn contains_a_file_only_the_binary_declares_returns_false() {
    // Arrange
    let library = target(&[
        ("src/lib.rs", "pub mod store;\n"),
        ("src/store.rs", ""),
        ("src/main.rs", "mod cli;\n\nfn main() {}\n"),
        ("src/cli.rs", ""),
    ]);

    // Act
    let reached = (
        library.contains("src/main.rs"),
        library.contains("src/cli.rs"),
    );

    // Assert
    assert_eq!(reached, (false, false));
}

#[test]
fn contains_a_file_the_crate_root_declares_returns_true() {
    // Arrange
    let library = target(&[("src/lib.rs", "pub mod store;\n"), ("src/store.rs", "")]);

    // Act
    let reached = library.contains("src/store.rs");

    // Assert
    assert!(reached);
}

#[test]
fn contains_of_a_package_without_a_library_returns_false() {
    // Arrange
    let library = target(&[
        ("src/main.rs", "mod cli;\n\nfn main() {}\n"),
        ("src/cli.rs", ""),
    ]);

    // Act
    let reached = library.contains("src/cli.rs");

    // Assert
    assert!(!reached);
}

#[test]
fn contains_the_crate_root_returns_true() {
    // Arrange
    let library = target(&[("src/lib.rs", "")]);

    // Act
    let reached = library.contains("src/lib.rs");

    // Assert
    assert!(reached);
}
