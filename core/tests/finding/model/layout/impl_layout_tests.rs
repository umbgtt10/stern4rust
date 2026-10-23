// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// An inherent impl block's members as written and as they belong, and the
// first one out of place.

use stern4rust::finding::model::layout::impl_layout::ImplLayout;
use stern4rust::finding::model::layout::impl_member::ImplMember;
use stern4rust::finding::model::layout::member_kind::MemberKind;

fn layout(members: &[(&str, MemberKind)]) -> ImplLayout {
    ImplLayout::new(
        String::from("Bookkeepers"),
        1,
        1,
        20,
        members
            .iter()
            .enumerate()
            .map(|(at, (name, kind))| {
                ImplMember::new(String::from(*name), *kind, 0, at * 4 + 2, at * 4 + 4)
            })
            .collect(),
        None,
    )
}

#[test]
fn first_misplaced_of_a_block_in_order_is_none() {
    // Arrange
    let block = layout(&[
        ("new", MemberKind::Constructor),
        ("catch_up", MemberKind::Public),
        ("cluster", MemberKind::Public),
    ]);

    // Act
    let misplaced = block.first_misplaced();

    // Assert
    assert_eq!(misplaced, None);
}

#[test]
fn first_misplaced_points_at_the_first_member_out_of_place() {
    // Arrange
    let block = layout(&[
        ("new", MemberKind::Constructor),
        ("cluster", MemberKind::Public),
        ("consensus", MemberKind::Public),
        ("catch_up", MemberKind::Public),
    ]);

    // Act
    let misplaced = block.first_misplaced();

    // Assert
    assert_eq!(misplaced, Some(1));
    assert_eq!(block.ordered()[1].name(), "catch_up");
}

#[test]
fn new_keeps_every_field_it_was_given() {
    // Arrange & Act
    let block = layout(&[("new", MemberKind::Constructor)]);

    // Assert
    assert_eq!(block.owner(), "Bookkeepers");
    assert_eq!(block.line(), 1);
    assert_eq!(block.open_line(), 1);
    assert_eq!(block.close_line(), 20);
    assert_eq!(block.members().len(), 1);
    assert_eq!(block.refusal(), None);
}

#[test]
fn ordered_puts_every_kind_in_its_place() {
    // Arrange
    let block = layout(&[
        ("helper", MemberKind::Other),
        ("len", MemberKind::Public),
        ("LIMIT", MemberKind::Head),
        ("new", MemberKind::Constructor),
    ]);

    // Act
    let names: Vec<&str> = block.ordered().iter().map(|member| member.name()).collect();

    // Assert
    assert_eq!(names, vec!["LIMIT", "new", "len", "helper"]);
}
