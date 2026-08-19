// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A test file put back into the order the structure rule asks for.
//
// The property that makes a rewriter safe where a text-munging script was not:
// it never reads the text. syn says where each item starts and ends, and whole
// line ranges are moved without being inspected -- so a string literal holding
// something that looks like Rust travels along like any other line.

use stern4rust::source_file::SourceFile;
use stern4rust::test_file_rewriter::TestFileRewriter;

fn rewrite(contents: &str) -> Option<String> {
    TestFileRewriter::rewrite(&SourceFile::new("tests/a_tests.rs", contents))
}

#[test]
fn rewrite_carries_a_comment_with_the_item_it_introduces() {
    // Arrange
    let contents =
        "// about beta\n#[test]\nfn beta() {}\n\n// about alpha\n#[test]\nfn alpha() {}\n";

    // Act
    let rewritten = rewrite(contents).expect("reordered");

    // Assert
    assert_eq!(
        rewritten,
        "// about alpha\n#[test]\nfn alpha() {}\n\n// about beta\n#[test]\nfn beta() {}\n"
    );
}

// The exact shape a hand-rolled script got wrong: a string literal holding Rust
// must be moved, never parsed.
#[test]
fn rewrite_does_not_look_inside_a_string_literal() {
    // Arrange
    let contents =
        "#[test]\nfn beta() {\n    let _ = \"fn alpha() {}\";\n}\n\n#[test]\nfn alpha() {}\n";

    // Act
    let rewritten = rewrite(contents).expect("reordered");

    // Assert
    assert!(rewritten.contains("let _ = \"fn alpha() {}\";"));
    assert!(rewritten.find("fn alpha() {}\n").unwrap() < rewritten.find("fn beta()").unwrap());
}

// Losing it would be the rewriter destroying content it was asked to tidy.
#[test]
fn rewrite_keeps_a_trailing_comment() {
    // Arrange
    let contents = "#[test]\nfn beta() {}\n\n#[test]\nfn alpha() {}\n\n// trailing note\n";

    // Act
    let rewritten = rewrite(contents).expect("reordered");

    // Assert
    assert!(rewritten.ends_with("// trailing note\n"));
}

#[test]
fn rewrite_keeps_the_header_and_preamble_where_they_are() {
    // Arrange
    let contents = "// Copyright\n\n// what this file is about\n\n#[test]\nfn beta() {}\n\n#[test]\nfn alpha() {}\n";

    // Act
    let rewritten = rewrite(contents).expect("reordered");

    // Assert
    assert!(rewritten.starts_with("// Copyright\n\n// what this file is about\n\n"));
}

// rustfmt owns import order and disagrees with a plain alphabet on two shapes,
// which is why test-file-structure stands down on those pairs. A fixer that
// sorted imports anyway would write an order `cargo fmt` undoes on the next
// run -- the unsatisfiable loop that stand-down exists to prevent.
#[test]
fn rewrite_leaves_the_order_of_imports_to_rustfmt() {
    // Arrange
    let contents = "use serde_json::Value;\nuse serde_json::from_str;\n\n#[test]\nfn beta() {}\n\n#[test]\nfn alpha() {}\n";

    // Act
    let rewritten = rewrite(contents).expect("reordered");

    // Assert
    assert!(
        rewritten.starts_with("use serde_json::Value;\nuse serde_json::from_str;\n"),
        "got {rewritten:?}"
    );
}

#[test]
fn rewrite_of_a_correct_file_is_none() {
    // Arrange
    let contents = "use std::fmt;\n\n#[test]\nfn alpha() {}\n\n#[test]\nfn beta() {}\n";

    // Act
    let rewritten = rewrite(contents);

    // Assert
    assert!(rewritten.is_none(), "got {rewritten:?}");
}

#[test]
fn rewrite_of_a_file_that_does_not_parse_is_none() {
    // Arrange & Act
    let rewritten = rewrite("fn broken( {\n");

    // Assert
    assert!(rewritten.is_none());
}

// The bug the first version of this shipped with: it rewrote thirty src/ files,
// reordering their imports into one alphabetical block and destroying a
// grouping convention no rule checks -- so no rule would have restored it, and
// the run went green over a tree nobody had reviewed. A fixer must never edit a
// file no rule governs.
#[test]
fn rewrite_of_a_source_file_is_none() {
    // Arrange
    let contents = "use std::fmt;\n\nuse crate::thing::Thing;\n\npub struct A;\n";

    // Act
    let rewritten = TestFileRewriter::rewrite(&SourceFile::new("src/a.rs", contents));

    // Assert
    assert!(rewritten.is_none(), "got {rewritten:?}");
}

#[test]
fn rewrite_of_a_tests_registry_is_none() {
    // Arrange
    let contents = "pub mod beta_tests;\npub mod alpha_tests;\n";

    // Act
    let rewritten = TestFileRewriter::rewrite(&SourceFile::new("tests/all_tests.rs", contents));

    // Assert
    assert!(rewritten.is_none(), "got {rewritten:?}");
}

#[test]
fn rewrite_orders_the_sections_before_their_members() {
    // Arrange
    let contents =
        "#[test]\nfn zeta() {}\n\nfn helper() {}\n\nconst LIMIT: usize = 1;\n\nuse std::fmt;\n";

    // Act
    let rewritten = rewrite(contents).expect("reordered");

    // Assert
    assert_eq!(
        rewritten,
        "use std::fmt;\n\nconst LIMIT: usize = 1;\n\nfn helper() {}\n\n#[test]\nfn zeta() {}\n"
    );
}

// Imports are the one group written without gaps.
#[test]
fn rewrite_puts_no_blank_line_between_imports() {
    // Arrange
    let contents = "use std::fmt;\n\nuse std::env;\n\n#[test]\nfn alpha() {}\n";

    // Act
    let rewritten = rewrite(contents).expect("reordered");

    // Assert
    assert!(
        rewritten.starts_with("use std::fmt;\nuse std::env;\n\n"),
        "got {rewritten:?}"
    );
}

#[test]
fn rewrite_sorts_tests_case_insensitively() {
    // Arrange
    let contents = "#[test]\nfn beta() {}\n\n#[test]\nfn Alpha() {}\n";

    // Act
    let rewritten = rewrite(contents).expect("reordered");

    // Assert
    assert!(rewritten.find("fn Alpha").unwrap() < rewritten.find("fn beta").unwrap());
}
