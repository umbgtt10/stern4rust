// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// One classified item and the lines it occupies. The sort key is the part that
// carries a decision: a constant is SHOUTED and a helper is not, so sorting by
// byte value would put every constant before every helper for reasons that have
// nothing to do with the alphabet.

use stern4rust::section::Section;
use stern4rust::test_file_item::TestFileItem;

fn item(name: &str) -> TestFileItem {
    TestFileItem::new(Section::Helpers, name.to_string(), 1, 2)
}

#[test]
fn new_keeps_every_field_it_was_given() {
    // Arrange & Act
    let subject = TestFileItem::new(Section::Tests, "alpha".to_string(), 10, 14);

    // Assert
    assert_eq!(subject.section, Section::Tests);
    assert_eq!(subject.name, "alpha");
    assert_eq!(subject.first_line, 10);
    assert_eq!(subject.last_line, 14);
}

#[test]
fn sort_key_ignores_case() {
    // Arrange
    let shouted = item("RECOVERY_RANGE");
    let quiet = item("recovery_range");

    // Act & Assert
    assert_eq!(shouted.sort_key(), quiet.sort_key());
}

// The case that matters: byte order puts every capital before every lowercase,
// so a SHOUTED constant would always sort ahead of a helper named after it.
#[test]
fn sort_key_orders_a_shouted_name_against_a_quiet_one_alphabetically() {
    // Arrange
    let shouted = item("ZULU");
    let quiet = item("alpha");

    // Act & Assert
    assert!(quiet.sort_key() < shouted.sort_key());
}

#[test]
fn sort_key_orders_two_quiet_names_alphabetically() {
    // Arrange
    let first = item("alpha");
    let second = item("beta");

    // Act & Assert
    assert!(first.sort_key() < second.sort_key());
}
