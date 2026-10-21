// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The methods of a file's inherent impls written `pub(crate)`, `pub(super)`,
// `pub(self)` or `pub(in ...)`, inline modules included. Trait impls carry no
// visibility of their own and free functions are not methods, so neither is
// looked at.

use stern4rust::finding::model::public_entry_point::PublicEntryPoint;
use stern4rust::finding::parsing::restricted_method_finder::RestrictedMethodFinder;
use syn::parse_file;

fn visibilities(source: &str) -> Vec<String> {
    RestrictedMethodFinder::find(&parse_file(source).expect("parses").items)
        .into_iter()
        .map(|method| method.visibility)
        .collect()
}

#[test]
fn find_a_method_in_an_inline_module_returns_it() {
    // Arrange & Act
    let found = visibilities(
        "mod detail {\n    impl Store {\n        pub(super) fn grow(&self) {}\n    }\n}\n",
    );

    // Assert
    assert_eq!(found, vec!["pub(super)"]);
}

#[test]
fn find_a_pub_crate_method_returns_its_call_line_visibility_and_owner() {
    // Arrange
    let syntax = parse_file(
        "impl RaftProtocol {\n    pub(crate) fn election(&mut self, term: u64) -> u64 {\n        term\n    }\n}\n",
    )
    .expect("parses");

    // Act
    let found = RestrictedMethodFinder::find(&syntax.items);

    // Assert
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].call, PublicEntryPoint::new("election", 1));
    assert_eq!(found[0].line, 2);
    assert_eq!(found[0].visibility, "pub(crate)");
    assert_eq!(found[0].owner, "RaftProtocol");
}

#[test]
fn find_a_pub_or_private_method_returns_nothing() {
    // Arrange & Act
    let found = visibilities(
        "impl Store {\n    pub fn len(&self) -> usize {\n        0\n    }\n\n    fn grow(&self) {}\n}\n",
    );

    // Assert
    assert!(found.is_empty(), "got {found:?}");
}

#[test]
fn find_a_restricted_free_function_returns_nothing() {
    // Arrange & Act
    let found = visibilities("pub(crate) fn helper() {}\n");

    // Assert
    assert!(found.is_empty(), "got {found:?}");
}

#[test]
fn find_each_restricted_form_returns_it_as_written() {
    // Arrange & Act
    let found = visibilities(
        "impl Store {\n    pub(crate) fn a(&self) {}\n    pub(super) fn b(&self) {}\n    \
         pub(self) fn c(&self) {}\n    pub(in crate::store) fn d(&self) {}\n}\n",
    );

    // Assert
    assert_eq!(
        found,
        vec![
            "pub(crate)",
            "pub(super)",
            "pub(self)",
            "pub(in crate::store)"
        ]
    );
}

// The language refuses visibility here, but a parser accepts it; the trait's
// signature is what decides who can call the method.
#[test]
fn find_in_a_trait_impl_returns_nothing() {
    // Arrange & Act
    let found = visibilities(
        "impl Shape for Square {\n    pub(crate) fn area(&self) -> u32 {\n        1\n    }\n}\n",
    );

    // Assert
    assert!(found.is_empty(), "got {found:?}");
}
