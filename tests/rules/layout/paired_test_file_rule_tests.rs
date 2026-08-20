// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A `<X>_tests.rs` names the source file it exercises, and that file exists.
//
// The other half of the mirrored pairing. twin4rust starts at a source file and
// looks for its test; this starts at a test file and looks for its source, which
// is the direction nothing checked.
//
// `_proptest_tests.rs` is exempt: a second suite for a module it does not name,
// whose stem resolves to a file that was never meant to exist.

use stern4rust::reporting::offence::Offence;
use stern4rust::rule::Rule;
use stern4rust::rules::layout::paired_test_file_rule::PairedTestFileRule;
use stern4rust::source_file::SourceFile;

const HEADER: &str = "// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>\n\
                      // Licensed under the MIT License\n\
                      // SPDX-License-Identifier: MIT\n";

const RULE: &str = "paired-test-file";

fn check(paths: &[&str]) -> Vec<Offence> {
    let files: Vec<SourceFile> = paths
        .iter()
        .map(|path| SourceFile::new(path, HEADER))
        .collect();
    PairedTestFileRule::new().check_workspace(&files)
}

// The file that would carry the offence is the one that does not exist, so this
// cannot be answered a file at a time.
#[test]
fn check_of_a_single_file_reports_nothing() {
    // Arrange
    let file = SourceFile::new("tests/rules/orphan_tests.rs", HEADER);

    // Act
    let offences = PairedTestFileRule::new().check(&file);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_of_a_nested_paired_test_file_reports_nothing() {
    // Arrange & Act
    let offences = check(&[
        "src/rules/widget_rule.rs",
        "tests/rules/widget_rule_tests.rs",
    ]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_of_a_paired_test_file_reports_nothing() {
    // Arrange & Act
    let offences = check(&["src/source_file.rs", "tests/source_file_tests.rs"]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// The stem resolves to a file nobody ever meant to write, so the pairing
// question cannot be asked of it. Excluded rather than answered wrongly.
#[test]
fn check_workspace_of_a_proptest_file_reports_nothing() {
    // Arrange & Act
    let offences = check(&[
        "src/state/retention_window.rs",
        "tests/state/retention_window_proptest_tests.rs",
    ]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// A helper under tests/ is not a test file and names no source file.
#[test]
fn check_workspace_of_a_support_file_reports_nothing() {
    // Arrange & Act
    let offences = check(&["src/lib.rs", "tests/rules/support.rs"]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_of_a_test_file_in_the_wrong_directory_reports_it() {
    // Arrange & Act
    let offences = check(&["src/rules/widget_rule.rs", "tests/widget_rule_tests.rs"]);

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(
        offences[0].expected.as_deref(),
        Some("src/widget_rule.rs"),
        "the pairing is by path, not by name alone"
    );
}

#[test]
fn check_workspace_of_a_test_file_with_no_source_file_reports_it() {
    // Arrange & Act
    let offences = check(&["src/lib.rs", "tests/state/etheram_state_ibft_tests.rs"]);

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].rule, RULE);
    assert_eq!(offences[0].file, "tests/state/etheram_state_ibft_tests.rs");
    assert_eq!(offences[0].line, 1);
    assert_eq!(
        offences[0].subject.as_deref(),
        Some("tests/state/etheram_state_ibft_tests.rs")
    );
    assert_eq!(
        offences[0].expected.as_deref(),
        Some("src/state/etheram_state_ibft.rs")
    );
    assert!(
        offences[0]
            .description
            .contains("src/state/etheram_state_ibft.rs"),
        "got {}",
        offences[0].description
    );
    assert_eq!(
        offences[0].correction,
        "rename it after the source file it exercises, or delete it if that file is gone"
    );
}

// all_tests.rs ends in `_tests.rs` and is a registry, not a test file. Resolving
// it would look for `src/all.rs`.
#[test]
fn check_workspace_of_an_all_tests_registry_reports_nothing() {
    // Arrange & Act
    let offences = check(&["src/lib.rs", "tests/all_tests.rs"]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// Every unpaired file is named, because each is a separate rename.
#[test]
fn check_workspace_reports_every_unpaired_file() {
    // Arrange & Act
    let offences = check(&[
        "src/kept.rs",
        "tests/kept_tests.rs",
        "tests/gone_tests.rs",
        "tests/also_gone_tests.rs",
    ]);

    // Assert. Reported in the order the files were walked; Offence::sort_key is
    // what orders the report, not the rule.
    assert_eq!(offences.len(), 2);
    assert_eq!(offences[0].file, "tests/gone_tests.rs");
    assert_eq!(offences[1].file, "tests/also_gone_tests.rs");
}

#[test]
fn is_configured_for_a_rule_that_needs_nothing_returns_true() {
    // Arrange & Act
    let configured = PairedTestFileRule::new().is_configured();

    // Assert
    assert!(configured);
}

#[test]
fn name_is_the_kebab_case_rule_name_used_in_the_report() {
    // Arrange & Act
    let name = PairedTestFileRule::new().name();

    // Assert
    assert_eq!(name, RULE);
}
