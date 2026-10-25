// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A function returns its output; it does not write it through a `&mut`
// parameter.
//
// The offence lands on the parameter carrying the mutable reference, because the
// edit that fixes it turns that argument into part of the return value. The
// `&mut self` receiver is left alone -- it is the one channel the house standard
// sanctions -- and a trait impl is left alone because its signature is the
// trait's decision, reported where the trait is declared rather than at every
// implementor.

use stern4rust::reporting::offence::Offence;
use stern4rust::rule::Rule;
use stern4rust::rules::source::returned_mutations_rule::ReturnedMutationsRule;
use stern4rust::source_file::SourceFile;

const HEADER: &str = "// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>\n\
                      // Licensed under the MIT License\n\
                      // SPDX-License-Identifier: MIT\n";
const RULE: &str = "returned-mutations";

fn check(path: &str, body: &str) -> Vec<Offence> {
    ReturnedMutationsRule::new().check(&SourceFile::new(path, &format!("{HEADER}\n{body}")))
}

// A bare function pointer taking a `&mut` is a callback contract by another
// spelling.
#[test]
fn check_a_bare_fn_pointer_taking_a_mut_ref_reports_nothing() {
    // Arrange & Act
    let offences = check("src/run.rs", "pub fn run(callback: fn(&mut u32)) {}\n");

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_a_by_value_parameter_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/consume.rs",
        "pub fn consume(value: Vec<u8>) -> usize {\n    value.len()\n}\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// A callback taking a `&mut` describes the callback's contract, not the
// function's own parameter list, so it is left alone.
#[test]
fn check_a_callback_taking_a_mut_ref_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/each.rs",
        "pub fn each(apply: impl FnMut(&mut u32)) {}\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// Only storing the borrow is left alone. A constructor that does anything else
// with its parameters is judged like any other function.
#[test]
fn check_a_constructor_doing_more_than_storing_a_mut_borrow_reports_it() {
    // Arrange & Act
    let offences = check(
        "src/router.rs",
        "pub struct Router<'a> {\n    protocol: &'a mut u32,\n    calls: u32,\n}\n\nimpl<'a> Router<'a> {\n    \
         pub fn new(protocol: &'a mut u32) -> Self {\n        Self { protocol, calls: 0 }\n    }\n}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].subject(), Some("new::protocol"));
}

// The unparseable file reads as a file with nothing wrong with it only because
// another rule has already reported that it does not parse.
#[test]
fn check_a_file_that_does_not_parse_reports_nothing() {
    // Arrange & Act
    let offences = check("src/broken.rs", "pub fn broken(");

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_a_free_function_with_a_mut_ref_parameter_reports_it() {
    // Arrange & Act
    let offences = check("src/apply.rs", "pub fn apply(target: &mut Vec<u8>) {}\n");

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].rule(), RULE);
    assert_eq!(offences[0].file(), "src/apply.rs");
    assert_eq!(offences[0].line(), 5);
    assert_eq!(offences[0].subject(), Some("apply::target"));
    assert!(
        offences[0].description().contains("apply::target"),
        "got {}",
        offences[0].description()
    );
    assert_eq!(
        offences[0].correction(),
        "return the new value instead of writing it through `&mut target`"
    );
}

// A body one level down is still a body this rule reads.
#[test]
fn check_a_function_in_an_inline_module_reports_it() {
    // Arrange & Act
    let offences = check(
        "src/nested.rs",
        "mod inner {\n    pub fn tweak(value: &mut u32) {}\n}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].subject(), Some("tweak::value"));
}

// A `&mut` buried in a generic argument is still a `&mut` the caller hands in.
#[test]
fn check_a_mut_ref_inside_a_generic_reports_the_parameter() {
    // Arrange & Act
    let offences = check("src/stash.rs", "pub fn stash(slot: Option<&mut u32>) {}\n");

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].subject(), Some("stash::slot"));
}

// A `&mut` buried in a tuple is still a `&mut` the caller hands in.
#[test]
fn check_a_mut_ref_inside_a_tuple_reports_the_parameter() {
    // Arrange & Act
    let offences = check("src/pair.rs", "pub fn pair(items: (&mut u32, bool)) {}\n");

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].subject(), Some("pair::items"));
}

// The receiver is the one mutable reference the house standard sanctions.
#[test]
fn check_a_mut_self_receiver_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/counter.rs",
        "pub struct Counter;\n\nimpl Counter {\n    pub fn bump(&mut self) {}\n}\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// A plain constructor writes nothing through the borrow it is handed: it keeps
// it, and the writes come later through `&mut self`, which this rule leaves
// alone. A component borrowing its owner for one call is built this way.
#[test]
fn check_a_plain_constructor_storing_a_mut_borrow_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/router.rs",
        "pub struct Router<'a> {\n    protocol: &'a mut u32,\n}\n\nimpl<'a> Router<'a> {\n    \
         pub fn new(protocol: &'a mut u32) -> Self {\n        Self { protocol }\n    }\n}\n",
    );

    // Assert
    assert!(offences.is_empty(), "{offences:?}");
}

#[test]
fn check_a_shared_ref_parameter_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/read.rs",
        "pub fn read(input: &Vec<u8>) -> usize {\n    input.len()\n}\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// A test file legitimately hands a fake a `&mut` to stand in for a real
// collaborator, and reporting it would report the fakes this tool's own tests
// are built from.
#[test]
fn check_a_test_file_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "tests/rules/fake_rule_tests.rs",
        "pub fn helper(out: &mut String) {}\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// A trait declaration is where a `&mut` parameter is the author's own decision,
// so it is where the offence is reported.
#[test]
fn check_a_trait_declaration_with_a_mut_ref_parameter_reports_it() {
    // Arrange & Act
    let offences = check(
        "src/store.rs",
        "pub trait Store {\n    fn commit(&self, buffer: &mut Vec<u8>);\n}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].subject(), Some("commit::buffer"));
}

// A trait impl's signature is fixed by the trait, which may be external and
// unchangeable; the offence belongs at the declaration, not at every
// implementor.
#[test]
fn check_a_trait_impl_method_with_a_mut_ref_parameter_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/report.rs",
        "use std::fmt;\n\npub struct Report;\n\nimpl fmt::Display for Report {\n    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {\n        write!(f, \"x\")\n    }\n}\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_an_inherent_method_with_a_mut_ref_parameter_reports_it() {
    // Arrange & Act
    let offences = check(
        "src/sink.rs",
        "pub struct Sink;\n\nimpl Sink {\n    pub fn write(&self, out: &mut String) {}\n}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].subject(), Some("write::out"));
}

// Every offending parameter is named, because each one is a separate edit.
#[test]
fn check_two_mut_ref_parameters_reports_both() {
    // Arrange & Act
    let offences = check(
        "src/merge.rs",
        "pub fn merge(a: &mut u32, b: &mut u32) {}\n",
    );

    // Assert
    assert_eq!(offences.len(), 2);
    assert_eq!(offences[0].subject(), Some("merge::a"));
    assert_eq!(offences[1].subject(), Some("merge::b"));
}

// This rule answers the per-file question only, and says so rather than
// inheriting a silence nobody chose.
#[test]
fn check_workspace_of_a_tree_with_a_mut_ref_reports_nothing() {
    // Arrange
    let file = SourceFile::new("src/apply.rs", "pub fn apply(target: &mut u32) {}\n");

    // Act
    let offences = ReturnedMutationsRule::new().check_workspace(&[file]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn is_configured_for_a_rule_that_needs_nothing_returns_true() {
    // Arrange & Act
    let configured = ReturnedMutationsRule::new().is_configured();

    // Assert
    assert!(configured);
}

#[test]
fn name_is_the_kebab_case_rule_name_used_in_the_report() {
    // Arrange & Act
    let name = ReturnedMutationsRule::new().name();

    // Assert
    assert_eq!(name, RULE);
}

// A rule that is always configured has nothing to ask for, and says so rather
// than inventing a requirement it does not have.
#[test]
fn requirement_of_a_rule_that_needs_nothing_is_none() {
    // Arrange & Act
    let requirement = ReturnedMutationsRule::new().requirement();

    // Assert
    assert!(requirement.is_none());
}
