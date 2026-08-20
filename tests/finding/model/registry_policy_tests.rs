// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// What counts as a declaration, which is not the same question in both trees.
//
// Under tests/, being compiled is the whole concern: a private `mod name;`
// compiles that file as well as a public one, so it is a declaration. Under
// src/, the module tree is the crate's shape, and a registry that hides part of
// it behind a private `mod` is describing something other than what it exports.
//
// `extern crate alloc;` is the one non-mod item a source registry may hold. A
// no_std crate has to say it somewhere and the crate root is where it belongs;
// no other extern crate has that excuse.

use stern4rust::finding::model::registry_policy::RegistryPolicy;
use syn::Item;
use syn::parse_file;

fn item(source: &str) -> Item {
    parse_file(source).expect("parses").items.remove(0)
}

#[test]
fn is_declaration_of_a_private_mod_under_the_source_policy_is_false() {
    // Arrange & Act
    let declaration = RegistryPolicy::source().is_declaration(&item("mod inner;\n"));

    // Assert
    assert!(!declaration);
}

#[test]
fn is_declaration_of_a_private_mod_under_the_tests_policy_is_true() {
    // Arrange & Act
    let declaration = RegistryPolicy::tests().is_declaration(&item("mod inner;\n"));

    // Assert
    assert!(declaration);
}

#[test]
fn is_declaration_of_a_pub_mod_is_true() {
    // Arrange & Act
    let source = RegistryPolicy::source().is_declaration(&item("pub mod inner;\n"));
    let tests = RegistryPolicy::tests().is_declaration(&item("pub mod inner;\n"));

    // Assert
    assert!(source);
    assert!(tests);
}

#[test]
fn is_declaration_of_an_import_is_false() {
    // Arrange & Act
    let declaration = RegistryPolicy::source().is_declaration(&item("use alpha::One;\n"));

    // Assert
    assert!(!declaration);
}

// A declaration points at a file. A module with a body is code living in the
// one file a reader scans expecting a list.
#[test]
fn is_declaration_of_an_inline_module_is_false() {
    // Arrange & Act
    let declaration = RegistryPolicy::source().is_declaration(&item("pub mod inner { }\n"));

    // Assert
    assert!(!declaration);
}

#[test]
fn is_declaration_of_extern_crate_alloc_under_the_source_policy_is_true() {
    // Arrange & Act
    let declaration = RegistryPolicy::source().is_declaration(&item("extern crate alloc;\n"));

    // Assert
    assert!(declaration);
}

// The tests tree is never no_std, so the exception has no reason to exist there.
#[test]
fn is_declaration_of_extern_crate_alloc_under_the_tests_policy_is_false() {
    // Arrange & Act
    let declaration = RegistryPolicy::tests().is_declaration(&item("extern crate alloc;\n"));

    // Assert
    assert!(!declaration);
}

// alloc earns its exception by being unavoidable in a no_std crate. Nothing
// else does, and `extern crate` in 2018-and-later Rust is otherwise a relic.
#[test]
fn is_declaration_of_extern_crate_other_under_the_source_policy_is_false() {
    // Arrange & Act
    let declaration = RegistryPolicy::source().is_declaration(&item("extern crate serde;\n"));

    // Assert
    assert!(!declaration);
}
