// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A module is declared by name, not by an explicit path.
//
// The attribute is legal Rust and this rule is not about taste: `#[path]` is
// what makes `registry-completeness` give a confident wrong answer, reporting a
// perfectly compiled file as never compiled. Forbidding it is what lets that
// rule keep resolving names by convention.
//
// `#[cfg_attr(unix, path = "...")]` is deliberately left alone -- a
// platform-gated module is the one legitimate use, and reporting it would accuse
// correct code.

use stern4rust::reporting::offence::Offence;
use stern4rust::rule::Rule;
use stern4rust::rules::declared_by_name_rule::DeclaredByNameRule;
use stern4rust::source_file::SourceFile;

const HEADER: &str = "// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>\n\
                      // Licensed under the MIT License\n\
                      // SPDX-License-Identifier: MIT\n";

const RULE: &str = "declared-by-name";

fn check(path: &str, body: &str) -> Vec<Offence> {
    DeclaredByNameRule::new().check(&SourceFile::new(path, &format!("{HEADER}\n{body}")))
}

// A platform-gated module is the one honest use of the attribute, and it never
// resolves by name on every platform anyway.
#[test]
fn check_a_conditional_path_attribute_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/lib.rs",
        "#[cfg_attr(unix, path = \"unix.rs\")]\npub mod platform;\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_a_file_that_does_not_parse_reports_nothing() {
    // Arrange & Act
    let offences = check("src/lib.rs", "pub mod broken\n");

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_a_module_declared_by_name_reports_nothing() {
    // Arrange & Act
    let offences = check("src/lib.rs", "pub mod alpha;\npub mod beta;\n");

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_a_module_declared_through_a_path_attribute_reports_it() {
    // Arrange & Act
    let offences = check(
        "src/lib.rs",
        "#[path = \"elsewhere/other.rs\"]\npub mod alpha;\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].rule, RULE);
    assert_eq!(offences[0].file, "src/lib.rs");
    assert_eq!(offences[0].subject.as_deref(), Some("alpha"));
    assert_eq!(offences[0].expected.as_deref(), Some("alpha.rs"));
    assert!(
        offences[0].description.contains("elsewhere/other.rs"),
        "the file it points at is named, got {}",
        offences[0].description
    );
    assert_eq!(
        offences[0].correction,
        "move `elsewhere/other.rs` to `alpha.rs` beside this file and drop the `#[path]` attribute"
    );
}

// An attribute one level down is as invisible to name resolution as one at the
// top.
#[test]
fn check_a_nested_module_with_a_path_attribute_reports_it() {
    // Arrange & Act
    let offences = check(
        "src/lib.rs",
        "pub mod outer {\n    #[path = \"deep.rs\"]\n    pub mod inner;\n}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].subject.as_deref(), Some("inner"));
}

// The rule is not scoped to a tree: `all_tests.rs` is where the house standard
// names it, and the harm is the same anywhere.
#[test]
fn check_a_path_attribute_in_a_test_file_reports_it() {
    // Arrange & Act
    let offences = check(
        "tests/all_tests.rs",
        "#[path = \"rules/alpha_tests.rs\"]\npub mod alpha_tests;\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].subject.as_deref(), Some("alpha_tests"));
}

#[test]
fn check_an_unrelated_attribute_reports_nothing() {
    // Arrange & Act
    let offences = check("src/lib.rs", "#[cfg(feature = \"x\")]\npub mod alpha;\n");

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_reports_every_path_attribute() {
    // Arrange & Act
    let offences = check(
        "src/lib.rs",
        "#[path = \"one.rs\"]\npub mod alpha;\n\n#[path = \"two.rs\"]\npub mod beta;\n",
    );

    // Assert
    assert_eq!(offences.len(), 2);
    assert_eq!(offences[0].subject.as_deref(), Some("alpha"));
    assert_eq!(offences[1].subject.as_deref(), Some("beta"));
}

#[test]
fn check_workspace_of_a_tree_reports_nothing() {
    // Arrange
    let file = SourceFile::new("src/lib.rs", "#[path = \"x.rs\"]\npub mod alpha;\n");

    // Act
    let offences = DeclaredByNameRule::new().check_workspace(&[file]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn is_configured_for_a_rule_that_needs_nothing_returns_true() {
    // Arrange & Act
    let configured = DeclaredByNameRule::new().is_configured();

    // Assert
    assert!(configured);
}

#[test]
fn name_is_the_kebab_case_rule_name_used_in_the_report() {
    // Arrange & Act
    let name = DeclaredByNameRule::new().name();

    // Assert
    assert_eq!(name, RULE);
}
