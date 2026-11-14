// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The identifier an item declares, and the source line to fall back on when it
// declares none.
//
// Three parsers each carried both, with the same arms and the same fallback.
// What they legitimately differ on is the *wording* around the name -- "the
// constant `X`", "constant `X`", or bare `X` -- so that stays with each of them
// and only the shared half moved here.

use stern4rust::finding::parsing::item_naming::ItemNaming;
use stern4rust::source_file::SourceFile;
use syn::Item;
use syn::ItemUse;
use syn::Type;
use syn::parse_file;

fn first_item(source: &str) -> Item {
    parse_file(source)
        .expect("parses")
        .items
        .into_iter()
        .next()
        .expect("one item")
}

fn import(source: &str) -> ItemUse {
    match first_item(source) {
        Item::Use(inner) => inner,
        _ => panic!("expected an import"),
    }
}

fn self_type(source: &str) -> Type {
    match first_item(source) {
        Item::Impl(inner) => *inner.self_ty,
        _ => panic!("expected an impl block"),
    }
}

// Past its attributes, doc comments among them: the line a reader would
// search the file for.
#[test]
fn code_line_of_an_attributed_item_is_its_first_line_of_code() {
    // Arrange
    let source = "#[cfg(test)]\n/// What it stands in for.\nimpl Thing {\n    fn f() {}\n}\n";
    let file = SourceFile::new("src/lib.rs", source);

    // Act
    let line = ItemNaming::code_line(&file, &first_item(source));

    // Assert
    assert_eq!(line, "impl Thing {");
}

#[test]
fn code_line_of_an_item_sharing_a_line_with_its_attribute_is_that_line() {
    // Arrange
    let source = "#[cfg(test)] use std::fmt;\n";
    let file = SourceFile::new("src/lib.rs", source);

    // Act
    let line = ItemNaming::code_line(&file, &first_item(source));

    // Assert
    assert_eq!(line, "#[cfg(test)] use std::fmt;");
}

#[test]
fn code_line_of_an_item_without_attributes_is_its_own_line() {
    // Arrange
    let source = "\n    extern crate alloc;\n";
    let file = SourceFile::new("src/lib.rs", source);

    // Act
    let line = ItemNaming::code_line(&file, &first_item(source));

    // Assert
    assert_eq!(line, "extern crate alloc;");
}

#[test]
fn identifier_of_a_constant_returns_its_name() {
    // Arrange
    let item = first_item("const LIMIT: usize = 1;");

    // Act
    let identifier = ItemNaming::identifier(&item);

    // Assert
    assert_eq!(identifier.as_deref(), Some("LIMIT"));
}

#[test]
fn identifier_of_a_function_returns_its_name() {
    // Arrange
    let item = first_item("fn run() {}");

    // Act
    let identifier = ItemNaming::identifier(&item);

    // Assert
    assert_eq!(identifier.as_deref(), Some("run"));
}

#[test]
fn identifier_of_a_struct_returns_its_name() {
    // Arrange
    let item = first_item("struct Widget;");

    // Act
    let identifier = ItemNaming::identifier(&item);

    // Assert
    assert_eq!(identifier.as_deref(), Some("Widget"));
}

#[test]
fn identifier_of_a_type_alias_returns_its_name() {
    // Arrange
    let item = first_item("type Pair = (usize, usize);");

    // Act
    let identifier = ItemNaming::identifier(&item);

    // Assert
    assert_eq!(identifier.as_deref(), Some("Pair"));
}

// An impl block declares no name of its own; it belongs to the type it
// implements, and each caller decides what to say about that.
#[test]
fn identifier_of_an_enum_a_trait_and_a_union_returns_each_name() {
    // Arrange
    let items = [
        first_item("enum Mode {\n    Fast,\n}\n"),
        first_item("trait Port {}\n"),
        first_item("union Bits {\n    raw: u8,\n}\n"),
    ];

    // Act
    let names: Vec<Option<String>> = items.iter().map(ItemNaming::identifier).collect();

    // Assert
    assert_eq!(
        names,
        [
            Some(String::from("Mode")),
            Some(String::from("Port")),
            Some(String::from("Bits")),
        ]
    );
}

#[test]
fn identifier_of_an_impl_block_is_none() {
    // Arrange
    let item = first_item("impl Widget {}");

    // Act
    let identifier = ItemNaming::identifier(&item);

    // Assert
    assert!(identifier.is_none());
}

#[test]
fn identifier_of_an_import_is_none() {
    // Arrange
    let item = first_item("use std::fmt;");

    // Act
    let identifier = ItemNaming::identifier(&item);

    // Assert
    assert!(identifier.is_none());
}

#[test]
fn implemented_type_of_a_generic_type_is_its_name_without_arguments() {
    // Arrange
    let ty = self_type("impl<T> Store<T> {}");

    // Act
    let name = ItemNaming::implemented_type(&ty);

    // Assert
    assert_eq!(name, "Store");
}

#[test]
fn implemented_type_of_a_path_is_its_last_segment() {
    // Arrange
    let ty = self_type("impl crate::store::Store {}");

    // Act
    let name = ItemNaming::implemented_type(&ty);

    // Assert
    assert_eq!(name, "Store");
}

// Not a name a reader could look for, so none is made up.
#[test]
fn implemented_type_of_a_reference_is_empty() {
    // Arrange
    let ty = self_type("impl Shape for &Widget {}");

    // Act
    let name = ItemNaming::implemented_type(&ty);

    // Assert
    assert!(name.is_empty());
}

// An item's span opens at its first attribute, so the line it starts on is the
// attribute's; the import is the line its `use` keyword is on.
#[test]
fn import_line_of_an_attributed_import_is_its_use_line() {
    // Arrange
    let source = "#[cfg(unix)]\n#[allow(unused_imports)]\n    use std::os::unix::ffi::OsStrExt;\n";
    let file = SourceFile::new("tests/a_tests.rs", source);

    // Act
    let line = ItemNaming::import_line(&file, &import(source));

    // Assert
    assert_eq!(line, "use std::os::unix::ffi::OsStrExt;");
}

// rustfmt moves the attribute onto a line of its own; until it has, the line
// the `use` is on is the whole of it.
#[test]
fn import_line_of_an_import_sharing_a_line_with_its_attribute_is_that_line() {
    // Arrange
    let source = "#[cfg(unix)] pub use alpha::One;\n";
    let file = SourceFile::new("tests/a_tests.rs", source);

    // Act
    let line = ItemNaming::import_line(&file, &import(source));

    // Assert
    assert_eq!(line, "#[cfg(unix)] pub use alpha::One;");
}

#[test]
fn source_line_past_the_end_of_the_file_is_empty() {
    // Arrange
    let file = SourceFile::new("src/a.rs", "struct A;\n");

    // Act
    let line = ItemNaming::source_line(&file, 99);

    // Assert
    assert!(line.is_empty());
}

// The fallback is what a reader would point at, so it is the line as written.
#[test]
fn source_line_returns_the_line_trimmed() {
    // Arrange
    let file = SourceFile::new("src/a.rs", "struct A;\n    use std::fmt;\n");

    // Act
    let line = ItemNaming::source_line(&file, 2);

    // Assert
    assert_eq!(line, "use std::fmt;");
}
