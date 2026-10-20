// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A tuple in a return type repeats no element type. Where two values share a
// type, position is the only thing telling them apart and a swap compiles; a
// struct names them. Reported wherever the tuple sits, in a private function as
// much as a public one, once per function; a trait impl is the trait's.

use stern4rust::reporting::offence::Offence;
use stern4rust::rule::Rule;
use stern4rust::rules::source::distinct_tuple_elements_rule::DistinctTupleElementsRule;
use stern4rust::source_file::SourceFile;

const HEADER: &str = "// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>\n\
                      // Licensed under the MIT License\n\
                      // SPDX-License-Identifier: MIT\n";
const RULE: &str = "distinct-tuple-elements";

fn check(path: &str, body: &str) -> Vec<Offence> {
    DistinctTupleElementsRule::new().check(&SourceFile::new(path, &format!("{HEADER}\n{body}")))
}

#[test]
fn check_a_file_that_does_not_parse_reports_nothing() {
    // Arrange & Act
    let offences = check("src/broken.rs", "pub fn broken( -> {\n");

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_a_function_in_an_inline_module_reports_it() {
    // Arrange & Act
    let offences = check(
        "src/outer.rs",
        "mod inner {\n    pub fn span() -> (u64, u64) {\n        (0, 1)\n    }\n}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].subject.as_deref(), Some("span"));
}

#[test]
fn check_a_function_returning_distinct_types_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/write.rs",
        "pub fn write() -> (PathBuf, u64) {\n    todo!()\n}\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// A private function's callers are in its own file and can swap its results as
// easily as anyone's.
#[test]
fn check_a_private_function_reports_it() {
    // Arrange & Act
    let offences = check(
        "src/bounds.rs",
        "fn bounds(values: &[u64]) -> (u64, u64) {\n    (values[0], values[1])\n}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].rule, RULE);
    assert_eq!(offences[0].subject.as_deref(), Some("bounds"));
    assert_eq!(offences[0].line, 5);
}

#[test]
fn check_a_repeated_tuple_names_the_type_and_offers_a_struct() {
    // Arrange & Act
    let offences = check(
        "src/bounds.rs",
        "pub fn bounds(values: &[u64]) -> (u64, u64) {\n    (values[0], values[1])\n}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
    assert!(
        offences[0].description.contains("`(u64, u64)`"),
        "got {:?}",
        offences[0].description
    );
    assert!(
        offences[0].correction.contains("struct") && offences[0].correction.contains("`u64`"),
        "got {:?}",
        offences[0].correction
    );
}

// A returned callback's parameter list is its contract, not a tuple.
#[test]
fn check_a_returned_callback_taking_two_of_one_type_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/sum.rs",
        "pub fn sum() -> impl Fn(u8, u8) -> u8 {\n    |a, b| a + b\n}\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_a_test_file_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "tests/bounds_tests.rs",
        "fn bounds() -> (u64, u64) {\n    (0, 1)\n}\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_a_trait_declaration_reports_it() {
    // Arrange & Act
    let offences = check(
        "src/range.rs",
        "pub trait Range {\n    fn ends(&self) -> (u64, u64);\n}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].subject.as_deref(), Some("ends"));
}

#[test]
fn check_a_trait_impl_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/pairs.rs",
        "pub struct Pairs;\n\nimpl Iterator for Pairs {\n    type Item = (u8, u8);\n\n    \
         fn next(&mut self) -> Option<(u8, u8)> {\n        None\n    }\n}\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_a_tuple_wrapped_in_a_generic_reports_it() {
    // Arrange & Act
    let offences = check(
        "src/window.rs",
        "pub fn window() -> Option<(Instant, Instant)> {\n    None\n}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].subject.as_deref(), Some("window"));
}

#[test]
fn check_an_inherent_method_reports_it() {
    // Arrange & Act
    let offences = check(
        "src/tally.rs",
        "pub struct Tally;\n\nimpl Tally {\n    pub fn counts(&self) -> (usize, usize) {\n        (0, 0)\n    }\n}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].subject.as_deref(), Some("counts"));
}

#[test]
fn check_two_repeated_tuples_in_one_return_type_reports_it_once() {
    // Arrange & Act
    let offences = check(
        "src/grid.rs",
        "pub fn corners() -> ((u8, u8), (u8, u8)) {\n    ((0, 0), (1, 1))\n}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
}

// This rule answers the per-file question only, and says so rather than
// inheriting a silence nobody chose.
#[test]
fn check_workspace_of_a_tree_with_a_repeated_tuple_reports_nothing() {
    // Arrange
    let file = SourceFile::new(
        "src/bounds.rs",
        "pub fn bounds() -> (u64, u64) {\n    (0, 1)\n}\n",
    );

    // Act
    let offences = DistinctTupleElementsRule::new().check_workspace(&[file]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn is_configured_for_a_rule_that_needs_nothing_returns_true() {
    // Arrange & Act
    let configured = DistinctTupleElementsRule::new().is_configured();

    // Assert
    assert!(configured);
}

#[test]
fn name_is_the_kebab_case_rule_name_used_in_the_report() {
    // Arrange & Act
    let name = DistinctTupleElementsRule::new().name();

    // Assert
    assert_eq!(name, RULE);
}

// A rule that is always configured has nothing to ask for, and says so rather
// than inventing a requirement it does not have.
#[test]
fn requirement_of_a_rule_that_needs_nothing_is_none() {
    // Arrange & Act
    let requirement = DistinctTupleElementsRule::new().requirement();

    // Assert
    assert!(requirement.is_none());
}
