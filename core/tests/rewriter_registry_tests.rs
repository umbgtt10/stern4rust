// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The fixers a run applies: each one whose rule the package applies, in turn,
// each working on what the one before left, so two fixers repairing one file
// compose rather than the second undoing the first.

use stern4rust::rewriter_registry::RewriterRegistry;
use stern4rust::source_file::SourceFile;
use stern4rust::source_rewriter::SourceRewriter;

struct Appending {
    rule: &'static str,
    suffix: &'static str,
}

impl SourceRewriter for Appending {
    fn rule(&self) -> &'static str {
        self.rule
    }

    fn rewrite(&self, file: &SourceFile) -> Option<String> {
        Some(format!("{}{}", file.contents(), self.suffix))
    }
}

fn registry() -> RewriterRegistry {
    RewriterRegistry::new(vec![
        Box::new(Appending {
            rule: "first-rule",
            suffix: "a",
        }),
        Box::new(Appending {
            rule: "second-rule",
            suffix: "b",
        }),
    ])
}

#[test]
fn all_reorders_an_impl_where_ordered_impl_functions_applies() {
    // Arrange
    let file = SourceFile::new(
        "src/store.rs",
        "impl Store {\n    fn b() {}\n\n    fn a() {}\n}\n",
    );

    // Act
    let rewritten = RewriterRegistry::all().rewrite(&file, &["ordered-impl-functions"]);

    // Assert
    assert_eq!(
        rewritten.as_deref(),
        Some("impl Store {\n    fn a() {}\n\n    fn b() {}\n}\n")
    );
}

#[test]
fn all_repairs_a_test_file_where_test_file_structure_applies() {
    // Arrange
    let file = SourceFile::new(
        "tests/widget_tests.rs",
        "#[test]\nfn zeta() {}\n\n#[test]\nfn alpha() {}\n",
    );

    // Act
    let rewritten = RewriterRegistry::all().rewrite(&file, &["test-file-structure"]);

    // Assert
    assert_eq!(
        rewritten.as_deref(),
        Some("#[test]\nfn alpha() {}\n\n#[test]\nfn zeta() {}\n")
    );
}

#[test]
fn rewrite_hands_each_rewriter_what_the_one_before_left() {
    // Arrange
    let file = SourceFile::new("src/widget.rs", "x");

    // Act
    let rewritten = registry().rewrite(&file, &["first-rule", "second-rule"]);

    // Assert
    assert_eq!(rewritten.as_deref(), Some("xab"));
}

// A rule the package skips, or the run did not select, governs nothing, and a
// fixer must never edit a file no rule governs.
#[test]
fn rewrite_skips_a_rewriter_whose_rule_is_not_applied() {
    // Arrange
    let file = SourceFile::new("src/widget.rs", "x");

    // Act
    let rewritten = registry().rewrite(&file, &["second-rule"]);

    // Assert
    assert_eq!(rewritten.as_deref(), Some("xb"));
}

#[test]
fn rewrite_with_no_rewriter_applied_is_none() {
    // Arrange
    let file = SourceFile::new("src/widget.rs", "x");

    // Act
    let rewritten = registry().rewrite(&file, &["another-rule"]);

    // Assert
    assert!(rewritten.is_none());
}
