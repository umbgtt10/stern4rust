// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use xtask::shear::ignore_reasons::IgnoreReasons;

const NO_EXCEPTIONS: &str = r#"{"packages":[{"name":"slotgate","metadata":null}],"metadata":null}"#;

#[test]
fn unexplained_of_a_package_exception_without_a_reason_names_the_package_and_the_crate() {
    // Arrange
    let metadata = r#"{"packages":[{"name":"core","metadata":{"cargo-shear":{"ignored":["proc-macro2"]}}}],"metadata":null}"#;

    // Act
    let unexplained = IgnoreReasons::new().unexplained(metadata);

    // Assert
    assert_eq!(
        unexplained.expect("cargo metadata's document"),
        vec![String::from("core: proc-macro2")]
    );
}

#[test]
fn unexplained_of_a_workspace_exception_without_a_reason_names_the_workspace() {
    // Arrange
    let metadata =
        r#"{"packages":[],"metadata":{"cargo-shear":{"ignored":["anyhow"],"reasons":{}}}}"#;

    // Act
    let unexplained = IgnoreReasons::new().unexplained(metadata);

    // Assert
    assert_eq!(
        unexplained.expect("cargo metadata's document"),
        vec![String::from("workspace: anyhow")]
    );
}

#[test]
fn unexplained_of_a_workspace_without_exceptions_is_empty() {
    // Arrange & Act
    let unexplained = IgnoreReasons::new().unexplained(NO_EXCEPTIONS);

    // Assert
    assert_eq!(
        unexplained.expect("cargo metadata's document"),
        Vec::<String>::new()
    );
}

#[test]
fn unexplained_of_an_empty_reason_counts_it_unexplained() {
    // Arrange
    let metadata = r#"{"packages":[{"name":"core","metadata":{"cargo-shear":{"ignored":["proc-macro2"],"reasons":{"proc-macro2":""}}}}],"metadata":null}"#;

    // Act
    let unexplained = IgnoreReasons::new().unexplained(metadata);

    // Assert
    assert_eq!(
        unexplained.expect("cargo metadata's document"),
        vec![String::from("core: proc-macro2")]
    );
}

#[test]
fn unexplained_of_an_exception_with_its_reason_is_empty() {
    // Arrange
    let metadata = r#"{"packages":[{"name":"core","metadata":{"cargo-shear":{"ignored":["proc-macro2"],"reasons":{"proc-macro2":"span-locations gives syn's spans their lines"}}}}],"metadata":null}"#;

    // Act
    let unexplained = IgnoreReasons::new().unexplained(metadata);

    // Assert
    assert_eq!(
        unexplained.expect("cargo metadata's document"),
        Vec::<String>::new()
    );
}

#[test]
fn unexplained_of_text_that_is_not_json_fails() {
    // Arrange & Act
    let unexplained = IgnoreReasons::new().unexplained("error: could not find Cargo.toml");

    // Assert
    assert!(unexplained.is_err());
}
