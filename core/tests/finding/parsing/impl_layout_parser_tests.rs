// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Each inherent impl block of a file, its braces, and its members as written:
// constants and types, constructors -- no receiver, a return type naming `Self`
// or the impl's type, directly or wrapped -- `pub` functions and the rest, each
// with the lines it would move with. And whether --fix could reorder it.

use stern4rust::finding::model::layout::fix_refusal::FixRefusal;
use stern4rust::finding::model::layout::impl_layout::ImplLayout;
use stern4rust::finding::model::layout::member_kind::MemberKind;
use stern4rust::finding::parsing::impl_layout_parser::ImplLayoutParser;
use stern4rust::source_file::SourceFile;

fn kinds(body: &str) -> Vec<(String, MemberKind)> {
    single(body)
        .members
        .into_iter()
        .map(|member| (member.name, member.kind))
        .collect()
}

fn single(body: &str) -> ImplLayout {
    let mut layouts = ImplLayoutParser::parse(&SourceFile::new(
        "src/store.rs",
        &format!("impl Store {{\n{body}}}\n"),
    ));
    assert_eq!(layouts.len(), 1, "expected one impl block");
    layouts.remove(0)
}

#[test]
fn parse_a_comment_after_the_last_member_does_not_refuse_the_fix() {
    // Arrange & Act
    let layout = single("    fn b() {}\n\n    fn a() {}\n\n    // the end\n");

    // Assert
    assert_eq!(layout.refusal, None);
}

#[test]
fn parse_a_constructor_is_found_through_result_option_and_a_tuple() {
    // Arrange & Act
    let found = kinds(
        "    fn plain() -> Self {\n        Self\n    }\n\n    fn parsed() -> Result<Self, String> {\n        \
         Ok(Self)\n    }\n\n    fn maybe() -> Option<Store> {\n        None\n    }\n\n    \
         fn with_outcomes() -> (Self, Vec<u8>) {\n        (Self, vec![])\n    }\n",
    );

    // Assert
    assert!(
        found
            .iter()
            .all(|(_, kind)| *kind == MemberKind::Constructor),
        "got {found:?}"
    );
}

#[test]
fn parse_a_file_that_does_not_parse_finds_nothing() {
    // Arrange & Act
    let layouts = ImplLayoutParser::parse(&SourceFile::new("src/store.rs", "impl Store {\n"));

    // Assert
    assert!(layouts.is_empty());
}

#[test]
fn parse_a_free_comment_between_functions_refuses_the_fix() {
    // Arrange & Act
    let layout = single("    fn b() {}\n\n    // -- queries --\n\n    fn a() {}\n");

    // Assert
    assert_eq!(layout.refusal, Some(FixRefusal::FreeComment));
}

#[test]
fn parse_a_macro_among_the_members_refuses_the_fix() {
    // Arrange & Act
    let layout = single("    fn b() {}\n\n    accessors!();\n\n    fn a() {}\n");

    // Assert
    assert_eq!(layout.refusal, Some(FixRefusal::MacroInvocation));
}

#[test]
fn parse_a_member_spans_its_attributes_doc_and_comment() {
    // Arrange & Act
    let layout =
        single("    fn b() {}\n\n    // Why.\n    /// What.\n    #[inline]\n    fn a() {}\n");

    // Assert
    assert_eq!(layout.members[1].first_line, 4);
    assert_eq!(layout.members[1].last_line, 7);
}

#[test]
fn parse_a_method_returning_self_is_not_a_constructor() {
    // Arrange & Act
    let found = kinds("    fn renamed(self) -> Self {\n        self\n    }\n");

    // Assert
    assert_eq!(found, vec![(String::from("renamed"), MemberKind::Other)]);
}

#[test]
fn parse_a_private_constructor_is_a_constructor() {
    // Arrange & Act
    let found = kinds("    fn empty() -> Self {\n        Self\n    }\n");

    // Assert
    assert_eq!(
        found,
        vec![(String::from("empty"), MemberKind::Constructor)]
    );
}

#[test]
fn parse_a_trait_impl_is_left_out() {
    // Arrange & Act
    let layouts = ImplLayoutParser::parse(&SourceFile::new(
        "src/store.rs",
        "impl Default for Store {\n    fn default() -> Self {\n        Self\n    }\n}\n",
    ));

    // Assert
    assert!(layouts.is_empty());
}

#[test]
fn parse_an_impl_names_its_type_line_and_braces() {
    // Arrange & Act
    let layouts = ImplLayoutParser::parse(&SourceFile::new(
        "src/store.rs",
        "use std::fmt;\n\nimpl<T> Store<T> {\n    fn a() {}\n}\n",
    ));

    // Assert
    assert_eq!(layouts[0].owner, "Store");
    assert_eq!(layouts[0].line, 3);
    assert_eq!(layouts[0].open_line, 3);
    assert_eq!(layouts[0].close_line, 5);
}

#[test]
fn parse_constants_and_types_are_heads_and_pub_variants_are_pub() {
    // Arrange & Act
    let found = kinds(
        "    pub const LIMIT: u8 = 3;\n    type Unit = u8;\n    pub const fn a() {}\n    pub async fn b() {}\n    \
         pub unsafe fn c() {}\n    pub(crate) fn d() {}\n",
    );

    // Assert
    let kinds: Vec<MemberKind> = found.into_iter().map(|(_, kind)| kind).collect();
    assert_eq!(
        kinds,
        vec![
            MemberKind::Head,
            MemberKind::Head,
            MemberKind::Public,
            MemberKind::Public,
            MemberKind::Public,
            MemberKind::Other,
        ]
    );
}

#[test]
fn parse_counts_a_constructors_parameters() {
    // Arrange & Act
    let layout = single("    fn from_parts(a: u8, b: u8) -> Self {\n        Self\n    }\n");

    // Assert
    assert_eq!(layout.members[0].parameters, 2);
}

// A raw identifier sorts as the word it spells.
#[test]
fn parse_names_a_raw_identifier_without_its_prefix() {
    // Arrange & Act
    let found = kinds("    pub fn r#match(&self) {}\n");

    // Assert
    assert_eq!(found[0].0, "match");
}

#[test]
fn parse_two_members_on_one_line_refuses_the_fix() {
    // Arrange & Act
    let layout = single("    fn b() {} fn a() {}\n");

    // Assert
    assert_eq!(layout.refusal, Some(FixRefusal::SharedLine));
}
