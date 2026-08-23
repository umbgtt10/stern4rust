// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Tests live in tests/, and the production source tree carries none of them.
//
// The rule is about where tests live, so it judges the source tree and leaves
// tests/ alone -- a #[test] under tests/ is the whole point of tests/, and a
// rule that reported it would report every test in the workspace.

use stern4rust::reporting::offence::Offence;
use stern4rust::rule::Rule;
use stern4rust::rules::source::test_free_source_rule::TestFreeSourceRule;
use stern4rust::source_file::SourceFile;

const RULE: &str = "test-free-source";

fn check(path: &str, body: &str) -> Vec<Offence> {
    TestFreeSourceRule::new().check(&SourceFile::new(path, body))
}

// Applying a derive behind a feature is ordinary library work. serde is the
// case this rule must not break.
#[test]
fn check_a_source_file_with_a_cfg_attr_on_a_feature_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/subject.rs",
        "#[cfg_attr(feature = \"serde\", derive(Serialize))]\npub struct A;\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_a_source_file_with_a_cfg_attr_on_test_reports_it() {
    // Arrange & Act
    let offences = check(
        "src/subject.rs",
        "#[cfg_attr(test, derive(Debug))]\npub struct A;\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].rule, RULE);
}

#[test]
fn check_a_source_file_with_a_cfg_test_module_reports_it() {
    // Arrange & Act
    let offences = check("src/subject.rs", "#[cfg(test)]\nmod tests {}\n");

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].file, "src/subject.rs");
    assert_eq!(offences[0].line, 1);
    assert!(
        offences[0]
            .description
            .contains("`#[cfg(test)]` module `tests`"),
        "got {}",
        offences[0].description
    );
}

#[test]
fn check_a_source_file_with_a_test_function_reports_it() {
    // Arrange & Act
    let offences = check("src/subject.rs", "#[test]\nfn checks_it() {}\n");

    // Assert
    assert_eq!(offences.len(), 1);
    assert!(
        offences[0]
            .description
            .contains("test function `checks_it`"),
        "got {}",
        offences[0].description
    );
}

#[test]
fn check_a_source_file_without_tests_reports_nothing() {
    // Arrange & Act
    let offences = check("src/subject.rs", "pub struct A;\n");

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// A #[test] under tests/ is the entire point of tests/.
#[test]
fn check_a_test_file_reports_nothing() {
    // Arrange & Act
    let offences = check("tests/subject_tests.rs", "#[test]\nfn checks_it() {}\n");

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// The correction names the file to move the test into, not the policy it
// breaks -- src/<path>.rs mirrors onto tests/<path>_tests.rs.
#[test]
fn check_correction_names_the_mirrored_test_file() {
    // Arrange & Act
    let offences = check("src/rules/subject.rs", "#[cfg(test)]\nmod tests {}\n");

    // Assert
    assert!(
        offences[0]
            .correction
            .contains("tests/rules/subject_tests.rs"),
        "got {}",
        offences[0].correction
    );
}

#[test]
fn check_reports_every_offending_item_rather_than_only_the_first() {
    // Arrange & Act
    let offences = check(
        "src/subject.rs",
        "#[cfg(test)]\nmod tests {}\n\n#[test]\nfn alpha() {}\n",
    );

    // Assert
    assert_eq!(offences.len(), 2);
}

#[test]
fn name_is_the_kebab_case_rule_name_used_in_the_report() {
    // Arrange & Act
    let name = TestFreeSourceRule::new().name();

    // Assert
    assert_eq!(name, RULE);
}
