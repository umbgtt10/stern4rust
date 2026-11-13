// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The body of a macro outside the standard family, read as code where it is:
// a list of expressions, else the brace groups that parse as blocks; never the
// body of a macro whose body is not code the file wrote.

use quote::ToTokens;
use stern4rust::finding::parsing::shapes::macro_body::MacroBody;
use syn::ExprMacro;
use syn::Macro;
use syn::parse_str;

fn blocks(text: &str) -> Vec<String> {
    let node = macro_of(text);
    MacroBody::of(&node)
        .expect("another macro")
        .blocks()
        .iter()
        .map(|block| block.to_token_stream().to_string())
        .collect()
}

fn macro_of(text: &str) -> Macro {
    parse_str::<ExprMacro>(text).expect("a macro").mac
}

#[test]
fn blocks_of_a_brace_group_that_is_not_a_block_are_the_blocks_inside_it() {
    // Arrange & Act
    let found = blocks("json!({ \"a\": { b } })");

    // Assert
    assert_eq!(found, ["{ b }"]);
}

#[test]
fn blocks_of_a_group_of_another_delimiter_are_the_blocks_inside_it() {
    // Arrange & Act
    let found = blocks("wrapped!((x, { y }))");

    // Assert
    assert_eq!(found, ["{ y }"]);
}

// A proptest!'s test is an item no parser takes -- `a in 0..1` -- but its
// body is a block.
#[test]
fn blocks_of_a_proptest_body_are_each_test_s_body() {
    // Arrange & Act
    let found = blocks(
        "proptest! { #[test] fn t(a in 0..1) { let _ = a; } #[test] fn u(b in 0..1) { b; } }",
    );

    // Assert
    assert_eq!(found, ["{ let _ = a ; }", "{ b ; }"]);
}

#[test]
fn expressions_of_a_body_of_expressions_returns_each_of_them() {
    // Arrange
    let node = macro_of("prop_assert_eq!(first(), second)");

    // Act
    let found: Option<Vec<String>> = MacroBody::of(&node)
        .expect("another macro")
        .expressions()
        .map(|expressions| {
            expressions
                .iter()
                .map(|expression| expression.to_token_stream().to_string())
                .collect()
        });

    // Assert
    assert_eq!(
        found,
        Some(vec![String::from("first ()"), String::from("second")])
    );
}

#[test]
fn expressions_of_a_body_that_is_no_list_of_expressions_returns_none() {
    // Arrange
    let node = macro_of("select! { got = next() => { handle(got); } }");

    // Act
    let found = MacroBody::of(&node)
        .expect("another macro")
        .expressions()
        .map(|expressions| expressions.len());

    // Assert
    assert_eq!(found, None);
}

#[test]
fn of_a_macro_of_the_standard_family_returns_none() {
    // Arrange
    let node = macro_of("assert_eq!(a, b)");

    // Act
    let read = MacroBody::of(&node).is_some();

    // Assert
    assert!(!read);
}

// quote! spells a generated path in full on purpose, and the compiler's
// built-ins take tokens or text: none of their bodies is code the file wrote.
#[test]
fn of_a_macro_whose_body_is_not_code_returns_none() {
    // Arrange
    let nodes = [
        "quote!(::std::env::args())",
        "quote_spanned!(span => ::std::env::args())",
        "parse_quote!(::std::env::args())",
        "stringify!(std::env::args())",
        "include_str!(\"a.txt\")",
        "concat!(\"a\", \"b\")",
    ]
    .map(macro_of);

    // Act
    let read: Vec<bool> = nodes
        .iter()
        .map(|node| MacroBody::of(node).is_some())
        .collect();

    // Assert
    assert_eq!(read, [false; 6]);
}

#[test]
fn of_another_macro_returns_it() {
    // Arrange
    let node = macro_of("proptest::prop_assert_eq!(a, b)");

    // Act
    let read = MacroBody::of(&node).is_some();

    // Assert
    assert!(read);
}
