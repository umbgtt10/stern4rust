// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The imports whose order rustfmt decides rather than the alphabet.
//
// rustfmt sorts `self`, `super` and `crate` ahead of every other path, and it
// treats case as significant in opposite directions at the two levels: an
// uppercase-initial crate goes behind every lowercase one, an uppercase-initial
// segment later in a path goes ahead of its lowercase siblings.
//
// None matches a plain alphabetic sort, and unlike every other
// disagreement this tool can have with a formatter, this one has no resolution:
// `cargo fmt` writes one order, the rule demands another, and stage 1 runs the
// formatter first. A file caught between them cannot be fixed by hand.
//
// So the rule stands down on exactly those pairs. Everything else in the import
// list is still ordered, because among segments of the same case rustfmt's
// comparator and the alphabet agree.
//
// Two shapes trigger it. A shared helper inside the tests tree, reached through
// `use crate::support::...`, because everything under tests/ is one crate rooted
// at all_tests.rs. And a pair like `use serde_json::Value` beside
// `use serde_json::from_str`, which is only remarkable as a pair: both paths are
// ordinary, and they part company at a second segment of differing case.

use stern4rust::import_path::ImportPath;

#[test]
fn decides_order_of_a_pair_diverging_at_the_first_segment_is_false() {
    // Arrange & Act
    let decides = ImportPath::decides_order("use serde_json::Value;", "use stern4rust::a::B;");

    // Assert
    assert!(!decides);
}

// The pair that made this pairwise. cargo fmt writes Value first; a
// case-insensitive alphabet puts from_str first; neither yields.
#[test]
fn decides_order_of_a_pair_diverging_by_case_is_true() {
    // Arrange & Act
    let decides = ImportPath::decides_order("use serde_json::Value;", "use serde_json::from_str;");

    // Assert
    assert!(decides);
}

#[test]
fn decides_order_of_a_pair_of_lowercase_segments_is_false() {
    // Arrange & Act
    let decides = ImportPath::decides_order("use std::fs;", "use std::path::PathBuf;");

    // Assert
    assert!(!decides);
}

// Both uppercase, so the comparators agree and the alphabet still rules.
#[test]
fn decides_order_of_a_pair_of_uppercase_segments_is_false() {
    // Arrange & Act
    let decides = ImportPath::decides_order("use syn::Item;", "use syn::Type;");

    // Assert
    assert!(!decides);
}

#[test]
fn decides_order_of_a_pair_with_a_crate_path_is_true() {
    // Arrange & Act
    let decides = ImportPath::decides_order("use anyhow::Result;", "use crate::support::Thing;");

    // Assert
    assert!(decides);
}

#[test]
fn is_specially_ordered_of_a_crate_path_is_true() {
    // Arrange & Act
    let special = ImportPath::is_specially_ordered("use crate::support::builders::a_widget;");

    // Assert
    assert!(special);
}

// The first segment, not a prefix. A crate genuinely named `crateful` sorts
// alphabetically like anything else.
#[test]
fn is_specially_ordered_of_a_path_named_crateful_is_false() {
    // Arrange & Act
    let special = ImportPath::is_specially_ordered("use crateful::Thing;");

    // Assert
    assert!(!special);
}

#[test]
fn is_specially_ordered_of_a_pub_use_is_judged_on_its_path() {
    // Arrange & Act
    let special = ImportPath::is_specially_ordered("pub use crate::support::Thing;");

    // Assert
    assert!(special);
}

#[test]
fn is_specially_ordered_of_a_self_path_is_true() {
    // Arrange & Act
    let special = ImportPath::is_specially_ordered("use self::inner::Thing;");

    // Assert
    assert!(special);
}

#[test]
fn is_specially_ordered_of_a_std_path_is_false() {
    // Arrange & Act
    let special = ImportPath::is_specially_ordered("use std::path::PathBuf;");

    // Assert
    assert!(!special);
}

#[test]
fn is_specially_ordered_of_a_super_path_is_true() {
    // Arrange & Act
    let special = ImportPath::is_specially_ordered("use super::Parent;");

    // Assert
    assert!(special);
}

#[test]
fn is_specially_ordered_of_an_ordinary_path_is_false() {
    // Arrange & Act
    let special = ImportPath::is_specially_ordered("use anyhow::Result;");

    // Assert
    assert!(!special);
}

// rustfmt puts an uppercase-initial path last rather than among its letter.
#[test]
fn is_specially_ordered_of_an_uppercase_path_is_true() {
    // Arrange & Act
    let special = ImportPath::is_specially_ordered("use Uppercase::Thing;");

    // Assert
    assert!(special);
}
