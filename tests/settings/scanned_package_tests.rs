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

#[test]
fn agreed_license_of_no_packages_returns_nothing() {
    // Arrange & Act
    let agreed = ScannedPackage::agreed_license(&[]);

    // Assert
    assert!(agreed.is_none());
}

// The case the old aggregate got wrong in the other direction: it compared a
// set of distinct licences against a count of packages, so four members all
// declaring one read as none declaring it.
#[test]
fn agreed_license_over_four_packages_declaring_one_licence_returns_it() {
    // Arrange
    let packages: Vec<ScannedPackage> = ["a", "b", "c", "d"]
        .iter()
        .map(|name| ScannedPackage::new(name, PathBuf::from(*name), Some("Apache-2.0".to_string())))
        .collect();

    // Act
    let agreed = ScannedPackage::agreed_license(&packages);

    // Assert
    assert_eq!(agreed.as_deref(), Some("Apache-2.0"));
}

// What the report answers for, and deliberately the weaker question: checking
// is per package, so a run-wide claim can only be made where there is nothing
// to disagree about.
#[test]
fn agreed_license_where_every_package_declares_the_same_one_returns_it() {
    // Arrange
    let packages = [
        ScannedPackage::new(
            "node",
            PathBuf::from("node"),
            Some("Apache-2.0".to_string()),
        ),
        ScannedPackage::new(
            "node-infra",
            PathBuf::from("node-infra"),
            Some("Apache-2.0".to_string()),
        ),
    ];

    // Act
    let agreed = ScannedPackage::agreed_license(&packages);

    // Assert
    assert_eq!(agreed.as_deref(), Some("Apache-2.0"));
}

#[test]
fn agreed_license_where_one_package_declares_nothing_returns_nothing() {
    // Arrange
    let packages = [
        ScannedPackage::new(
            "node",
            PathBuf::from("node"),
            Some("Apache-2.0".to_string()),
        ),
        ScannedPackage::new("other", PathBuf::from("other"), None),
    ];

    // Act
    let agreed = ScannedPackage::agreed_license(&packages);

    // Assert
    assert!(agreed.is_none());
}

#[test]
fn agreed_license_where_packages_disagree_returns_nothing() {
    // Arrange
    let packages = [
        ScannedPackage::new("tool", PathBuf::from("tool"), Some("MIT".to_string())),
        ScannedPackage::new(
            "node",
            PathBuf::from("node"),
            Some("Apache-2.0".to_string()),
        ),
    ];

    // Act
    let agreed = ScannedPackage::agreed_license(&packages);

    // Assert
    assert!(agreed.is_none());
}

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
