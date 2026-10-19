// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A trait or type a file declares, where it declares it.

use stern4rust::finding::model::declared_item::DeclaredItem;

#[test]
fn new_keeps_every_field_it_was_given() {
    // Arrange & Act
    let found = DeclaredItem::new("Partitioner", 7);

    // Assert
    assert_eq!(found.name, "Partitioner");
    assert_eq!(found.line, 7);
}
