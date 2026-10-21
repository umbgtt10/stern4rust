// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Where each type of a package is declared and which files hold its inherent
// impls, by the last segment of the type's path. Trait impls are not counted; a
// name declared in two files is read as alternatives, each to be implemented
// beside its own declaration.

use stern4rust::finding::parsing::type_homes::TypeHomes;
use stern4rust::source_file::SourceFile;

fn homes(files: &[(&str, &str)]) -> TypeHomes {
    let files: Vec<SourceFile> = files
        .iter()
        .map(|(file, contents)| SourceFile::new(file, contents))
        .collect();
    TypeHomes::of("src/", &files)
}

fn spread_store() -> TypeHomes {
    homes(&[
        ("src/store.rs", "pub struct Store;\n\nimpl Store {}\n"),
        ("src/store_queries.rs", "\nimpl Store {}\n"),
    ])
}

#[test]
fn anchor_file_of_a_declared_type_is_its_declaration() {
    // Arrange & Act
    let file = spread_store().anchor_file("Store");

    // Assert
    assert_eq!(file, "src/store.rs");
}

#[test]
fn anchor_file_of_a_type_declared_nowhere_is_its_first_implementing_file() {
    // Arrange
    let homes = homes(&[
        ("src/b.rs", "impl Store {}\n"),
        ("src/a.rs", "impl Store {}\n"),
    ]);

    // Act
    let file = homes.anchor_file("Store");

    // Assert
    assert_eq!(file, "src/a.rs");
}

#[test]
fn anchor_line_of_a_declared_type_is_its_declaration_line() {
    // Arrange & Act
    let line = spread_store().anchor_line("Store");

    // Assert
    assert_eq!(line, 1);
}

#[test]
fn anchor_line_of_a_type_declared_nowhere_is_its_first_impl() {
    // Arrange
    let homes = homes(&[
        ("src/a.rs", "use std::fmt;\n\nimpl Store {}\n"),
        ("src/b.rs", "impl Store {}\n"),
    ]);

    // Act
    let line = homes.anchor_line("Store");

    // Assert
    assert_eq!(line, 3);
}

#[test]
fn declaring_files_lists_every_declaration_in_order() {
    // Arrange
    let homes = homes(&[
        ("src/qemu.rs", "pub struct Flash;\n\nimpl Flash {}\n"),
        ("src/board.rs", "pub struct Flash;\n\nimpl Flash {}\n"),
    ]);

    // Act
    let files = homes.declaring_files("Flash");

    // Assert
    assert_eq!(files, vec!["src/board.rs", "src/qemu.rs"]);
}

#[test]
fn implementing_files_lists_each_file_once_in_order() {
    // Arrange
    let homes = homes(&[
        ("src/store_queries.rs", "impl Store {}\n\nimpl Store {}\n"),
        ("src/store.rs", "pub struct Store;\n\nimpl Store {}\n"),
    ]);

    // Act
    let files = homes.implementing_files("Store");

    // Assert
    assert_eq!(files, vec!["src/store.rs", "src/store_queries.rs"]);
}

#[test]
fn of_leaves_a_file_outside_the_root_alone() {
    // Arrange & Act
    let homes = homes(&[
        ("src/store.rs", "pub struct Store;\n\nimpl Store {}\n"),
        ("tests/store_tests.rs", "impl Store {}\n"),
    ]);

    // Assert
    assert!(homes.spread().is_empty());
}

#[test]
fn of_leaves_a_file_that_does_not_parse_alone() {
    // Arrange & Act
    let homes = homes(&[
        ("src/store.rs", "pub struct Store;\n\nimpl Store {}\n"),
        ("src/broken.rs", "impl Store {\n"),
    ]);

    // Assert
    assert!(homes.spread().is_empty());
}

#[test]
fn outside_declarations_lists_the_implementing_files_that_declare_none() {
    // Arrange
    let homes = homes(&[
        ("src/qemu.rs", "pub struct Flash;\n\nimpl Flash {}\n"),
        ("src/board.rs", "pub struct Flash;\n\nimpl Flash {}\n"),
        ("src/flash_ops.rs", "impl Flash {}\n"),
    ]);

    // Act
    let files = homes.outside_declarations("Flash");

    // Assert
    assert_eq!(files, vec!["src/flash_ops.rs"]);
}

#[test]
fn split_by_platform_with_impls_a_cfg_selects_returns_true() {
    // Arrange
    let homes = homes(&[
        ("src/flash.rs", "pub struct Flash;\n"),
        ("src/qemu.rs", "#[cfg(feature = \"qemu\")]\nimpl Flash {}\n"),
        (
            "src/board.rs",
            "#[cfg(feature = \"board\")]\nimpl Flash {}\n",
        ),
    ]);

    // Act
    let split = homes.split_by_platform("Flash");

    // Assert
    assert!(split);
}

#[test]
fn split_by_platform_with_impls_in_two_gated_files_returns_true() {
    // Arrange
    let homes = homes(&[
        (
            "src/lib.rs",
            "#[cfg(feature = \"qemu\")]\npub mod qemu;\n#[cfg(feature = \"board\")]\npub mod board;\n",
        ),
        ("src/flash.rs", "pub struct Flash;\n\nimpl Flash {}\n"),
        ("src/qemu.rs", "impl Flash {}\n"),
        ("src/board.rs", "impl Flash {}\n"),
    ]);

    // Act
    let split = homes.split_by_platform("Flash");

    // Assert
    assert!(split);
}

#[test]
fn split_by_platform_with_one_gated_file_returns_false() {
    // Arrange
    let homes = homes(&[
        ("src/flash.rs", "pub struct Flash;\n\nimpl Flash {}\n"),
        ("src/qemu.rs", "#[cfg(feature = \"qemu\")]\nimpl Flash {}\n"),
    ]);

    // Act
    let split = homes.split_by_platform("Flash");

    // Assert
    assert!(!split);
}

#[test]
fn spread_counts_an_impl_inside_an_inline_module() {
    // Arrange & Act
    let homes = homes(&[
        ("src/store.rs", "pub struct Store;\n\nimpl Store {}\n"),
        (
            "src/queries.rs",
            "mod detail {\n    impl super::Store {}\n}\n",
        ),
    ]);

    // Assert
    assert_eq!(homes.spread(), vec!["Store"]);
}

#[test]
fn spread_counts_generic_concrete_and_path_qualified_impls_as_one_type() {
    // Arrange
    let homes = homes(&[
        (
            "src/store.rs",
            "pub struct Store<T>(T);\n\nimpl<T> Store<T> {}\n",
        ),
        ("src/bytes.rs", "impl Store<u8> {}\n"),
        ("src/text.rs", "impl crate::store::Store<String> {}\n"),
    ]);

    // Act
    let spread = homes.spread();

    // Assert
    assert_eq!(spread, vec!["Store"]);
    assert_eq!(homes.implementing_files("Store").len(), 3);
}

// A return type is not an impl block.
#[test]
fn spread_does_not_read_a_returned_impl_trait_as_an_impl() {
    // Arrange & Act
    let homes = homes(&[
        (
            "src/a.rs",
            "pub fn numbers() -> impl Iterator<Item = u8> {\n    0..3\n}\n",
        ),
        (
            "src/b.rs",
            "pub fn letters() -> impl Iterator<Item = char> {\n    'a'..'c'\n}\n",
        ),
    ]);

    // Assert
    assert!(homes.spread().is_empty());
}

#[test]
fn spread_leaves_trait_impls_alone() {
    // Arrange & Act
    let homes = homes(&[
        ("src/store.rs", "pub struct Store;\n\nimpl Store {}\n"),
        (
            "src/store_display.rs",
            "impl std::fmt::Display for Store {}\n",
        ),
    ]);

    // Assert
    assert!(homes.spread().is_empty());
}

#[test]
fn spread_of_a_type_implemented_away_from_its_declaration_in_one_file_is_empty() {
    // Arrange & Act
    let homes = homes(&[
        ("src/store.rs", "pub struct Store;\n"),
        ("src/store_impl.rs", "impl Store {}\n"),
    ]);

    // Assert
    assert!(homes.spread().is_empty());
}

#[test]
fn spread_of_a_type_implemented_in_one_file_is_empty() {
    // Arrange & Act
    let homes = homes(&[(
        "src/store.rs",
        "pub struct Store;\n\nimpl Store {}\n\nimpl Store {}\n",
    )]);

    // Assert
    assert!(homes.spread().is_empty());
}

#[test]
fn spread_of_a_type_implemented_in_two_files_names_it() {
    // Arrange & Act
    let spread = spread_store().spread();

    // Assert
    assert_eq!(spread, vec!["Store"]);
}

#[test]
fn spread_of_alternatives_each_beside_its_declaration_is_empty() {
    // Arrange & Act
    let homes = homes(&[
        ("src/qemu.rs", "pub struct Flash;\n\nimpl Flash {}\n"),
        ("src/board.rs", "pub struct Flash;\n\nimpl Flash {}\n"),
    ]);

    // Assert
    assert!(homes.spread().is_empty());
}

#[test]
fn spread_of_alternatives_with_an_impl_astray_names_the_type() {
    // Arrange & Act
    let homes = homes(&[
        ("src/qemu.rs", "pub struct Flash;\n\nimpl Flash {}\n"),
        ("src/board.rs", "pub struct Flash;\n"),
        ("src/flash_ops.rs", "impl Flash {}\n"),
    ]);

    // Assert
    assert_eq!(homes.spread(), vec!["Flash"]);
}
