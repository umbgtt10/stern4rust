// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A file's module-level constants declared after its first item that is neither
// an import nor a constant, and where they would go. Imports and `mod`
// declarations are not items here; an inline module is one, and what it holds
// is not looked at.

use stern4rust::finding::model::layout::fix_refusal::FixRefusal;
use stern4rust::finding::parsing::constant_layout_parser::ConstantLayoutParser;
use stern4rust::source_file::SourceFile;

fn names(contents: &str) -> Vec<String> {
    ConstantLayoutParser::parse(&SourceFile::new("src/widget.rs", contents))
        .map(|layout| {
            layout
                .misplaced()
                .iter()
                .map(|constant| constant.name().to_string())
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn parse_a_constant_after_a_module_declaration_finds_nothing() {
    // Arrange
    let file = SourceFile::new("src/widget.rs", "mod inner;\n\nconst LIMIT: u8 = 3;\n");

    // Act
    let layout = ConstantLayoutParser::parse(&file);

    // Assert
    assert!(layout.is_none());
}

#[test]
fn parse_a_constant_after_a_struct_finds_it_and_the_struct() {
    // Arrange
    let file = SourceFile::new(
        "src/widget.rs",
        "use std::fmt;\n\npub struct Widget;\n\nconst LIMIT: u8 = 3;\n",
    );

    // Act
    let layout = ConstantLayoutParser::parse(&file).expect("a misplaced constant");

    // Assert
    assert_eq!(layout.leader(), "Widget");
    assert_eq!(layout.misplaced().len(), 1);
    assert_eq!(layout.misplaced()[0].name(), "LIMIT");
    assert_eq!(layout.misplaced()[0].line(), 5);
}

#[test]
fn parse_a_constant_after_an_impl_names_the_impl() {
    // Arrange
    let file = SourceFile::new(
        "src/widget.rs",
        "#[cfg(test)]\nimpl Widget {}\n\nconst LIMIT: u8 = 3;\n",
    );

    // Act
    let layout = ConstantLayoutParser::parse(&file).expect("a misplaced constant");

    // Assert
    assert_eq!(layout.leader(), "impl Widget");
}

// A comment directly above a constant introduces it, and an attribute is part of
// it, so both move with it.
#[test]
fn parse_a_constant_below_a_comment_and_an_attribute_spans_all_three() {
    // Arrange
    let file = SourceFile::new(
        "src/widget.rs",
        "pub struct Widget;\n\n// The most a widget holds.\n#[allow(dead_code)]\nconst LIMIT: u8 = 3;\n",
    );

    // Act
    let layout = ConstantLayoutParser::parse(&file).expect("a misplaced constant");

    // Assert
    let constant = &layout.misplaced()[0];
    assert_eq!(constant.line(), 5);
    assert_eq!(constant.first_line(), 3);
    assert_eq!(constant.last_line(), 5);
}

#[test]
fn parse_a_constant_on_the_line_of_the_item_it_follows_refuses_the_fix() {
    // Arrange
    let file = SourceFile::new("src/widget.rs", "pub struct Widget; const LIMIT: u8 = 3;\n");

    // Act
    let layout = ConstantLayoutParser::parse(&file).expect("a misplaced constant");

    // Assert
    assert_eq!(layout.refusal(), Some(FixRefusal::SharedLine));
}

#[test]
fn parse_a_constant_on_the_line_of_the_next_item_refuses_the_fix() {
    // Arrange
    let file = SourceFile::new(
        "src/widget.rs",
        "pub struct Widget;\n\nconst LIMIT: u8 = 3; fn helper() {}\n",
    );

    // Act
    let layout = ConstantLayoutParser::parse(&file).expect("a misplaced constant");

    // Assert
    assert_eq!(layout.refusal(), Some(FixRefusal::SharedLine));
}

#[test]
fn parse_a_file_of_only_constants_finds_nothing() {
    // Arrange
    let file = SourceFile::new(
        "src/limits.rs",
        "pub const LOW: u8 = 1;\n\npub const HIGH: u8 = 9;\n",
    );

    // Act
    let layout = ConstantLayoutParser::parse(&file);

    // Assert
    assert!(layout.is_none());
}

#[test]
fn parse_a_file_that_does_not_parse_finds_nothing() {
    // Arrange
    let file = SourceFile::new(
        "src/broken.rs",
        "pub struct Widget\n\nconst LIMIT: u8 = 3;\n",
    );

    // Act
    let layout = ConstantLayoutParser::parse(&file);

    // Assert
    assert!(layout.is_none());
}

// Textual order decides which `macro_rules!` a later item can call, so moving a
// constant above one can stop it compiling.
#[test]
fn parse_a_file_with_a_macro_rules_refuses_the_fix() {
    // Arrange
    let file = SourceFile::new(
        "src/widget.rs",
        "macro_rules! limit {\n    () => {\n        3\n    };\n}\n\nconst LIMIT: u8 = limit!();\n",
    );

    // Act
    let layout = ConstantLayoutParser::parse(&file).expect("a misplaced constant");

    // Assert
    assert_eq!(layout.leader(), "macro_rules! limit");
    assert_eq!(layout.refusal(), Some(FixRefusal::MacroRules));
}

#[test]
fn parse_a_first_item_on_the_line_of_an_import_refuses_the_fix() {
    // Arrange
    let file = SourceFile::new(
        "src/widget.rs",
        "use std::fmt; pub struct Widget;\n\nconst LIMIT: u8 = 3;\n",
    );

    // Act
    let layout = ConstantLayoutParser::parse(&file).expect("a misplaced constant");

    // Assert
    assert_eq!(layout.refusal(), Some(FixRefusal::SharedLine));
}

// With nothing above the first item, a comment running from the top of the file
// into it could be the header or the item's own; neither may be split off.
#[test]
fn parse_a_first_item_the_opening_comment_runs_into_refuses_the_fix() {
    // Arrange
    let file = SourceFile::new(
        "src/widget.rs",
        "// Copyright\npub struct Widget;\n\nconst LIMIT: u8 = 3;\n",
    );

    // Act
    let layout = ConstantLayoutParser::parse(&file).expect("a misplaced constant");

    // Assert
    assert_eq!(layout.refusal(), Some(FixRefusal::OpeningComment));
}

#[test]
fn parse_after_a_constant_in_place_inserts_after_it() {
    // Arrange
    let file = SourceFile::new(
        "src/widget.rs",
        "use std::fmt;\n\nconst LOW: u8 = 1;\n\npub struct Widget;\n\nconst HIGH: u8 = 9;\n",
    );

    // Act
    let layout = ConstantLayoutParser::parse(&file).expect("a misplaced constant");

    // Assert
    assert_eq!(layout.insertion_line(), 4);
    assert!(layout.follows_constant());
}

#[test]
fn parse_after_the_imports_inserts_after_the_last_of_them() {
    // Arrange
    let file = SourceFile::new(
        "src/widget.rs",
        "use std::fmt;\nuse std::io;\n\npub struct Widget;\n\nconst LIMIT: u8 = 3;\n",
    );

    // Act
    let layout = ConstantLayoutParser::parse(&file).expect("a misplaced constant");

    // Assert
    assert_eq!(layout.insertion_line(), 3);
    assert!(!layout.follows_constant());
    assert_eq!(layout.refusal(), None);
}

#[test]
fn parse_an_inline_module_is_an_item_constants_follow() {
    // Arrange & Act
    let found = names("mod inner {}\n\nconst LIMIT: u8 = 3;\n");

    // Assert
    assert_eq!(found, vec!["LIMIT"]);
}

#[test]
fn parse_constants_inside_an_inline_module_are_not_looked_at() {
    // Arrange
    let file = SourceFile::new(
        "src/widget.rs",
        "pub struct Widget;\n\nmod inner {\n    pub struct Gadget;\n\n    const LIMIT: u8 = 3;\n}\n",
    );

    // Act
    let layout = ConstantLayoutParser::parse(&file);

    // Assert
    assert!(layout.is_none());
}

#[test]
fn parse_constants_straight_after_the_imports_finds_nothing() {
    // Arrange
    let file = SourceFile::new(
        "src/widget.rs",
        "use std::fmt;\n\nconst LIMIT: u8 = 3;\n\npub struct Widget;\n",
    );

    // Act
    let layout = ConstantLayoutParser::parse(&file);

    // Assert
    assert!(layout.is_none());
}

#[test]
fn parse_every_constant_after_the_first_item_is_found_in_order() {
    // Arrange & Act
    let found = names(
        "pub struct Widget;\n\nconst LOW: u8 = 1;\n\nfn helper() {}\n\nstatic HIGH: u8 = 9;\n",
    );

    // Assert
    assert_eq!(found, vec!["LOW", "HIGH"]);
}

#[test]
fn parse_with_nothing_above_the_first_item_inserts_before_its_block() {
    // Arrange
    let file = SourceFile::new(
        "src/widget.rs",
        "// Copyright\n\n// A widget.\npub struct Widget;\n\nconst LIMIT: u8 = 3;\n",
    );

    // Act
    let layout = ConstantLayoutParser::parse(&file).expect("a misplaced constant");

    // Assert
    assert_eq!(layout.insertion_line(), 3);
    assert!(!layout.follows_constant());
    assert_eq!(layout.refusal(), None);
}
