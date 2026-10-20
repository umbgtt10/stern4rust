// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The first tuple in a type holding two elements of one type, wherever it sits.
// Types are compared as written, whitespace and lifetimes ignored: `&'a str` and
// `&'static str` are one type, and an alias that names a meaning is another.

use stern4rust::finding::parsing::repeated_tuple_finder::RepeatedTupleFinder;
use syn::Type;
use syn::parse_str;

fn ty(source: &str) -> Type {
    parse_str(source).expect("parses as a type")
}

// A callback's parameter list is its contract, not a tuple this type returns.
#[test]
fn first_in_a_callback_taking_two_of_one_type_finds_nothing() {
    // Arrange
    let callback = ty("impl Fn(u8, u8) -> u8");

    // Act
    let found = RepeatedTupleFinder::first_in(&callback);

    // Assert
    assert!(found.is_none());
}

// An alias names what a value means, which is the cure rather than a loophole.
#[test]
fn first_in_a_tuple_of_an_alias_and_its_target_finds_nothing() {
    // Arrange
    let aliased = ty("(Height, u64)");

    // Act
    let found = RepeatedTupleFinder::first_in(&aliased);

    // Assert
    assert!(found.is_none());
}

#[test]
fn first_in_a_tuple_of_distinct_types_finds_nothing() {
    // Arrange
    let distinct = ty("(PathBuf, u64)");

    // Act
    let found = RepeatedTupleFinder::first_in(&distinct);

    // Assert
    assert!(found.is_none());
}

#[test]
fn first_in_a_tuple_repeating_a_type_finds_the_tuple_and_the_type() {
    // Arrange
    let repeated = ty("(u64, u64)");

    // Act
    let found = RepeatedTupleFinder::first_in(&repeated).expect("a repeated tuple");

    // Assert
    assert_eq!(found.tuple, "(u64, u64)");
    assert_eq!(found.element, "u64");
}

#[test]
fn first_in_a_tuple_repeating_a_type_with_different_lifetimes_finds_it() {
    // Arrange
    let repeated = ty("(&'a str, usize, &'static str)");

    // Act
    let found = RepeatedTupleFinder::first_in(&repeated).expect("a repeated tuple");

    // Assert
    assert_eq!(found.tuple, "(&str, usize, &str)");
    assert_eq!(found.element, "&str");
}

#[test]
fn first_in_a_tuple_wrapped_in_a_generic_finds_it() {
    // Arrange
    let wrapped = ty("Option<(Instant, Instant)>");

    // Act
    let found = RepeatedTupleFinder::first_in(&wrapped).expect("a repeated tuple");

    // Assert
    assert_eq!(found.element, "Instant");
}

#[test]
fn first_in_a_vec_of_pairs_finds_the_pair() {
    // Arrange
    let pairs = ty("Vec<(usize, usize)>");

    // Act
    let found = RepeatedTupleFinder::first_in(&pairs).expect("a repeated tuple");

    // Assert
    assert_eq!(found.tuple, "(usize, usize)");
}

#[test]
fn first_in_the_unit_type_finds_nothing() {
    // Arrange
    let unit = ty("()");

    // Act
    let found = RepeatedTupleFinder::first_in(&unit);

    // Assert
    assert!(found.is_none());
}

#[test]
fn first_in_two_repeated_tuples_finds_the_outer_one() {
    // Arrange
    let nested = ty("((u8, u8), (u8, u8))");

    // Act
    let found = RepeatedTupleFinder::first_in(&nested).expect("a repeated tuple");

    // Assert
    assert_eq!(found.tuple, "((u8, u8), (u8, u8))");
    assert_eq!(found.element, "(u8, u8)");
}
