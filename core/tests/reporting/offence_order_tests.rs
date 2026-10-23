// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Where an offence sits in the report: grouped by file, then by line, then by
// rule. A named key orders exactly as the tuple it replaces did, and says which
// `&str` is the file and which the rule.

use stern4rust::reporting::offence_order::OffenceOrder;

#[test]
fn new_keeps_the_file_the_line_and_the_rule() {
    // Arrange & Act
    let order = OffenceOrder::new("src/a.rs", 3, "header");

    // Assert
    assert_eq!(order.file(), "src/a.rs");
    assert_eq!(order.line(), 3);
    assert_eq!(order.rule(), "header");
}

#[test]
fn new_orders_by_file_before_line() {
    // Arrange
    let earlier_file = OffenceOrder::new("src/a.rs", 9, "header");
    let later_file = OffenceOrder::new("src/b.rs", 1, "header");

    // Act
    let ordered = earlier_file < later_file;

    // Assert
    assert!(ordered);
}

#[test]
fn new_orders_by_rule_when_the_file_and_line_agree() {
    // Arrange
    let first = OffenceOrder::new("src/a.rs", 3, "header");
    let second = OffenceOrder::new("src/a.rs", 3, "imported-paths");

    // Act
    let ordered = first < second;

    // Assert
    assert!(ordered);
}
