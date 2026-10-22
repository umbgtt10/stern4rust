// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Whether a trait an impl names is a port -- one a test can hand the code an
// implementation of -- rather than the standard library's. A trait the package
// declares is one, and so is a dependency's: a family's core crate declares the
// ports its runtimes implement. Display, Iterator and the rest are a value's
// behaviour, not a seam.

use std::collections::BTreeSet;
use stern4rust::finding::parsing::time::imported_names::ImportedNames;
use stern4rust::finding::parsing::time::port_traits::PortTraits;
use syn::Item;
use syn::parse_file;

fn is_port(source: &str, declared: &[&str]) -> bool {
    let syntax = parse_file(source).expect("parses");
    let names = ImportedNames::of(&syntax.items);
    let declared: BTreeSet<String> = declared.iter().map(|name| (*name).to_string()).collect();
    let path = syntax
        .items
        .iter()
        .find_map(|item| match item {
            Item::Impl(block) => block.trait_.as_ref().map(|(_, path, _)| path.clone()),
            _ => None,
        })
        .expect("a trait impl");
    PortTraits::new(&declared, &names).is_port(&path)
}

#[test]
fn is_port_of_a_crate_path_to_a_trait_the_package_does_not_declare_is_false() {
    // Arrange & Act
    let port = is_port("impl crate::ports::Clock for SystemClock {}\n", &[]);

    // Assert
    assert!(!port);
}

#[test]
fn is_port_of_a_dependencys_trait_by_its_full_path_is_true() {
    // Arrange & Act
    let port = is_port(
        "impl etheram_core::timer_output::TimerOutput for Timer {}\n",
        &[],
    );

    // Assert
    assert!(port);
}

#[test]
fn is_port_of_a_dependencys_trait_through_an_import_is_true() {
    // Arrange & Act
    let port = is_port(
        "use etheram_core::timer_output::TimerOutput;\n\nimpl TimerOutput for Timer {}\n",
        &[],
    );

    // Assert
    assert!(port);
}

#[test]
fn is_port_of_a_prelude_trait_is_false() {
    // Arrange & Act
    let ports = [
        is_port("impl Iterator for Ticks {}\n", &[]),
        is_port("impl Default for Ticks {}\n", &[]),
        is_port("impl Drop for Ticks {}\n", &[]),
    ];

    // Assert
    assert_eq!(ports, [false, false, false]);
}

#[test]
fn is_port_of_a_standard_trait_through_a_module_is_false() {
    // Arrange & Act
    let port = is_port("use std::fmt;\n\nimpl fmt::Display for Elapsed {}\n", &[]);

    // Assert
    assert!(!port);
}

#[test]
fn is_port_of_a_standard_trait_through_an_import_is_false() {
    // Arrange & Act
    let port = is_port(
        "use std::fmt::Display;\n\nimpl Display for Elapsed {}\n",
        &[],
    );

    // Assert
    assert!(!port);
}

#[test]
fn is_port_of_a_trait_the_package_declares_is_true() {
    // Arrange & Act
    let ports = [
        is_port("impl Clock for SystemClock {}\n", &["Clock"]),
        is_port("impl crate::ports::Clock for SystemClock {}\n", &["Clock"]),
    ];

    // Assert
    assert_eq!(ports, [true, true]);
}
