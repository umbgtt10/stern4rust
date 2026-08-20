// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A trait declares; it does not implement.
//
// The offence lands on the method carrying the body rather than on the trait,
// because the edit that fixes it is the removal of that one body. Everything a
// trait may legitimately default -- an associated type, an associated constant --
// is left alone: only a method body makes an implementor's silence look like a
// decision somebody made.

use stern4rust::reporting::offence::Offence;
use stern4rust::rule::Rule;
use stern4rust::rules::source::pure_traits_rule::PureTraitsRule;
use stern4rust::source_file::SourceFile;

const HEADER: &str = "// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>\n\
                      // Licensed under the MIT License\n\
                      // SPDX-License-Identifier: MIT\n";

const RULE: &str = "pure-traits";

fn check(path: &str, body: &str) -> Vec<Offence> {
    PureTraitsRule::new().check(&SourceFile::new(path, &format!("{HEADER}\n{body}")))
}

// An associated constant with a value is not a method body. It is a shared fact
// rather than a shared decision, and no implementor is silently inheriting
// behaviour by leaving it alone.
#[test]
fn check_a_defaulted_associated_constant_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/limits.rs",
        "pub trait Limits {\n    const MAX: usize = 20;\n\n    fn limit(&self) -> usize;\n}\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_a_file_that_does_not_parse_reports_nothing() {
    // Arrange & Act
    let offences = check("src/broken.rs", "pub trait Broken {\n");

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// A test file legitimately declares a trait with a body to stand in for a real
// one, and reporting it would report the fakes this tool's own tests are built
// from.
#[test]
fn check_a_test_file_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "tests/rules/fake_rule_tests.rs",
        "trait Fake {\n    fn value(&self) -> usize {\n        1\n    }\n}\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// A body does not stop being a body for sitting one level down.
#[test]
fn check_a_trait_in_an_inline_module_reports_the_method() {
    // Arrange & Act
    let offences = check(
        "src/nested.rs",
        "mod inner {\n    pub trait Inner {\n        fn ready(&self) -> bool {\n            true\n        }\n    }\n}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].subject.as_deref(), Some("Inner::ready"));
}

#[test]
fn check_a_trait_with_a_default_body_reports_the_method() {
    // Arrange & Act
    let offences = check(
        "src/collection.rs",
        "pub trait Collection {\n    fn len(&self) -> usize;\n    fn is_empty(&self) -> bool {\n        self.len() == 0\n    }\n}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].rule, RULE);
    assert_eq!(offences[0].file, "src/collection.rs");
    assert_eq!(offences[0].line, 7);
    assert_eq!(offences[0].subject.as_deref(), Some("Collection::is_empty"));
    assert!(
        offences[0].description.contains("Collection::is_empty"),
        "got {}",
        offences[0].description
    );
    assert_eq!(
        offences[0].correction,
        "move the body into each implementor"
    );
}

// An associated type is a declaration, whatever it is bound to.
#[test]
fn check_a_trait_with_an_associated_type_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/protocol.rs",
        "pub trait Protocol {\n    type Message;\n\n    fn handle(&self, message: Self::Message);\n}\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_a_trait_with_only_declarations_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/storage.rs",
        "pub trait Storage {\n    fn query(&self) -> usize;\n    fn mutate(&self);\n}\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// Every body is named, rather than the trait being reported once, because each
// one is a separate edit.
#[test]
fn check_a_trait_with_two_default_bodies_reports_both() {
    // Arrange & Act
    let offences = check(
        "src/rule.rs",
        "pub trait Rule {\n    fn name(&self) -> &'static str;\n    fn check(&self) -> bool {\n        false\n    }\n    fn is_configured(&self) -> bool {\n        true\n    }\n}\n",
    );

    // Assert
    assert_eq!(offences.len(), 2);
    assert_eq!(offences[0].subject.as_deref(), Some("Rule::check"));
    assert_eq!(offences[1].subject.as_deref(), Some("Rule::is_configured"));
}

// The bodies in an impl block are the whole point of an impl block. This is
// where the bodies are supposed to end up.
#[test]
fn check_an_impl_block_of_a_trait_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/vector.rs",
        "pub struct Vector;\n\nimpl Collection for Vector {\n    fn is_empty(&self) -> bool {\n        true\n    }\n}\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// This rule answers the per-file question only, and says so rather than
// inheriting a silence nobody chose.
#[test]
fn check_workspace_of_a_tree_with_a_default_body_reports_nothing() {
    // Arrange
    let file = SourceFile::new(
        "src/collection.rs",
        "pub trait Collection {\n    fn is_empty(&self) -> bool {\n        true\n    }\n}\n",
    );

    // Act
    let offences = PureTraitsRule::new().check_workspace(&[file]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn is_configured_for_a_rule_that_needs_nothing_returns_true() {
    // Arrange & Act
    let configured = PureTraitsRule::new().is_configured();

    // Assert
    assert!(configured);
}

#[test]
fn name_is_the_kebab_case_rule_name_used_in_the_report() {
    // Arrange & Act
    let name = PureTraitsRule::new().name();

    // Assert
    assert_eq!(name, RULE);
}

// A rule that is always configured has nothing to ask for, and says so rather
// than inventing a requirement it does not have.
#[test]
fn requirement_of_a_rule_that_needs_nothing_is_none() {
    // Arrange & Act
    let requirement = PureTraitsRule::new().requirement();

    // Assert
    assert!(requirement.is_none());
}
