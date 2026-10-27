// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use xtask::boundary::package_dependencies::PackageDependencies;

#[test]
fn of_a_document_that_is_not_json_returns_an_error() {
    // Arrange & Act
    let dependencies = PackageDependencies::new().of("not json", "core");

    // Assert
    assert!(dependencies.is_err());
}

// A crate a package uses in its code and its tests is declared twice, once per
// section; the gate names it once.
#[test]
fn of_a_package_declaring_a_crate_in_two_sections_names_it_once() {
    // Arrange
    let metadata = r#"{"packages":[{"name":"core","dependencies":[{"name":"clap","kind":null},{"name":"syn","kind":null},{"name":"clap","kind":"dev"}]}]}"#;

    // Act
    let dependencies = PackageDependencies::new().of(metadata, "core");

    // Assert
    assert_eq!(
        dependencies.expect("cargo metadata's document"),
        Some(vec![String::from("clap"), String::from("syn")])
    );
}

#[test]
fn of_a_package_in_the_document_names_its_dependencies_in_every_section() {
    // Arrange
    let metadata = r#"{"packages":[{"name":"core","dependencies":[{"name":"syn","kind":null},{"name":"tempfile","kind":"dev"},{"name":"cc","kind":"build"}]}]}"#;

    // Act
    let dependencies = PackageDependencies::new().of(metadata, "core");

    // Assert
    assert_eq!(
        dependencies.expect("cargo metadata's document"),
        Some(vec![
            String::from("syn"),
            String::from("tempfile"),
            String::from("cc")
        ])
    );
}

#[test]
fn of_a_package_missing_from_the_document_returns_none() {
    // Arrange
    let metadata = r#"{"packages":[{"name":"cli","dependencies":[]}]}"#;

    // Act
    let dependencies = PackageDependencies::new().of(metadata, "core");

    // Assert
    assert_eq!(dependencies.expect("cargo metadata's document"), None);
}
