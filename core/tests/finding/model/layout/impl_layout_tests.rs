// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// An inherent impl block's members as written and as they belong, and the
// first one out of place.

use stern4rust::finding::model::layout::impl_layout::ImplLayout;
use stern4rust::finding::model::layout::impl_member::ImplMember;
use stern4rust::finding::model::layout::member_kind::MemberKind;

fn layout(members: &[(&str, MemberKind)]) -> ImplLayout {
    ImplLayout {
        owner: String::from("Bookkeepers"),
        line: 1,
        open_line: 1,
        close_line: 20,
        members: members
            .iter()
            .enumerate()
            .map(|(at, (name, kind))| ImplMember {
                name: String::from(*name),
                kind: *kind,
                parameters: 0,
                first_line: at * 4 + 2,
                last_line: at * 4 + 4,
            })
            .collect(),
        refusal: None,
    }
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
    assert_eq!(block.ordered()[1].name, "catch_up");
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
    let names: Vec<&str> = block
        .ordered()
        .iter()
        .map(|member| member.name.as_str())
        .collect();

    // Assert
    assert_eq!(names, vec!["LIMIT", "new", "len", "helper"]);
}
