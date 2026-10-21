// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// What a member of an inherent impl is, in the order the kinds are listed:
// constants and types, constructors, `pub` functions, then the rest.

use stern4rust::finding::model::layout::member_kind::MemberKind;

#[test]
fn cmp_lists_heads_then_constructors_then_pub_functions_then_the_rest() {
    // Arrange
    let mut kinds = vec![
        MemberKind::Other,
        MemberKind::Public,
        MemberKind::Constructor,
        MemberKind::Head,
    ];

    // Act
    kinds.sort();

    // Assert
    assert_eq!(
        kinds,
        vec![
            MemberKind::Head,
            MemberKind::Constructor,
            MemberKind::Public,
            MemberKind::Other,
        ]
    );
}
