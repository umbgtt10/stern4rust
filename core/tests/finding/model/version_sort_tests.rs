// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The 2024 style edition's order for two names: the Rust style guide's version
// sorting, which rustfmt applies to import segments. Each expectation below is
// one `cargo fmt` was seen to leave on an edition 2024 crate.

use std::cmp::Ordering;
use stern4rust::finding::model::version_sort::VersionSort;

#[test]
fn compare_of_a_name_and_its_extension_puts_the_shorter_first() {
    // Arrange & Act
    let order = VersionSort::compare("select", "select4");

    // Assert
    assert_eq!(order, Ordering::Less);
}

// A number is compared by value however many digits it has, so one too long for
// any integer still sorts.
#[test]
fn compare_of_a_number_longer_than_any_integer_orders_it_by_value() {
    // Arrange & Act
    let order = VersionSort::compare("x99999999999999999999999", "x100000000000000000000000");

    // Assert
    assert_eq!(order, Ordering::Less);
}

// Equal by value, the number written with more leading zeros comes first.
#[test]
fn compare_of_equal_numbers_puts_the_one_with_more_leading_zeros_first() {
    // Arrange & Act
    let order = VersionSort::compare("x01", "x1");

    // Assert
    assert_eq!(order, Ordering::Less);
}

#[test]
fn compare_of_identical_names_is_equal() {
    // Arrange & Act
    let order = VersionSort::compare("alpha_2", "alpha_2");

    // Assert
    assert_eq!(order, Ordering::Equal);
}

// What a byte compare gets backwards: `1` sorts before `2`, so it puts `item10`
// first, and 2024 compares the numbers.
#[test]
fn compare_of_numbered_names_orders_them_by_value() {
    // Arrange & Act
    let order = VersionSort::compare("item2", "item10");

    // Assert
    assert_eq!(order, Ordering::Less);
}

#[test]
fn compare_puts_a_capital_before_a_lowercase_letter() {
    // Arrange & Act
    let order = VersionSort::compare("Beta", "alpha");

    // Assert
    assert_eq!(order, Ordering::Less);
}

#[test]
fn compare_puts_a_number_before_a_letter() {
    // Arrange & Act
    let order = VersionSort::compare("a1", "ab");

    // Assert
    assert_eq!(order, Ordering::Less);
}

// The other thing a byte compare gets backwards: `B` (0x42) sorts before `_`
// (0x5F), and 2024 puts an underscore before everything.
#[test]
fn compare_puts_an_underscore_before_a_capital() {
    // Arrange & Act
    let order = VersionSort::compare("a_b", "aB");

    // Assert
    assert_eq!(order, Ordering::Less);
}

#[test]
fn compare_puts_an_underscore_before_a_lowercase_letter() {
    // Arrange & Act
    let order = VersionSort::compare("a_b", "ab");

    // Assert
    assert_eq!(order, Ordering::Less);
}
