// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The files of a package a `#[cfg]` selects: a module whose `mod` declaration
// carries one, everything inside it, and a file opening with `#![cfg(...)]` --
// how a crate builds a different file for each platform.

use stern4rust::finding::parsing::modules::gated_modules::GatedModules;
use stern4rust::source_file::SourceFile;
use syn::Attribute;
use syn::ItemStruct;
use syn::parse_str;

fn attributes(source: &str) -> Vec<Attribute> {
    parse_str::<ItemStruct>(source)
        .expect("parses as a struct")
        .attrs
}

fn gated(files: &[(&str, &str)], path: &str) -> bool {
    let files: Vec<SourceFile> = files
        .iter()
        .map(|(file, contents)| SourceFile::new(file, contents))
        .collect();
    GatedModules::of(&files).gates(path)
}

#[test]
fn gates_a_file_inside_a_gated_module_returns_true() {
    // Arrange & Act
    let selected = gated(
        &[(
            "src/lib.rs",
            "#[cfg(feature = \"board\")]\npub mod hardware;\n",
        )],
        "src/hardware/board/flash.rs",
    );

    // Assert
    assert!(selected);
}

#[test]
fn gates_a_file_opening_with_a_cfg_returns_true() {
    // Arrange & Act
    let selected = gated(
        &[(
            "src/qemu.rs",
            "#![cfg(feature = \"qemu\")]\n\npub struct Flash;\n",
        )],
        "src/qemu.rs",
    );

    // Assert
    assert!(selected);
}

#[test]
fn gates_a_file_whose_declaration_carries_a_cfg_returns_true() {
    // Arrange & Act
    let selected = gated(
        &[("src/lib.rs", "#[cfg(feature = \"qemu\")]\npub mod qemu;\n")],
        "src/qemu.rs",
    );

    // Assert
    assert!(selected);
}

#[test]
fn gates_a_module_a_named_file_declares_returns_true() {
    // Arrange & Act
    let selected = gated(
        &[("src/platform.rs", "#[cfg(feature = \"qemu\")]\nmod qemu;\n")],
        "src/platform/qemu.rs",
    );

    // Assert
    assert!(selected);
}

#[test]
fn gates_a_module_a_nested_registry_gates_returns_true() {
    // Arrange & Act
    let selected = gated(
        &[
            ("src/lib.rs", "pub mod hardware;\n"),
            (
                "src/hardware/mod.rs",
                "#[cfg(feature = \"board\")]\npub mod board;\n",
            ),
        ],
        "src/hardware/board/flash.rs",
    );

    // Assert
    assert!(selected);
}

// The stem is a module's whole name, so `qemu` does not select `qemu_tools`.
#[test]
fn gates_a_module_whose_name_a_gated_one_begins_returns_false() {
    // Arrange & Act
    let selected = gated(
        &[(
            "src/lib.rs",
            "#[cfg(feature = \"qemu\")]\npub mod qemu;\npub mod qemu_tools;\n",
        )],
        "src/qemu_tools.rs",
    );

    // Assert
    assert!(!selected);
}

#[test]
fn gates_an_ungated_module_returns_false() {
    // Arrange & Act
    let selected = gated(
        &[("src/lib.rs", "pub mod hardware;\n")],
        "src/hardware/flash.rs",
    );

    // Assert
    assert!(!selected);
}

// `cfg_attr` sets an attribute under a condition; the module is built either way.
#[test]
fn gates_through_a_cfg_attr_returns_false() {
    // Arrange & Act
    let selected = gated(
        &[(
            "src/lib.rs",
            "#[cfg_attr(test, allow(dead_code))]\npub mod hardware;\n",
        )],
        "src/hardware.rs",
    );

    // Assert
    assert!(!selected);
}

#[test]
fn is_cfg_of_a_cfg_attribute_returns_true() {
    // Arrange
    let attrs = attributes("#[cfg(feature = \"qemu\")]\nstruct Flash;");

    // Act
    let gating = GatedModules::is_cfg(&attrs);

    // Assert
    assert!(gating);
}

#[test]
fn is_cfg_of_another_attribute_returns_false() {
    // Arrange
    let attrs = attributes("#[derive(Debug)]\nstruct Flash;");

    // Act
    let gating = GatedModules::is_cfg(&attrs);

    // Assert
    assert!(!gating);
}
