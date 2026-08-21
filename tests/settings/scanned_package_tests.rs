// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// One package about to be walked, carrying what its manifest says about it.
//
// The licence is the field that matters: it is optional because a manifest may
// declare none, and that is the one case `spdx-matches-manifest` stands down on
// rather than guesses at.

use std::path::PathBuf;
use stern4rust::settings::scanned_package::ScannedPackage;

// Two packages of the same name from different roots are different packages,
// which is what lets the scan loop key configuration on one.
#[test]
fn eq_of_two_packages_differing_only_by_root_is_false() {
    // Arrange
    let left = ScannedPackage::new("node", PathBuf::from("a"), None);
    let right = ScannedPackage::new("node", PathBuf::from("b"), None);

    // Act & Assert
    assert_ne!(left, right);
}

#[test]
fn new_keeps_the_licence_it_was_given() {
    // Arrange
    let root = PathBuf::from("core");

    // Act
    let package = ScannedPackage::new("node", root.clone(), Some("Apache-2.0".to_string()));

    // Assert
    assert_eq!(package.name, "node");
    assert_eq!(package.root, root);
    assert_eq!(package.license.as_deref(), Some("Apache-2.0"));
}

// A manifest with no `license` field is not an error and not a default. It is
// the absence the rule reads as "nothing to hold this file to".
#[test]
fn new_without_a_licence_keeps_it_absent() {
    // Arrange & Act
    let package = ScannedPackage::new("node", PathBuf::from("core"), None);

    // Assert
    assert!(package.license.is_none());
}
