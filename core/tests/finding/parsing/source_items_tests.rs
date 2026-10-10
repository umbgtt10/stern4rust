// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The parsed items of a file under a source root, or none, and the offences a
// rule finds in them. The rules that judge `src/` alone each opened with the
// same guard and parse and closed with the same find-then-judge.

use stern4rust::finding::parsing::source_items::SourceItems;
use stern4rust::reporting::offence::Offence;
use stern4rust::source_file::SourceFile;
use syn::Item;

const ROOT: &str = "src/";

fn every_item(items: &[Item]) -> Vec<&Item> {
    items.iter().collect()
}

fn one_offence(_file: &SourceFile, _item: &Item) -> Vec<Offence> {
    vec![Offence::new(
        "src/lib.rs",
        1,
        "probe",
        String::from("found"),
        String::from("none"),
    )]
}

#[test]
fn offences_for_a_file_outside_the_root_returns_nothing() {
    // Arrange
    let file = SourceFile::new("tests/lib_tests.rs", "fn first() {}");

    // Act
    let offences = SourceItems::offences(ROOT, &file, every_item, one_offence);

    // Assert
    assert!(offences.is_empty());
}

#[test]
fn offences_judges_every_item_found_in_a_file_inside_the_root() {
    // Arrange
    let file = SourceFile::new("src/lib.rs", "fn first() {}\nfn second() {}");

    // Act
    let offences = SourceItems::offences(ROOT, &file, every_item, one_offence);

    // Assert
    assert_eq!(offences.len(), 2);
}

#[test]
fn under_with_a_file_inside_the_root_returns_its_items() {
    // Arrange
    let file = SourceFile::new("src/lib.rs", "fn first() {}\nfn second() {}");

    // Act
    let items = SourceItems::under(ROOT, &file);

    // Assert
    assert_eq!(items.len(), 2);
}

#[test]
fn under_with_a_file_outside_the_root_returns_nothing() {
    // Arrange
    let file = SourceFile::new("tests/lib_tests.rs", "fn first() {}");

    // Act
    let items = SourceItems::under(ROOT, &file);

    // Assert
    assert!(items.is_empty());
}

// A file that does not parse is rustc's to report, not a rule's.
#[test]
fn under_with_a_file_that_does_not_parse_returns_nothing() {
    // Arrange
    let file = SourceFile::new("src/lib.rs", "fn (");

    // Act
    let items = SourceItems::under(ROOT, &file);

    // Assert
    assert!(items.is_empty());
}
