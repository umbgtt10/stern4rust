// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use syn::parse_file;
use xtask::boundary::crate_root_attributes::CrateRootAttributes;

#[test]
fn of_a_crate_root_reads_each_inner_attribute_with_its_nested_names() {
    // Arrange
    let root = parse_file("#![no_std]\n#![forbid(unsafe_code, missing_docs)]\n\npub mod a;\n")
        .expect("valid Rust");

    // Act
    let attributes = CrateRootAttributes::new().of(&root);

    // Assert
    assert_eq!(
        attributes,
        vec![
            String::from("no_std"),
            String::from("forbid(unsafe_code, missing_docs)")
        ]
    );
}

#[test]
fn of_a_file_without_inner_attributes_returns_none() {
    // Arrange
    let root = parse_file("#[derive(Debug)]\npub struct A;\n").expect("valid Rust");

    // Act
    let attributes = CrateRootAttributes::new().of(&root);

    // Assert
    assert!(attributes.is_empty());
}
