// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// An inherent impl block lists constructors first, fewest parameters first,
// then its `pub` functions, then the rest, each alphabetical, with constants and
// types ahead of them all. One offence per block, at the impl, naming the first
// member out of place; trait impls are left alone and tests/ is exempt.

use stern4rust::reporting::offence::Offence;
use stern4rust::rule::Rule;
use stern4rust::rules::source::ordered_impl_functions_rule::OrderedImplFunctionsRule;
use stern4rust::source_file::SourceFile;

const RULE: &str = "ordered-impl-functions";

fn check(path: &str, contents: &str) -> Vec<Offence> {
    OrderedImplFunctionsRule::new().check(&SourceFile::new(path, contents))
}

fn impl_of(body: &str) -> String {
    format!("pub struct Store;\n\nimpl Store {{\n{body}}}\n")
}

#[test]
fn check_a_block_in_order_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/store.rs",
        &impl_of(
            "    const LIMIT: u8 = 3;\n\n    pub fn new() -> Self {\n        Self\n    }\n\n    \
             pub fn with_capacity(size: u8) -> Self {\n        Self\n    }\n\n    pub fn len(&self) -> u8 {\n        \
             0\n    }\n\n    fn grow(&self) {}\n",
        ),
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_a_block_out_of_order_names_the_first_member_misplaced() {
    // Arrange & Act
    let offences = check(
        "src/bookkeepers.rs",
        "pub struct Bookkeepers;\n\nimpl Bookkeepers {\n    pub fn new() -> Self {\n        Self\n    }\n\n    \
         pub fn cluster(&self) {}\n\n    pub fn consensus(&self) {}\n\n    pub fn catch_up(&self) {}\n}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].rule(), RULE);
    assert_eq!(offences[0].line(), 3);
    assert_eq!(offences[0].subject(), Some("Bookkeepers"));
    assert_eq!(
        offences[0].description(),
        "`impl Bookkeepers` is out of order: `catch_up` belongs before `cluster`"
    );
}

#[test]
fn check_a_block_the_fix_can_repair_offers_it() {
    // Arrange & Act
    let offences = check("src/store.rs", &impl_of("    fn b() {}\n\n    fn a() {}\n"));

    // Assert
    assert!(
        offences[0].correction().contains("`--fix`")
            && !offences[0].correction().contains("by hand"),
        "got {:?}",
        offences[0].correction()
    );
}

#[test]
fn check_a_block_the_fix_declines_says_why() {
    // Arrange & Act
    let offences = check(
        "src/store.rs",
        &impl_of("    fn b() {}\n\n    // -- queries --\n\n    fn a() {}\n"),
    );

    // Assert
    assert!(
        offences[0].correction().contains("by hand")
            && offences[0].correction().contains("comment"),
        "got {:?}",
        offences[0].correction()
    );
}

#[test]
fn check_a_constant_after_a_function_reports_it() {
    // Arrange & Act
    let offences = check(
        "src/store.rs",
        &impl_of("    fn a() {}\n\n    const LIMIT: u8 = 3;\n"),
    );

    // Assert
    assert_eq!(
        offences[0].description(),
        "`impl Store` is out of order: `LIMIT` belongs before `a`"
    );
}

#[test]
fn check_a_file_that_does_not_parse_reports_nothing() {
    // Arrange & Act
    let offences = check("src/store.rs", "impl Store {\n    fn b() {}\n");

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_a_private_function_before_a_pub_one_reports_it() {
    // Arrange & Act
    let offences = check(
        "src/store.rs",
        &impl_of("    fn a() {}\n\n    pub fn b() {}\n"),
    );

    // Assert
    assert_eq!(
        offences[0].description(),
        "`impl Store` is out of order: `b` belongs before `a`"
    );
}

#[test]
fn check_a_test_file_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "tests/store_tests.rs",
        &impl_of("    fn b() {}\n\n    fn a() {}\n"),
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_a_trait_impl_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/store.rs",
        "pub struct Store;\n\nimpl Iterator for Store {\n    type Item = u8;\n\n    fn next(&mut self) -> Option<u8> {\n        \
         None\n    }\n\n    fn count(self) -> usize {\n        0\n    }\n}\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// `cfg`-gated twins share a name, and stay adjacent in the order written.
#[test]
fn check_cfg_gated_twins_report_nothing() {
    // Arrange & Act
    let offences = check(
        "src/store.rs",
        &impl_of(
            "    #[cfg(unix)]\n    pub fn path(&self) {}\n\n    #[cfg(windows)]\n    pub fn path(&self) {}\n\n    \
             pub fn size(&self) {}\n",
        ),
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_constructors_out_of_parameter_order_report_it() {
    // Arrange & Act
    let offences = check(
        "src/store.rs",
        &impl_of(
            "    pub fn with_capacity(size: u8) -> Self {\n        Self\n    }\n\n    pub fn new() -> Self {\n        \
             Self\n    }\n",
        ),
    );

    // Assert
    assert_eq!(
        offences[0].description(),
        "`impl Store` is out of order: `new` belongs before `with_capacity`"
    );
}

#[test]
fn check_reports_each_block_out_of_order_once() {
    // Arrange & Act
    let offences = check(
        "src/store.rs",
        "pub struct Store;\n\nimpl Store {\n    fn c() {}\n\n    fn b() {}\n\n    fn a() {}\n}\n\n\
         pub struct Shelf;\n\nimpl Shelf {\n    fn b() {}\n\n    fn a() {}\n}\n",
    );

    // Assert
    let subjects: Vec<Option<&str>> = offences.iter().map(|offence| offence.subject()).collect();
    assert_eq!(subjects, vec![Some("Store"), Some("Shelf")]);
}

// This rule answers the per-file question only, and says so rather than
// inheriting a silence nobody chose.
#[test]
fn check_workspace_of_a_tree_with_a_block_out_of_order_reports_nothing() {
    // Arrange
    let file = SourceFile::new("src/store.rs", &impl_of("    fn b() {}\n\n    fn a() {}\n"));

    // Act
    let offences = OrderedImplFunctionsRule::new().check_workspace(&[file]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn is_configured_for_a_rule_that_needs_nothing_returns_true() {
    // Arrange & Act
    let configured = OrderedImplFunctionsRule::new().is_configured();

    // Assert
    assert!(configured);
}

#[test]
fn name_is_the_kebab_case_rule_name_used_in_the_report() {
    // Arrange & Act
    let name = OrderedImplFunctionsRule::new().name();

    // Assert
    assert_eq!(name, RULE);
}

// A rule that is always configured has nothing to ask for, and says so rather
// than inventing a requirement it does not have.
#[test]
fn requirement_of_a_rule_that_needs_nothing_is_none() {
    // Arrange & Act
    let requirement = OrderedImplFunctionsRule::new().requirement();

    // Assert
    assert!(requirement.is_none());
}
