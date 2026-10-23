// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A method in an inherent impl whose visibility is neither `pub` nor private:
// how a call names it, the type it belongs to, and the form it was written in.

use stern4rust::finding::model::public_entry_point::PublicEntryPoint;
use stern4rust::finding::model::restricted_method::RestrictedMethod;

fn restricted(visibility: &str) -> RestrictedMethod {
    RestrictedMethod::new(
        PublicEntryPoint::new("election", 1),
        12,
        String::from(visibility),
        String::from("RaftProtocol"),
    )
}

#[test]
fn is_private_already_of_pub_crate_returns_false() {
    // Arrange & Act
    let private = restricted("pub(crate)").is_private_already();

    // Assert
    assert!(!private);
}

// `pub(self)` is private spelled long, so nothing outside the file can be
// relying on it.
#[test]
fn is_private_already_of_pub_self_returns_true() {
    // Arrange & Act
    let private = restricted("pub(self)").is_private_already();

    // Assert
    assert!(private);
}

#[test]
fn new_keeps_every_field_it_was_given() {
    // Arrange & Act
    let method = restricted("pub(crate)");

    // Assert
    assert_eq!(method.call(), &PublicEntryPoint::new("election", 1));
    assert_eq!(method.line(), 12);
    assert_eq!(method.visibility(), "pub(crate)");
    assert_eq!(method.owner(), "RaftProtocol");
}
