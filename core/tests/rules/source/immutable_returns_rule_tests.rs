// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A function does not hand out a mutable reference.
//
// The offence lands on the function, because the edit that fixes it changes what
// the function returns. `&'static mut` is left alone -- a static handed over once,
// not a type lending its internals -- and so is a trait impl, whose signature is
// the trait's decision, reported where the trait is declared.

use stern4rust::reporting::offence::Offence;
use stern4rust::rule::Rule;
use stern4rust::rules::source::immutable_returns_rule::ImmutableReturnsRule;
use stern4rust::source_file::SourceFile;

const HEADER: &str = "// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>\n\
                      // Licensed under the MIT License\n\
                      // SPDX-License-Identifier: MIT\n";
const RULE: &str = "immutable-returns";

fn check(path: &str, body: &str) -> Vec<Offence> {
    ImmutableReturnsRule::new().check(&SourceFile::new(path, &format!("{HEADER}\n{body}")))
}

// A returned callback that takes a `&mut` describes what the caller's closure
// receives, not what this function lends.
#[test]
fn check_a_callback_taking_a_mut_ref_in_the_return_type_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/each.rs",
        "pub fn each() -> impl FnMut(&mut u32) {\n    |_| {}\n}\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_a_file_that_does_not_parse_reports_nothing() {
    // Arrange & Act
    let offences = check("src/broken.rs", "pub fn broken( -> {\n");

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_a_free_function_returning_a_mut_ref_reports_it() {
    // Arrange & Act
    let offences = check(
        "src/first.rs",
        "pub fn first<'a>(values: &'a mut [u32]) -> &'a mut u32 {\n    &mut values[0]\n}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].rule, RULE);
    assert_eq!(offences[0].subject.as_deref(), Some("first"));
    assert_eq!(offences[0].line, 5);
}

#[test]
fn check_a_function_in_an_inline_module_reports_it() {
    // Arrange & Act
    let offences = check(
        "src/outer.rs",
        "mod inner {\n    pub fn slot(values: &mut Vec<u32>) -> &mut u32 {\n        &mut values[0]\n    }\n}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].subject.as_deref(), Some("slot"));
}

#[test]
fn check_a_function_returning_a_value_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/total.rs",
        "pub fn total(count: u32) -> u32 {\n    count\n}\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_a_mut_ref_inside_a_generic_reports_it() {
    // Arrange & Act
    let offences = check(
        "src/find.rs",
        "pub fn find(values: &mut [u32]) -> Option<&mut u32> {\n    values.first_mut()\n}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].subject.as_deref(), Some("find"));
}

#[test]
fn check_a_mut_ref_inside_a_tuple_reports_it() {
    // Arrange & Act
    let offences = check(
        "src/split.rs",
        "pub fn split(values: &mut [u32]) -> (usize, &mut [u32]) {\n    (values.len(), values)\n}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].subject.as_deref(), Some("split"));
}

// A chain of `&mut Self` calls holds a mutable borrow across every call; taking
// and returning `self` keeps the chain and drops the loan.
#[test]
fn check_a_mut_self_return_offers_to_take_self_by_value() {
    // Arrange & Act
    let offences = check(
        "src/check.rs",
        "pub struct Check;\n\nimpl Check {\n    pub fn observe(&mut self) -> &mut Self {\n        self\n    }\n}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
    assert!(
        offences[0].correction.contains("`self` by value"),
        "got {:?}",
        offences[0].correction
    );
}

#[test]
fn check_a_shared_ref_return_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/meter.rs",
        "pub struct Meter {\n    total: u64,\n}\n\nimpl Meter {\n    pub fn total(&self) -> &u64 {\n        &self.total\n    }\n}\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// What an embedded executor requires: a static handed over once, with no type
// whose internals it could lend.
#[test]
fn check_a_static_mut_ref_return_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/resources.rs",
        "pub fn resources() -> &'static mut [u8; 4] {\n    todo!()\n}\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_a_test_file_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "tests/fake_store.rs",
        "pub fn slot(values: &mut Vec<u32>) -> &mut u32 {\n    &mut values[0]\n}\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// Where the trait is the author's own, the declaration is the one place changing
// the return type is both possible and enough.
#[test]
fn check_a_trait_declaration_returning_a_mut_ref_reports_it() {
    // Arrange & Act
    let offences = check(
        "src/store.rs",
        "pub trait Store {\n    fn slot(&mut self) -> &mut u32;\n}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].subject.as_deref(), Some("slot"));
}

#[test]
fn check_a_trait_impl_returning_a_mut_ref_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/meter.rs",
        "pub struct Meter {\n    total: u64,\n}\n\nimpl std::ops::DerefMut for Meter {\n    \
         fn deref_mut(&mut self) -> &mut u64 {\n        &mut self.total\n    }\n}\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_an_inherent_method_returning_a_mut_ref_offers_to_return_the_value() {
    // Arrange & Act
    let offences = check(
        "src/meter.rs",
        "pub struct Meter {\n    total: u64,\n}\n\nimpl Meter {\n    pub fn total_mut(&mut self) -> &mut u64 {\n        &mut self.total\n    }\n}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].subject.as_deref(), Some("total_mut"));
    assert!(
        offences[0].correction.contains("return the value"),
        "got {:?}",
        offences[0].correction
    );
}

// This rule answers the per-file question only, and says so rather than
// inheriting a silence nobody chose.
#[test]
fn check_workspace_of_a_tree_with_a_mut_ref_return_reports_nothing() {
    // Arrange
    let file = SourceFile::new(
        "src/first.rs",
        "pub fn first(values: &mut [u32]) -> &mut u32 {\n    &mut values[0]\n}\n",
    );

    // Act
    let offences = ImmutableReturnsRule::new().check_workspace(&[file]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn is_configured_for_a_rule_that_needs_nothing_returns_true() {
    // Arrange & Act
    let configured = ImmutableReturnsRule::new().is_configured();

    // Assert
    assert!(configured);
}

#[test]
fn name_is_the_kebab_case_rule_name_used_in_the_report() {
    // Arrange & Act
    let name = ImmutableReturnsRule::new().name();

    // Assert
    assert_eq!(name, RULE);
}

// A rule that is always configured has nothing to ask for, and says so rather
// than inventing a requirement it does not have.
#[test]
fn requirement_of_a_rule_that_needs_nothing_is_none() {
    // Arrange & Act
    let requirement = ImmutableReturnsRule::new().requirement();

    // Assert
    assert!(requirement.is_none());
}
