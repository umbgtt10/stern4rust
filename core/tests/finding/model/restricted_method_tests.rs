// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A method in an inherent impl whose visibility is neither `pub` nor private:
// how a call names it, the type it belongs to, and the form it was written in.

use stern4rust::finding::model::public_entry_point::PublicEntryPoint;
use stern4rust::finding::model::restricted_method::RestrictedMethod;

fn restricted(visibility: &str) -> RestrictedMethod {
    RestrictedMethod {
        call: PublicEntryPoint::new("election", 1),
        line: 12,
        visibility: String::from(visibility),
        owner: String::from("RaftProtocol"),
    }
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
