// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A type's inherent impl lives in one file. A type with inherent impl blocks in
// two or more files of its crate is reported once, at its declaration, naming
// every file; trait impls are left alone, a name declared in two files is read
// as alternatives, and tests/ is exempt.

use stern4rust::reporting::offence::Offence;
use stern4rust::rule::Rule;
use stern4rust::rules::source::implemented_in_one_file_rule::ImplementedInOneFileRule;
use stern4rust::source_file::SourceFile;

const RULE: &str = "implemented-in-one-file";

fn check_workspace(files: &[(&str, &str)]) -> Vec<Offence> {
    let files: Vec<SourceFile> = files
        .iter()
        .map(|(file, contents)| SourceFile::new(file, contents))
        .collect();
    ImplementedInOneFileRule::new().check_workspace(&files)
}

// This rule answers the tree question only, and says so rather than inheriting
// a silence nobody chose.
#[test]
fn check_of_a_file_reports_nothing() {
    // Arrange
    let file = SourceFile::new("src/store.rs", "pub struct Store;\n\nimpl Store {}\n");

    // Act
    let offences = ImplementedInOneFileRule::new().check(&file);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_names_every_file_that_implements_the_type() {
    // Arrange & Act
    let offences = check_workspace(&[
        (
            "src/store.rs",
            "pub struct Store<T>(T);\n\nimpl<T> Store<T> {}\n",
        ),
        ("src/bytes.rs", "impl Store<u8> {}\n"),
        ("src/text.rs", "impl crate::store::Store<String> {}\n"),
    ]);

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(
        offences[0].description,
        "`Store` is implemented in 3 files: `src/bytes.rs`, `src/store.rs`, `src/text.rs`"
    );
}

#[test]
fn check_workspace_of_a_trait_impl_elsewhere_reports_nothing() {
    // Arrange & Act
    let offences = check_workspace(&[
        ("src/store.rs", "pub struct Store;\n\nimpl Store {}\n"),
        (
            "src/store_display.rs",
            "impl std::fmt::Display for Store {}\n",
        ),
    ]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_of_a_type_declared_nowhere_offers_one_file() {
    // Arrange & Act
    let offences = check_workspace(&[
        ("src/a.rs", "impl Store {}\n"),
        ("src/b.rs", "impl Store {}\n"),
    ]);

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].file, "src/a.rs");
    assert_eq!(
        offences[0].correction,
        "move every `impl Store` into one file, or split `Store` into types of their own"
    );
}

#[test]
fn check_workspace_of_a_type_implemented_in_one_file_reports_nothing() {
    // Arrange & Act
    let offences = check_workspace(&[(
        "src/store.rs",
        "pub struct Store;\n\nimpl Store {}\n\nimpl Store {}\n",
    )]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_of_a_type_implemented_in_two_files_reports_it_at_its_declaration() {
    // Arrange & Act
    let offences = check_workspace(&[
        (
            "src/store.rs",
            "use std::fmt;\n\npub struct Store;\n\nimpl Store {}\n",
        ),
        ("src/store_queries.rs", "impl Store {}\n"),
    ]);

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].rule, RULE);
    assert_eq!(offences[0].file, "src/store.rs");
    assert_eq!(offences[0].line, 3);
    assert_eq!(offences[0].subject.as_deref(), Some("Store"));
}

#[test]
fn check_workspace_of_a_type_split_by_platform_offers_a_trait() {
    // Arrange & Act
    let offences = check_workspace(&[
        (
            "src/lib.rs",
            "pub mod flash;\n#[cfg(feature = \"qemu\")]\npub mod qemu;\n#[cfg(feature = \"board\")]\npub mod board;\n",
        ),
        ("src/flash.rs", "pub struct Flash;\n\nimpl Flash {}\n"),
        ("src/qemu.rs", "impl Flash {}\n"),
        ("src/board.rs", "impl Flash {}\n"),
    ]);

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(
        offences[0].correction,
        "put the contract in a trait in `traits/` and give each platform its own implementing type"
    );
}

#[test]
fn check_workspace_of_alternatives_each_beside_its_declaration_reports_nothing() {
    // Arrange & Act
    let offences = check_workspace(&[
        ("src/qemu.rs", "pub struct Flash;\n\nimpl Flash {}\n"),
        ("src/board.rs", "pub struct Flash;\n\nimpl Flash {}\n"),
    ]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_of_alternatives_with_an_impl_astray_names_it_and_offers_their_declarations() {
    // Arrange & Act
    let offences = check_workspace(&[
        ("src/qemu.rs", "pub struct Flash;\n\nimpl Flash {}\n"),
        ("src/board.rs", "pub struct Flash;\n"),
        ("src/flash_ops.rs", "impl Flash {}\n"),
    ]);

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(
        offences[0].description,
        "`Flash` is declared in 2 files and implemented in 1 that declares neither: \
         `src/flash_ops.rs`"
    );
    assert_eq!(
        offences[0].correction,
        "move each `impl Flash` beside the declaration it is for, or give the types that \
         share the name names of their own"
    );
}

#[test]
fn check_workspace_of_impls_in_tests_reports_nothing() {
    // Arrange & Act
    let offences = check_workspace(&[
        ("tests/store_tests.rs", "struct Store;\n\nimpl Store {}\n"),
        ("tests/support/store_fake.rs", "impl Store {}\n"),
    ]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_offers_the_declaring_file_or_a_split() {
    // Arrange & Act
    let offences = check_workspace(&[
        ("src/store.rs", "pub struct Store;\n\nimpl Store {}\n"),
        ("src/store_queries.rs", "impl Store {}\n"),
    ]);

    // Assert
    assert_eq!(
        offences[0].correction,
        "move every `impl Store` into `src/store.rs`, or split `Store` into types of their own"
    );
}

#[test]
fn is_configured_for_a_rule_that_needs_nothing_returns_true() {
    // Arrange & Act
    let configured = ImplementedInOneFileRule::new().is_configured();

    // Assert
    assert!(configured);
}

#[test]
fn name_is_the_kebab_case_rule_name_used_in_the_report() {
    // Arrange & Act
    let name = ImplementedInOneFileRule::new().name();

    // Assert
    assert_eq!(name, RULE);
}

// A rule that is always configured has nothing to ask for, and says so rather
// than inventing a requirement it does not have.
#[test]
fn requirement_of_a_rule_that_needs_nothing_is_none() {
    // Arrange & Act
    let requirement = ImplementedInOneFileRule::new().requirement();

    // Assert
    assert!(requirement.is_none());
}
