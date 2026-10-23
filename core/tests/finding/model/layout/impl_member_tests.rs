// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Where one member of an inherent impl belongs against another: by kind, then
// constructors by how many parameters they take, then functions by name, byte
// by byte. Constants and types keep the order written.

use std::cmp::Ordering;
use stern4rust::finding::model::layout::impl_member::ImplMember;
use stern4rust::finding::model::layout::member_kind::MemberKind;

fn member(name: &str, kind: MemberKind, parameters: usize) -> ImplMember {
    ImplMember::new(String::from(name), kind, parameters, 1, 1)
}

#[test]
fn new_keeps_every_field_it_was_given() {
    // Arrange & Act
    let found = member("election", MemberKind::Public, 2);

    // Assert
    assert_eq!(found.name(), "election");
    assert_eq!(found.kind(), MemberKind::Public);
    assert_eq!(found.parameters(), 2);
    assert_eq!((found.first_line(), found.last_line()), (1, 1));
}

#[test]
fn order_of_a_constructor_against_a_pub_function_puts_it_first() {
    // Arrange & Act
    let order =
        member("new", MemberKind::Constructor, 0).order(&member("len", MemberKind::Public, 0));

    // Assert
    assert_eq!(order, Ordering::Less);
}

#[test]
fn order_of_constructors_puts_the_fewest_parameters_first() {
    // Arrange & Act
    let order = member("from_parts", MemberKind::Constructor, 2).order(&member(
        "with_capacity",
        MemberKind::Constructor,
        1,
    ));

    // Assert
    assert_eq!(order, Ordering::Greater);
}

#[test]
fn order_of_constructors_with_as_many_parameters_is_alphabetical() {
    // Arrange & Act
    let order = member("new", MemberKind::Constructor, 1).order(&member(
        "empty",
        MemberKind::Constructor,
        1,
    ));

    // Assert
    assert_eq!(order, Ordering::Greater);
}

// Constants and types are not functions; the rule asks only that they come
// first, so among themselves they stay as written.
#[test]
fn order_of_two_heads_keeps_them_as_written() {
    // Arrange & Act
    let order = member("ZETA", MemberKind::Head, 0).order(&member("ALPHA", MemberKind::Head, 0));

    // Assert
    assert_eq!(order, Ordering::Equal);
}

#[test]
fn order_of_two_pub_functions_compares_names_byte_by_byte() {
    // Arrange & Act
    let order =
        member("alpha", MemberKind::Public, 0).order(&member("Zeta", MemberKind::Public, 0));

    // Assert
    assert_eq!(order, Ordering::Greater);
}
