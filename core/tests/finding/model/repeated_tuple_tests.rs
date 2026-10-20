// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A tuple holding two elements of one type, as written: the tuple, and the type
// only position tells apart.

use stern4rust::finding::model::repeated_tuple::RepeatedTuple;

#[test]
fn new_keeps_the_tuple_and_the_type_it_repeats() {
    // Arrange & Act
    let found = RepeatedTuple::new(String::from("(u64, u64)"), String::from("u64"));

    // Assert
    assert_eq!(found.tuple, "(u64, u64)");
    assert_eq!(found.element, "u64");
}
