// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The identifiers a stretch of source uses, however deeply its groups nest.
// Literals are not identifiers, so `"Widget"` in a string names nothing.

use proc_macro2::TokenStream;
use quote::ToTokens;
use stern4rust::finding::parsing::identifiers::Identifiers;
use syn::ItemFn;
use syn::parse_str;

fn tokens(source: &str) -> TokenStream {
    parse_str::<ItemFn>(source)
        .expect("parses")
        .to_token_stream()
}

#[test]
fn of_a_function_collects_identifiers_inside_nested_groups() {
    // Arrange
    let source = tokens(
        "fn draw() -> u8 {\n    if ready() {\n        border(Widget)\n    } else {\n        0\n    }\n}\n",
    );

    // Act
    let found = Identifiers::of(source);

    // Assert
    assert!(
        ["draw", "u8", "ready", "border", "Widget"]
            .iter()
            .all(|name| found.contains(*name)),
        "got {found:?}"
    );
}

#[test]
fn of_a_function_leaves_out_what_its_literals_spell() {
    // Arrange
    let source = tokens("fn label() -> &'static str {\n    \"Widget\"\n}\n");

    // Act
    let found = Identifiers::of(source);

    // Assert
    assert!(!found.contains("Widget"), "got {found:?}");
}
