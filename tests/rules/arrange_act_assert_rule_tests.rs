// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A test reads Arrange, then one or more Act/Assert pairs.
//
// The markers are comments, which never reach the syntax tree, so this rule
// reads lines. That is what makes the string-literal case the one that matters:
// this repository's own tests embed Rust source in raw strings, and a line
// scanner that cannot tell code from a string reports the fixtures it is built
// from.

use stern4rust::reporting::offence::Offence;
use stern4rust::rule::Rule;
use stern4rust::rules::arrange_act_assert_rule::ArrangeActAssertRule;
use stern4rust::source_file::SourceFile;

const HEADER: &str = "// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>\n\
                      // Licensed under the MIT License\n\
                      // SPDX-License-Identifier: MIT\n";

const RULE: &str = "arrange-act-assert";

// A raw string holding lines that begin with markers. Excluding it is the
// difference between this rule passing and failing this crate.
const WITH_EMBEDDED_SOURCE: &str = r##"#[test]
fn parse_of_a_fixture_returns_it() {
    // Arrange & Act
    let source = r#"
// Act
// Assert
"#;

    // Assert
    assert!(!source.is_empty());
}
"##;

fn check(body: &str) -> Vec<Offence> {
    ArrangeActAssertRule::new().check(&SourceFile::new(
        "tests/widget_tests.rs",
        &format!("{HEADER}\n{body}"),
    ))
}

#[test]
fn check_a_canonical_arrange_act_assert_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "#[test]\nfn poll_of_a_queue_returns_it() {\n    // Arrange\n    let q = 1;\n\n    // Act\n    let p = q;\n\n    // Assert\n    assert_eq!(p, 1);\n}\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_a_file_that_does_not_parse_reports_nothing() {
    // Arrange & Act
    let offences = check("#[test]\nfn a_b_c() {\n");

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_a_fully_merged_marker_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "#[test]\nfn label_reads_as_itself() {\n    // Arrange & Act & Assert\n    assert_eq!(1, 1);\n}\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_a_helper_function_reports_nothing() {
    // Arrange & Act
    let offences = check("fn helper() -> usize {\n    1\n}\n");

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// Comment lines above a marker are folded into it. The explanation is the thing
// worth keeping, so it must not read as a spacing offence.
#[test]
fn check_a_marker_below_an_explanatory_comment_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "#[test]\nfn poll_of_a_queue_returns_it() {\n    // Arrange\n    let q = 1;\n\n    // this is the interesting part\n    // Act\n    let p = q;\n\n    // Assert\n    assert_eq!(p, 1);\n}\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_a_marker_not_preceded_by_a_blank_line_reports_it() {
    // Arrange & Act
    let offences = check(
        "#[test]\nfn poll_of_a_queue_returns_it() {\n    // Arrange\n    let q = 1;\n    // Act\n    let p = q;\n\n    // Assert\n    assert_eq!(p, 1);\n}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].rule, RULE);
    assert!(
        offences[0].description.contains("blank line"),
        "got {}",
        offences[0].description
    );
    assert_eq!(offences[0].correction, "put a blank line before `// Act`");
}

// The observed style: a marker that explains itself.
#[test]
fn check_a_marker_with_trailing_prose_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "#[test]\nfn heal_of_a_partition_converges() {\n    // Arrange -- four nodes, one split\n    let n = 4;\n\n    // Act: heal the partition\n    let healed = n;\n\n    // Assert. every node commits\n    assert_eq!(healed, 4);\n}\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_a_merged_act_and_assert_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "#[test]\nfn poll_of_a_queue_returns_it() {\n    // Arrange\n    let q = 1;\n\n    // Act & Assert\n    assert_eq!(q, 1);\n}\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_a_merged_arrange_and_act_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "#[test]\nfn name_of_a_rule_returns_it() {\n    // Arrange & Act\n    let n = 1;\n\n    // Assert\n    assert_eq!(n, 1);\n}\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// A word boundary, not a prefix: `Actually` is prose, not a marker.
#[test]
fn check_a_prose_comment_beginning_with_act_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "#[test]\nfn poll_of_a_queue_returns_it() {\n    // Arrange & Act\n    let q = 1;\n\n    // Actually this is worth explaining\n    // Assert\n    assert_eq!(q, 1);\n}\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_a_source_file_reports_nothing() {
    // Arrange
    let file = SourceFile::new(
        "src/widget.rs",
        &format!("{HEADER}\n#[test]\nfn a_b_c() {{\n    let x = 1;\n}}\n"),
    );

    // Act
    let offences = ArrangeActAssertRule::new().check(&file);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_a_test_with_no_markers_reports_it() {
    // Arrange & Act
    let offences = check("#[test]\nfn poll_of_a_queue_returns_it() {\n    assert_eq!(1, 1);\n}\n");

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].rule, RULE);
    assert_eq!(
        offences[0].subject.as_deref(),
        Some("poll_of_a_queue_returns_it")
    );
}

#[test]
fn check_an_act_without_an_assert_reports_it() {
    // Arrange & Act
    let offences = check(
        "#[test]\nfn poll_of_a_queue_returns_it() {\n    // Arrange\n    let q = 1;\n\n    // Act\n    let _p = q;\n}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
    assert!(
        offences[0].description.contains("Arrange, Act"),
        "the sequence found is named, got {}",
        offences[0].description
    );
}

#[test]
fn check_an_assert_without_an_act_reports_it() {
    // Arrange & Act
    let offences = check(
        "#[test]\nfn poll_of_a_queue_returns_it() {\n    // Arrange\n    let q = 1;\n\n    // Assert\n    assert_eq!(q, 1);\n}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
}

// `// Act` first means the Arrange was merged and must say so.
#[test]
fn check_an_unmerged_missing_arrange_reports_it() {
    // Arrange & Act
    let offences = check(
        "#[test]\nfn poll_of_a_queue_returns_it() {\n    // Act\n    let q = 1;\n\n    // Assert\n    assert_eq!(q, 1);\n}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
    assert!(
        offences[0].correction.contains("Arrange & Act"),
        "got {}",
        offences[0].correction
    );
}

// The case this rule lives or dies on.
#[test]
fn check_markers_inside_a_string_literal_are_ignored() {
    // Arrange & Act
    let offences = check(WITH_EMBEDDED_SOURCE);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_several_act_assert_pairs_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "#[test]\nfn poll_of_a_queue_returns_it() {\n    // Arrange\n    let q = 1;\n\n    // Act\n    let p = q;\n\n    // Assert\n    assert_eq!(p, 1);\n\n    // Act\n    let r = p;\n\n    // Assert\n    assert_eq!(r, 1);\n}\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_of_a_tree_reports_nothing() {
    // Arrange
    let file = SourceFile::new("tests/widget_tests.rs", "#[test]\nfn a_b_c() {}\n");

    // Act
    let offences = ArrangeActAssertRule::new().check_workspace(&[file]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn is_configured_for_a_rule_that_needs_nothing_returns_true() {
    // Arrange & Act
    let configured = ArrangeActAssertRule::new().is_configured();

    // Assert
    assert!(configured);
}

#[test]
fn name_is_the_kebab_case_rule_name_used_in_the_report() {
    // Arrange & Act
    let name = ArrangeActAssertRule::new().name();

    // Assert
    assert_eq!(name, RULE);
}
