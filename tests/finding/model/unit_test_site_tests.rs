// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// One place in the source tree where a test, or the machinery of one, is living.
//
// It carries no behaviour beyond construction. What is worth stating is that the
// three parts stay attached: where it is, what it is, and what to do about it.
// The correction is built here rather than by the rule because the finder is the
// only thing that knows which of the two offences it found.

use stern4rust::finding::model::unit_test_site::UnitTestSite;

#[test]
fn new_keeps_every_field_it_was_given() {
    // Arrange & Act
    let site = UnitTestSite::new(12, "the `#[cfg(test)]` module `tests`", "move it");

    // Assert
    assert_eq!(site.line, 12);
    assert_eq!(site.label, "the `#[cfg(test)]` module `tests`");
    assert_eq!(site.correction, "move it");
}

#[test]
fn new_keeps_two_sites_of_the_same_label_distinct_by_line() {
    // Arrange & Act
    let first = UnitTestSite::new(3, "the test function `alpha`", "move it");
    let second = UnitTestSite::new(9, "the test function `alpha`", "move it");

    // Assert
    assert_ne!(first.line, second.line);
}
