// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Where an offence sits in the report: grouped by file, then by line, then by
// rule. A named key orders exactly as the tuple it replaces did, and says which
// `&str` is the file and which the rule.

use stern4rust::reporting::offence_order::OffenceOrder;

#[test]
fn lt_orders_by_file_before_line() {
    // Arrange
    let earlier_file = OffenceOrder {
        file: "src/a.rs",
        line: 9,
        rule: "header",
    };
    let later_file = OffenceOrder {
        file: "src/b.rs",
        line: 1,
        rule: "header",
    };

    // Act
    let ordered = earlier_file < later_file;

    // Assert
    assert!(ordered);
}

#[test]
fn lt_orders_by_rule_when_the_file_and_line_agree() {
    // Arrange
    let first = OffenceOrder {
        file: "src/a.rs",
        line: 3,
        rule: "header",
    };
    let second = OffenceOrder {
        file: "src/a.rs",
        line: 3,
        rule: "imported-paths",
    };

    // Act
    let ordered = first < second;

    // Assert
    assert!(ordered);
}
