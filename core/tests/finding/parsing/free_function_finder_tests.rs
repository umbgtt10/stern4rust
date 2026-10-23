// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The functions of a file that belong to no impl block.
//
// Module level, inside an inline module, and nested in another function's body
// all count: a function declared inside a method is still a free function, just
// a harder one to see. Associated functions and trait methods do not count.
//
// Left out are the functions something other than the author requires to be
// free -- an entry point a framework attribute names, an exported or extern
// symbol, `main` in an entry file -- and anything under `#[cfg(test)]`, which
// is test-free-source's to report.

use stern4rust::finding::model::free_function::FreeFunction;
use stern4rust::finding::parsing::free_function_finder::FreeFunctionFinder;
use syn::Item;
use syn::parse_file;

fn items(source: &str) -> Vec<Item> {
    parse_file(source).expect("parses").items
}

fn names(found: Vec<FreeFunction>) -> Vec<String> {
    found
        .into_iter()
        .map(|function| function.name().to_string())
        .collect()
}

#[test]
fn among_descends_into_an_inline_module() {
    // Arrange
    let items = items("mod detail {\n    pub fn helper() {}\n}\n");

    // Act
    let found = FreeFunctionFinder::among(&items, false);

    // Assert
    assert_eq!(names(found), ["helper"]);
}

#[test]
fn among_finds_a_function_nested_in_a_method_body() {
    // Arrange
    let items = items(
        "impl Widget {\n    fn draw(&self) {\n        fn inner() {}\n        inner();\n    }\n}\n",
    );

    // Act
    let found = FreeFunctionFinder::among(&items, false);

    // Assert
    assert_eq!(names(found), ["inner"]);
}

#[test]
fn among_finds_module_level_and_nested_functions_in_source_order() {
    // Arrange
    let items = items("fn outer() {\n    fn inner() {}\n}\n\nfn last() {}\n");

    // Act
    let found = FreeFunctionFinder::among(&items, false);

    // Assert
    assert_eq!(names(found), ["outer", "inner", "last"]);
}

#[test]
fn among_keeps_main_outside_an_entry_file() {
    // Arrange
    let items = items("fn main() {}\n");

    // Act
    let found = FreeFunctionFinder::among(&items, false);

    // Assert
    assert_eq!(names(found), ["main"]);
}

#[test]
fn among_leaves_out_an_extern_function() {
    // Arrange
    let items = items("pub extern \"C\" fn callback() {}\n");

    // Act
    let found = FreeFunctionFinder::among(&items, false);

    // Assert
    assert!(found.is_empty());
}

#[test]
fn among_leaves_out_associated_functions_and_trait_methods() {
    // Arrange
    let items = items(
        "impl Widget {\n    pub fn new() -> Self {\n        Self\n    }\n}\n\npub trait Draw {\n    fn draw(&self);\n}\n",
    );

    // Act
    let found = FreeFunctionFinder::among(&items, false);

    // Assert
    assert!(found.is_empty());
}

#[test]
fn among_leaves_out_cfg_test_items() {
    // Arrange
    let items =
        items("#[cfg(test)]\nfn helper() {}\n\n#[cfg(test)]\nmod tests {\n    fn case() {}\n}\n");

    // Act
    let found = FreeFunctionFinder::among(&items, false);

    // Assert
    assert!(found.is_empty());
}

#[test]
fn among_leaves_out_functions_a_framework_requires_free() {
    // Arrange
    let items = items(
        "#[embassy_executor::task]\nasync fn run_task() {}\n\n#[panic_handler]\nfn panic(_info: &PanicInfo) -> ! {\n    loop {}\n}\n\n#[cortex_m_rt::exception]\nunsafe fn HardFault(_frame: &ExceptionFrame) -> ! {\n    loop {}\n}\n\n#[unsafe(no_mangle)]\npub fn exported() {}\n\n#[proc_macro_derive(Widget)]\npub fn derive(input: TokenStream) -> TokenStream {\n    input\n}\n",
    );

    // Act
    let found = FreeFunctionFinder::among(&items, false);

    // Assert
    assert!(found.is_empty(), "got {:?}", names(found));
}

#[test]
fn among_leaves_out_main_in_an_entry_file() {
    // Arrange
    let items = items("fn main() {}\n\nfn helper() {}\n");

    // Act
    let found = FreeFunctionFinder::among(&items, true);

    // Assert
    assert_eq!(names(found), ["helper"]);
}

#[test]
fn among_records_the_line_and_the_names_a_function_uses() {
    // Arrange
    let items = items("pub struct Widget;\n\npub fn parse() -> Option<Widget> {\n    None\n}\n");

    // Act
    let found = FreeFunctionFinder::among(&items, false);

    // Assert
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].line(), 3);
    assert!(found[0].names("Widget"));
    assert!(!found[0].names("Gadget"));
}
