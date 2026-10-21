// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Every inherent impl block out of order put back in order: whole line ranges
// from syn's spans, each member with its attributes, doc comments and the
// comment directly above it. Nothing outside the block's braces changes, members
// are one blank line apart -- constants and types none -- and a block the fixer
// cannot move as lines is left exactly as it is.

use stern4rust::ordered_impl_functions_rewriter::OrderedImplFunctionsRewriter;
use stern4rust::rule::Rule;
use stern4rust::rules::source::ordered_impl_functions_rule::OrderedImplFunctionsRule;
use stern4rust::source_file::SourceFile;
use stern4rust::source_rewriter::SourceRewriter;

fn rewrite(contents: &str) -> Option<String> {
    OrderedImplFunctionsRewriter::new().rewrite(&SourceFile::new("src/store.rs", contents))
}

#[test]
fn rewrite_carries_a_members_attributes_doc_and_comment() {
    // Arrange
    let contents = "impl Store {\n    fn b() {}\n\n    // Why.\n    /// What.\n    #[inline]\n    fn a() {}\n}\n";

    // Act
    let rewritten = rewrite(contents).expect("rewritten");

    // Assert
    assert_eq!(
        rewritten,
        "impl Store {\n    // Why.\n    /// What.\n    #[inline]\n    fn a() {}\n\n    fn b() {}\n}\n"
    );
}

#[test]
fn rewrite_fixes_the_block_it_can_and_leaves_the_one_it_cannot() {
    // Arrange
    let contents = "impl Store {\n    fn b() {}\n\n    fn a() {}\n}\n\nimpl Shelf {\n    fn b() {}\n\n    \
                    accessors!();\n\n    fn a() {}\n}\n";

    // Act
    let rewritten = rewrite(contents).expect("rewritten");

    // Assert
    assert_eq!(
        rewritten,
        "impl Store {\n    fn a() {}\n\n    fn b() {}\n}\n\nimpl Shelf {\n    fn b() {}\n\n    \
         accessors!();\n\n    fn a() {}\n}\n"
    );
}

#[test]
fn rewrite_keeps_a_comment_after_the_last_member_at_the_end() {
    // Arrange
    let contents = "impl Store {\n    fn b() {}\n\n    fn a() {}\n\n    // the end\n}\n";

    // Act
    let rewritten = rewrite(contents).expect("rewritten");

    // Assert
    assert_eq!(
        rewritten,
        "impl Store {\n    fn a() {}\n\n    fn b() {}\n\n    // the end\n}\n"
    );
}

#[test]
fn rewrite_keeps_every_line_outside_the_block() {
    // Arrange
    let contents = "use std::fmt;\n\n// A store.\npub struct Store;\n\nimpl   Store {\n    fn b() {}\n\n    fn a() {}\n}\n\n\
                    fn   after() {}\n";

    // Act
    let rewritten = rewrite(contents).expect("rewritten");

    // Assert
    assert_eq!(
        rewritten,
        "use std::fmt;\n\n// A store.\npub struct Store;\n\nimpl   Store {\n    fn a() {}\n\n    fn b() {}\n}\n\n\
         fn   after() {}\n"
    );
}

// The point of a fixer, checked the way the run checks it: by the rule, after.
#[test]
fn rewrite_leaves_a_file_the_rule_finds_nothing_in() {
    // Arrange
    let contents = "pub struct Store;\n\nimpl Store {\n    fn helper() {}\n\n    pub fn len(&self) -> u8 {\n        0\n    }\n\n    \
                    pub fn with_capacity(size: u8) -> Self {\n        Self\n    }\n\n    const LIMIT: u8 = 3;\n\n    \
                    pub fn new() -> Self {\n        Self\n    }\n}\n";

    // Act
    let rewritten = rewrite(contents).expect("rewritten");

    // Assert
    let offences =
        OrderedImplFunctionsRule::new().check(&SourceFile::new("src/store.rs", &rewritten));
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn rewrite_of_a_block_with_a_free_comment_is_none() {
    // Arrange & Act
    let rewritten =
        rewrite("impl Store {\n    fn b() {}\n\n    // -- queries --\n\n    fn a() {}\n}\n");

    // Assert
    assert!(rewritten.is_none());
}

#[test]
fn rewrite_of_a_block_with_two_members_on_one_line_is_none() {
    // Arrange & Act
    let rewritten = rewrite("impl Store {\n    fn b() {} fn a() {}\n}\n");

    // Assert
    assert!(rewritten.is_none());
}

#[test]
fn rewrite_of_a_file_in_order_is_none() {
    // Arrange & Act
    let rewritten = rewrite("impl Store {\n    fn a() {}\n\n    fn b() {}\n}\n");

    // Assert
    assert!(rewritten.is_none());
}

#[test]
fn rewrite_of_a_test_file_is_none() {
    // Arrange
    let file = SourceFile::new(
        "tests/store_tests.rs",
        "impl Store {\n    fn b() {}\n\n    fn a() {}\n}\n",
    );

    // Act
    let rewritten = OrderedImplFunctionsRewriter::new().rewrite(&file);

    // Assert
    assert!(rewritten.is_none());
}

#[test]
fn rewrite_puts_one_blank_line_between_functions_and_none_between_constants() {
    // Arrange
    let contents = "impl Store {\n    fn b() {}\n    fn a() {}\n\n\n    const Y: u8 = 2;\n\n    const X: u8 = 1;\n}\n";

    // Act
    let rewritten = rewrite(contents).expect("rewritten");

    // Assert
    assert_eq!(
        rewritten,
        "impl Store {\n    const Y: u8 = 2;\n    const X: u8 = 1;\n\n    fn a() {}\n\n    fn b() {}\n}\n"
    );
}

#[test]
fn rule_is_the_one_whose_offences_it_repairs() {
    // Arrange & Act
    let rule = OrderedImplFunctionsRewriter::new().rule();

    // Assert
    assert_eq!(rule, "ordered-impl-functions");
}
