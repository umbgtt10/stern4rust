// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The items an inline module holds. Five walkers each carried their own copy of
// this one line, so a rule that descends into `mod a { ... }` reads it from here.

use stern4rust::finding::parsing::inline_module::InlineModule;
use stern4rust::finding::parsing::item_naming::ItemNaming;
use syn::Item;
use syn::ItemMod;
use syn::parse_file;

fn module(source: &str) -> ItemMod {
    match parse_file(source).expect("parses").items.into_iter().next() {
        Some(Item::Mod(module)) => module,
        _ => panic!("expected a module"),
    }
}

// Every item, the inline modules themselves and what they hold, in source order:
// what a walker that wants one kind of item filters.
#[test]
fn flatten_lists_every_item_descending_into_inline_modules() {
    // Arrange
    let items = parse_file("struct A;\nmod inner {\n    struct B;\n}\nmod outside;\n")
        .expect("parses")
        .items;

    // Act
    let flattened = InlineModule::flatten(&items);

    // Assert
    let names: Vec<String> = flattened
        .iter()
        .filter_map(|item| ItemNaming::identifier(item))
        .collect();
    assert_eq!(names, ["A", "inner", "B", "outside"]);
}

// `mod inner;` declares a module whose items live in another file, which the
// walker reaches on its own when it reads that file.
#[test]
fn items_of_a_module_declaration_returns_none() {
    // Arrange
    let module = module("mod inner;");

    // Act
    let items = InlineModule::items(&module);

    // Assert
    assert!(items.is_none());
}

#[test]
fn items_of_an_inline_module_returns_what_it_holds() {
    // Arrange
    let module = module("mod inner { fn run() {} struct Probe; }");

    // Act
    let items = InlineModule::items(&module);

    // Assert
    assert_eq!(items.map(<[Item]>::len), Some(2));
}
