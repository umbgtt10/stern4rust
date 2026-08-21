// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The imports whose order rustfmt decides rather than the alphabet.
//
// rustfmt sorts `self`, `super` and `crate` ahead of every other path, and it
// treats case as significant -- in a direction that depends on the style
// edition. 2021 sorts an uppercase-initial crate last and `from_str` ahead of
// `Value`; 2024 does the opposite of both. The two disagree with each other, so
// no single alphabet is right for both and standing down is the only answer
// correct under either.
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

use stern4rust::finding::model::import_path::ImportPath;

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
fn decides_order_of_a_path_extended_by_a_brace_group_is_true() {
    // Arrange & Act
    let decides = ImportPath::decides_order("use aaa::bbb;", "use aaa::bbb::{ccc, Ddd};");

    // Assert
    assert!(decides);
}

#[test]
fn decides_order_of_a_path_extended_by_a_glob_is_true() {
    // Arrange & Act
    let decides = ImportPath::decides_order("use hhh::iii;", "use hhh::iii::*;");

    // Assert
    assert!(decides);
}

// A brace group broken across lines reaches the rule as its first line only,
// so the text carries no closing `;` at all. It is still an extension.
#[test]
fn decides_order_of_a_path_extended_by_a_group_opened_on_the_line_is_true() {
    // Arrange & Act
    let decides = ImportPath::decides_order("use xxx::yyy;", "use xxx::yyy::{");

    // Assert
    assert!(decides);
}

// Not only uppercase extensions. Compared as written, `::` is 58 and the `;`
// ending the shorter line is 59, so a plain sort demands the longer path first
// whatever follows -- and rustfmt demands the shorter. Every extension is a
// disagreement, so every extension stands down.
#[test]
fn decides_order_of_a_path_extended_by_a_lowercase_segment_is_true() {
    // Arrange & Act
    let decides = ImportPath::decides_order("use aaa::bbb;", "use aaa::bbb::ccc;");

    // Assert
    assert!(decides);
}

// The shape `zip` could not see. `alloc::vec` and `alloc::vec::Vec` share every
// segment they have in common, so the search for a first difference found none
// and the case check never ran. The difference is real -- it is between nothing
// and `Vec` -- and rustfmt owns it.
//
// Measured against rustfmt at both style editions, because the two disagree
// about almost everything else in this file: 2021 sorts an uppercase-initial
// crate last and `from_str` ahead of `Value`, 2024 does the opposite of both.
// On this one they agree, and the shorter path goes first either way.
#[test]
fn decides_order_of_a_path_extended_by_an_uppercase_segment_is_true() {
    // Arrange & Act
    let decides = ImportPath::decides_order("use alloc::vec;", "use alloc::vec::Vec;");

    // Assert
    assert!(decides);
}

#[test]
fn decides_order_of_a_renamed_path_beside_a_longer_one_is_false() {
    // Arrange & Act
    let decides = ImportPath::decides_order("use ddd::eee as ggg;", "use ddd::eee::fff;");

    // Assert
    assert!(!decides);
}

// A rename is not an extension: `bbb as ccc` is one segment, not `bbb`
// followed by another. rustfmt puts the rename first at both editions and the
// text comparison already agrees, so this pair stays the alphabet's to judge --
// standing down here would lose a check that works.
#[test]
fn decides_order_of_a_renamed_path_beside_its_bare_form_is_false() {
    // Arrange & Act
    let decides = ImportPath::decides_order("use aaa::bbb as ccc;", "use aaa::bbb;");

    // Assert
    assert!(!decides);
}

// Identical paths are not extensions of one another, and nothing about them
// needs rustfmt.
#[test]
fn decides_order_of_two_identical_paths_is_false() {
    // Arrange & Act
    let decides = ImportPath::decides_order("use aaa::bbb;", "use aaa::bbb;");

    // Assert
    assert!(!decides);
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
