// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// One thing that does not belong in a registry file.
//
// It carries no behaviour beyond construction, so what is worth stating is that
// the line and the label stay attached to each other -- the whole reason this
// type exists is that the offence used to carry neither.

use stern4rust::finding::model::registry_item::RegistryItem;

#[test]
fn new_keeps_every_field_it_was_given() {
    // Arrange & Act
    let item = RegistryItem::new(7, "the constant `LIMIT`");

    // Assert
    assert_eq!(item.line, 7);
    assert_eq!(item.label, "the constant `LIMIT`");
}

#[test]
fn new_keeps_two_items_of_the_same_label_distinct_by_line() {
    // Arrange & Act
    let first = RegistryItem::new(3, "the function `helper`");
    let second = RegistryItem::new(9, "the function `helper`");

    // Assert
    assert_ne!(first.line, second.line);
}
