// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A file that holds tests is named for it.
//
// One direction only: holding a test obliges the name. A `_tests.rs` file that
// holds none is a different question and not this rule's.
//
// The two exemptions carry the weight. A `#[test]` in `src/` is already
// `test-free-source`'s offence and renaming the file would not fix it -- the
// file has to move. A `#[test]` in a registry is already `tests-layout`'s, and
// `mod.rs` cannot be renamed at all.

use stern4rust::reporting::offence::Offence;
use stern4rust::rule::Rule;
use stern4rust::rules::test_file_name_postfix_rule::TestFileNamePostfixRule;
use stern4rust::source_file::SourceFile;

const HEADER: &str = "// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>\n\
                      // Licensed under the MIT License\n\
                      // SPDX-License-Identifier: MIT\n";

const RULE: &str = "test-file-name-postfix";

fn check(path: &str, body: &str) -> Vec<Offence> {
    TestFileNamePostfixRule::new().check(&SourceFile::new(path, &format!("{HEADER}\n{body}")))
}

// `#[tokio::test]` is a test by every meaning that matters here, and the last
// path segment is what says so.
#[test]
fn check_a_custom_test_attribute_reports_it() {
    // Arrange & Act
    let offences = check(
        "tests/runtime.rs",
        "#[tokio::test]\nasync fn poll_with_a_ready_future_returns_it() {}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].rule, RULE);
}

#[test]
fn check_a_file_that_does_not_parse_reports_nothing() {
    // Arrange & Act
    let offences = check("tests/broken.rs", "#[test]\nfn open_brace_only() {\n");

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_a_file_with_a_test_named_tests_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "tests/rules/widget_rule_tests.rs",
        "#[test]\nfn check_of_a_widget_reports_nothing() {}\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_a_file_with_a_test_not_named_tests_reports_it() {
    // Arrange & Act
    let offences = check(
        "tests/rules/widget.rs",
        "#[test]\nfn check_of_a_widget_reports_nothing() {}\n\n#[test]\nfn name_is_the_rule_name() {}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].rule, RULE);
    assert_eq!(offences[0].file, "tests/rules/widget.rs");
    assert_eq!(offences[0].line, 1);
    assert_eq!(
        offences[0].subject.as_deref(),
        Some("tests/rules/widget.rs")
    );
    assert_eq!(
        offences[0].expected.as_deref(),
        Some("tests/rules/widget_tests.rs")
    );
    assert!(
        offences[0].description.contains('2'),
        "expected the test count, got {}",
        offences[0].description
    );
    assert_eq!(
        offences[0].correction,
        "rename it `tests/rules/widget_tests.rs`"
    );
}

// One direction only. A file with no tests is not obliged to be named for them.
#[test]
fn check_a_file_with_no_tests_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "tests/rules/support.rs",
        "pub fn make_widget() -> usize {\n    1\n}\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// tests-layout already reports a function in a registry, and `mod.rs` cannot be
// renamed -- the correction would be impossible to follow.
#[test]
fn check_a_mod_registry_holding_a_test_reports_nothing() {
    // Arrange & Act
    let offences = check("tests/rules/mod.rs", "#[test]\nfn stray_test_here() {}\n");

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// test-free-source already reports this one, and the fix is to move the file,
// not to rename it.
#[test]
fn check_a_source_file_with_a_test_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/widget.rs",
        "#[test]\nfn new_of_a_widget_returns_it() {}\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// A test does not stop being a test for sitting one level down.
#[test]
fn check_a_test_in_an_inline_module_reports_it() {
    // Arrange & Act
    let offences = check(
        "tests/nested.rs",
        "mod inner {\n    #[test]\n    fn poll_of_an_empty_queue_returns_nothing() {}\n}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(
        offences[0].expected.as_deref(),
        Some("tests/nested_tests.rs")
    );
}

#[test]
fn check_an_all_tests_registry_holding_a_test_reports_nothing() {
    // Arrange & Act
    let offences = check("tests/all_tests.rs", "#[test]\nfn stray_test_here() {}\n");

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// The name of a file is a fact about that file, so it is answered one file at a
// time rather than by the tree.
#[test]
fn check_workspace_of_a_tree_with_a_misnamed_file_reports_nothing() {
    // Arrange
    let file = SourceFile::new("tests/widget.rs", "#[test]\nfn a_b_c() {}\n");

    // Act
    let offences = TestFileNamePostfixRule::new().check_workspace(&[file]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn is_configured_for_a_rule_that_needs_nothing_returns_true() {
    // Arrange & Act
    let configured = TestFileNamePostfixRule::new().is_configured();

    // Assert
    assert!(configured);
}

#[test]
fn name_is_the_kebab_case_rule_name_used_in_the_report() {
    // Arrange & Act
    let name = TestFileNamePostfixRule::new().name();

    // Assert
    assert_eq!(name, RULE);
}
