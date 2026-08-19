// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The imports whose order rustfmt decides rather than the alphabet.
//
// rustfmt sorts `self`, `super` and `crate` ahead of every other path, and an
// uppercase-initial path behind them all. Neither matches a plain alphabetic
// sort, and unlike every other disagreement this tool can have with a
// formatter, this one has no resolution: `cargo fmt` writes one order, the rule
// demands another, and stage 1 runs the formatter first. A file caught between
// them cannot be fixed by hand.
//
// So the rule stands down on exactly those pairs. Everything else in the import
// list is still ordered, because among ordinary paths rustfmt's comparator and
// the alphabet agree.
//
// The shape this exists for: a shared helper inside the tests tree. Everything
// under tests/ is one crate rooted at all_tests.rs, so a sibling reaches a
// helper through `use crate::support::...`, and that is the trigger.

use stern4rust::import_path::ImportPath;

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
