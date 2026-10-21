// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A source file's misplaced constants moved up to the end of its constants
// group, straight after the imports when there is none. Whole line ranges from
// syn's spans, each with its attributes and the comment directly above it;
// every other line keeps its place and its bytes. A file it cannot repair
// without guessing is left alone, and keeps its offence.

use stern4rust::leading_constants_rewriter::LeadingConstantsRewriter;
use stern4rust::rule::Rule;
use stern4rust::rules::source::leading_constants_rule::LeadingConstantsRule;
use stern4rust::source_file::SourceFile;
use stern4rust::source_rewriter::SourceRewriter;

const HEADER: &str = "// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>\n\
                      // Licensed under the MIT License\n\
                      // SPDX-License-Identifier: MIT\n";

fn rewrite(contents: &str) -> Option<String> {
    LeadingConstantsRewriter::new().rewrite(&SourceFile::new("src/widget.rs", contents))
}

#[test]
fn rewrite_appends_to_the_constants_already_in_place() {
    // Arrange
    let contents =
        "use std::fmt;\n\nconst LOW: u8 = 1;\n\npub struct Widget;\n\nconst HIGH: u8 = 9;\n";

    // Act
    let rewritten = rewrite(contents).expect("rewritten");

    // Assert
    assert_eq!(
        rewritten,
        "use std::fmt;\n\nconst LOW: u8 = 1;\nconst HIGH: u8 = 9;\n\npub struct Widget;\n"
    );
}

#[test]
fn rewrite_carries_a_constants_attributes_and_comment() {
    // Arrange
    let contents = "pub struct Widget;\n\n// The most a widget holds.\n#[allow(dead_code)]\nconst LIMIT: u8 = 3;\n";

    // Act
    let rewritten = rewrite(contents).expect("rewritten");

    // Assert
    assert_eq!(
        rewritten,
        "// The most a widget holds.\n#[allow(dead_code)]\nconst LIMIT: u8 = 3;\n\npub struct Widget;\n"
    );
}

#[test]
fn rewrite_keeps_a_trailing_comment() {
    // Arrange
    let contents = "pub struct Widget;\n\nconst LIMIT: u8 = 3;\n\n// trailing note\n";

    // Act
    let rewritten = rewrite(contents).expect("rewritten");

    // Assert
    assert_eq!(
        rewritten,
        "const LIMIT: u8 = 3;\n\npub struct Widget;\n\n// trailing note\n"
    );
}

#[test]
fn rewrite_keeps_every_other_line_as_written() {
    // Arrange
    let contents = "use std::fmt;\n\npub struct Widget {\n    size: u8,\n}\n\n\
                    const LIMIT: u8 = 3;\n\n// Widgets render.\nimpl   fmt::Display for Widget {\n    \
                    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { Ok(()) }\n}\n";

    // Act
    let rewritten = rewrite(contents).expect("rewritten");

    // Assert
    assert_eq!(
        rewritten,
        "use std::fmt;\n\nconst LIMIT: u8 = 3;\n\npub struct Widget {\n    size: u8,\n}\n\n\
         // Widgets render.\nimpl   fmt::Display for Widget {\n    \
         fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { Ok(()) }\n}\n"
    );
}

#[test]
fn rewrite_keeps_the_header_above_the_constants() {
    // Arrange
    let contents = format!("{HEADER}\npub struct Widget;\n\nconst LIMIT: u8 = 3;\n");

    // Act
    let rewritten = rewrite(&contents).expect("rewritten");

    // Assert
    assert_eq!(
        rewritten,
        format!("{HEADER}\nconst LIMIT: u8 = 3;\n\npub struct Widget;\n")
    );
}

// The point of a fixer, checked the way the run checks it: by the rule, after.
#[test]
fn rewrite_leaves_a_file_the_rule_finds_nothing_in() {
    // Arrange
    let contents = format!(
        "{HEADER}\nuse std::fmt;\n\npub struct Widget;\n\nconst LOW: u8 = 1;\n\nfn helper() {{}}\n\n\
         static HIGH: u8 = 9;\n"
    );

    // Act
    let rewritten = rewrite(&contents).expect("rewritten");

    // Assert
    let offences = LeadingConstantsRule::new().check(&SourceFile::new("src/widget.rs", &rewritten));
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn rewrite_leaves_one_blank_line_where_a_constant_was() {
    // Arrange
    let contents =
        "use std::fmt;\n\npub struct Widget;\n\nconst LIMIT: u8 = 3;\n\nimpl Widget {}\n";

    // Act
    let rewritten = rewrite(contents).expect("rewritten");

    // Assert
    assert_eq!(
        rewritten,
        "use std::fmt;\n\nconst LIMIT: u8 = 3;\n\npub struct Widget;\n\nimpl Widget {}\n"
    );
}

// A multi-line string is moved as lines, never read.
#[test]
fn rewrite_moves_a_constant_holding_rust_in_a_string_whole() {
    // Arrange
    let contents = "pub struct Widget;\n\nconst SOURCE: &str = \"\nfn widget() {}\n\";\n";

    // Act
    let rewritten = rewrite(contents).expect("rewritten");

    // Assert
    assert_eq!(
        rewritten,
        "const SOURCE: &str = \"\nfn widget() {}\n\";\n\npub struct Widget;\n"
    );
}

#[test]
fn rewrite_of_a_constant_sharing_a_line_is_none() {
    // Arrange & Act
    let rewritten = rewrite("pub struct Widget; const LIMIT: u8 = 3;\n");

    // Assert
    assert!(rewritten.is_none());
}

#[test]
fn rewrite_of_a_correct_file_is_none() {
    // Arrange & Act
    let rewritten = rewrite("use std::fmt;\n\nconst LIMIT: u8 = 3;\n\npub struct Widget;\n");

    // Assert
    assert!(rewritten.is_none());
}

#[test]
fn rewrite_of_a_file_that_does_not_parse_is_none() {
    // Arrange & Act
    let rewritten = rewrite("pub struct Widget\n\nconst LIMIT: u8 = 3;\n");

    // Assert
    assert!(rewritten.is_none());
}

#[test]
fn rewrite_of_a_file_whose_opening_comment_runs_into_its_first_item_is_none() {
    // Arrange & Act
    let rewritten = rewrite("// Copyright\npub struct Widget;\n\nconst LIMIT: u8 = 3;\n");

    // Assert
    assert!(rewritten.is_none());
}

#[test]
fn rewrite_of_a_file_with_a_macro_rules_is_none() {
    // Arrange & Act
    let rewritten = rewrite(
        "macro_rules! limit {\n    () => {\n        3\n    };\n}\n\nconst LIMIT: u8 = limit!();\n",
    );

    // Assert
    assert!(rewritten.is_none());
}

// The scope the rule has, exactly: a fixer that edits a file no rule governs is
// the failure ADR-FixOnlyWhatIsSafe was written after.
#[test]
fn rewrite_of_a_module_registry_is_none() {
    // Arrange
    let file = SourceFile::new(
        "src/inner/mod.rs",
        "pub struct Widget;\n\nconst LIMIT: u8 = 3;\n",
    );

    // Act
    let rewritten = LeadingConstantsRewriter::new().rewrite(&file);

    // Assert
    assert!(rewritten.is_none());
}

#[test]
fn rewrite_of_a_test_file_is_none() {
    // Arrange
    let file = SourceFile::new(
        "tests/widget_tests.rs",
        "fn widget() {}\n\nconst LIMIT: u8 = 3;\n",
    );

    // Act
    let rewritten = LeadingConstantsRewriter::new().rewrite(&file);

    // Assert
    assert!(rewritten.is_none());
}

#[test]
fn rule_is_the_one_whose_offences_it_repairs() {
    // Arrange & Act
    let rule = LeadingConstantsRewriter::new().rule();

    // Assert
    assert_eq!(rule, "leading-constants");
}
