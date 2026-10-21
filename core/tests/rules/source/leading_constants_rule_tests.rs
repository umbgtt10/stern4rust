// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A source file reads header, imports, constants, then everything else. A
// module-level constant or static declared after any other item is reported,
// once per file, at the first one out of place; imports and `mod` declarations
// are not judged, inline modules are not descended into, and registries and
// tests/ are exempt.

use stern4rust::reporting::offence::Offence;
use stern4rust::rule::Rule;
use stern4rust::rules::source::leading_constants_rule::LeadingConstantsRule;
use stern4rust::source_file::SourceFile;

const HEADER: &str = "// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>\n\
                      // Licensed under the MIT License\n\
                      // SPDX-License-Identifier: MIT\n";
const RULE: &str = "leading-constants";

fn check(path: &str, body: &str) -> Vec<Offence> {
    LeadingConstantsRule::new().check(&SourceFile::new(path, &format!("{HEADER}\n{body}")))
}

#[test]
fn check_a_constant_after_a_function_reports_it() {
    // Arrange & Act
    let offences = check(
        "src/widget.rs",
        "pub fn widget() {}\n\nconst LIMIT: u8 = 3;\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].subject.as_deref(), Some("LIMIT"));
}

#[test]
fn check_a_constant_after_a_struct_reports_it() {
    // Arrange & Act
    let offences = check(
        "src/widget.rs",
        "use std::fmt;\n\npub struct Widget;\n\nconst LIMIT: u8 = 3;\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].rule, RULE);
    assert_eq!(offences[0].subject.as_deref(), Some("LIMIT"));
    assert_eq!(offences[0].line, 9);
}

#[test]
fn check_a_constant_after_a_type_alias_reports_it() {
    // Arrange & Act
    let offences = check(
        "src/widget.rs",
        "pub type Size = u8;\n\nconst LIMIT: Size = 3;\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
}

#[test]
fn check_a_constant_after_an_impl_reports_it() {
    // Arrange & Act
    let offences = check("src/widget.rs", "impl Widget {}\n\nconst LIMIT: u8 = 3;\n");

    // Assert
    assert_eq!(offences.len(), 1);
    assert!(
        offences[0].description.contains("`impl Widget`"),
        "got {:?}",
        offences[0].description
    );
}

#[test]
fn check_a_file_of_only_constants_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/limits.rs",
        "pub const LOW: u8 = 1;\n\npub const HIGH: u8 = 9;\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_a_file_that_does_not_parse_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/broken.rs",
        "pub struct Widget\n\nconst LIMIT: u8 = 3;\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_a_file_the_fix_can_repair_offers_it() {
    // Arrange & Act
    let offences = check(
        "src/widget.rs",
        "pub struct Widget;\n\nconst LIMIT: u8 = 3;\n",
    );

    // Assert
    assert!(
        offences[0].correction.contains("`--fix`") && !offences[0].correction.contains("by hand"),
        "got {:?}",
        offences[0].correction
    );
}

// The fixer has to decline it, so the correction says why and that the move is
// the reader's to make.
#[test]
fn check_a_file_the_fix_declines_says_why() {
    // Arrange & Act
    let offences = check(
        "src/widget.rs",
        "macro_rules! limit {\n    () => {\n        3\n    };\n}\n\nconst LIMIT: u8 = limit!();\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
    assert!(
        offences[0].correction.contains("by hand")
            && offences[0].correction.contains("`macro_rules!`"),
        "got {:?}",
        offences[0].correction
    );
}

#[test]
fn check_a_misplaced_constant_names_it_and_the_item_it_follows() {
    // Arrange & Act
    let offences = check(
        "src/widget.rs",
        "pub struct Widget;\n\nconst LIMIT: u8 = 3;\n",
    );

    // Assert
    assert_eq!(
        offences[0].description,
        "`LIMIT` is declared after `Widget`; constants come straight after the imports"
    );
}

#[test]
fn check_a_module_registry_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/inner/mod.rs",
        "pub mod widget;\n\npub struct Widget;\n\nconst LIMIT: u8 = 3;\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_a_static_after_a_struct_reports_it() {
    // Arrange & Act
    let offences = check(
        "src/widget.rs",
        "pub struct Widget;\n\nstatic COUNT: u8 = 0;\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].subject.as_deref(), Some("COUNT"));
}

#[test]
fn check_a_test_file_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "tests/widget_tests.rs",
        "fn widget() {}\n\nconst LIMIT: u8 = 3;\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_constants_in_an_inline_module_report_nothing() {
    // Arrange & Act
    let offences = check(
        "src/widget.rs",
        "pub struct Widget;\n\nmod inner {\n    pub struct Gadget;\n\n    const LIMIT: u8 = 3;\n}\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_constants_straight_after_the_imports_report_nothing() {
    // Arrange & Act
    let offences = check(
        "src/widget.rs",
        "use std::fmt;\n\nconst LIMIT: u8 = 3;\nstatic COUNT: u8 = 0;\n\npub struct Widget;\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_the_crate_root_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/lib.rs",
        "pub mod widget;\n\npub struct Widget;\n\nconst LIMIT: u8 = 3;\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_two_misplaced_constants_reports_the_first_once() {
    // Arrange & Act
    let offences = check(
        "src/widget.rs",
        "pub struct Widget;\n\nconst LOW: u8 = 1;\n\nfn helper() {}\n\nconst HIGH: u8 = 9;\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].subject.as_deref(), Some("LOW"));
}

// This rule answers the per-file question only, and says so rather than
// inheriting a silence nobody chose.
#[test]
fn check_workspace_of_a_tree_with_a_misplaced_constant_reports_nothing() {
    // Arrange
    let file = SourceFile::new(
        "src/widget.rs",
        "pub struct Widget;\n\nconst LIMIT: u8 = 3;\n",
    );

    // Act
    let offences = LeadingConstantsRule::new().check_workspace(&[file]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn is_configured_for_a_rule_that_needs_nothing_returns_true() {
    // Arrange & Act
    let configured = LeadingConstantsRule::new().is_configured();

    // Assert
    assert!(configured);
}

#[test]
fn name_is_the_kebab_case_rule_name_used_in_the_report() {
    // Arrange & Act
    let name = LeadingConstantsRule::new().name();

    // Assert
    assert_eq!(name, RULE);
}

// A rule that is always configured has nothing to ask for, and says so rather
// than inventing a requirement it does not have.
#[test]
fn requirement_of_a_rule_that_needs_nothing_is_none() {
    // Arrange & Act
    let requirement = LeadingConstantsRule::new().requirement();

    // Assert
    assert!(requirement.is_none());
}
