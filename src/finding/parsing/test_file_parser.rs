// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use syn::Item;
use syn::parse_file;
use syn::spanned::Spanned;

use crate::finding::model::section::Section;
use crate::finding::model::test_file_item::TestFileItem;
use crate::finding::parsing::item_naming::ItemNaming;
use crate::source_file::SourceFile;

// Turns a test file into the list of items the structure rule reasons about.
//
// Plain `//` comments never reach the syntax tree, so the header and every
// explanatory comment are invisible here -- which is why the rule does not need
// to know where the header ends. What it does need is for a comment introducing
// an item to count as part of that item, so each block is extended upwards over
// the comment lines directly above it.
pub struct TestFileParser;

impl TestFileParser {
    // None means the file does not parse. That is rustc's to report, far more
    // clearly than this could, and guessing at a shape from broken source would
    // pile noise on top of a compile error.
    pub fn parse(file: &SourceFile) -> Option<Vec<TestFileItem>> {
        let syntax = parse_file(&file.contents()).ok()?;
        Some(
            syntax
                .items
                .iter()
                .map(|item| Self::item(file, item))
                .collect(),
        )
    }

    fn item(file: &SourceFile, item: &Item) -> TestFileItem {
        let span = item.span();
        let first_line = Self::with_leading_comments(file, span.start().line);
        TestFileItem::new(
            Self::section(item),
            Self::name(file, item, span.start().line),
            first_line,
            span.end().line,
        )
    }

    fn section(item: &Item) -> Section {
        match item {
            Item::Use(_) => Section::Imports,
            Item::Const(_) | Item::Static(_) => Section::Constants,
            Item::Fn(function) if Self::is_test(&function.attrs) => Section::Tests,
            _ => Section::Helpers,
        }
    }

    // `#[test]`, `#[tokio::test]` and any other harness spelled the same way.
    // Matching the last path segment rather than the whole path is what keeps
    // this from having to enumerate test frameworks.
    fn is_test(attrs: &[syn::Attribute]) -> bool {
        attrs.iter().any(|attr| {
            attr.path()
                .segments
                .last()
                .is_some_and(|segment| segment.ident == "test")
        })
    }

    fn name(file: &SourceFile, item: &Item, start_line: usize) -> String {
        // An impl block belongs to the type it implements, so it sorts under
        // that name and sits beside the struct rather than drifting to the end
        // of the section. That is this parser's own answer; the rest is shared.
        if let Item::Impl(inner) = item {
            return Self::type_name(&inner.self_ty);
        }
        ItemNaming::identifier(item).unwrap_or_else(|| ItemNaming::source_line(file, start_line))
    }

    fn type_name(ty: &syn::Type) -> String {
        match ty {
            syn::Type::Path(path) => path
                .path
                .segments
                .last()
                .map_or_else(String::new, |segment| segment.ident.to_string()),
            _ => String::new(),
        }
    }

    // An import sorts and reports as it was written, which is how rustfmt orders
    // them and how a reader would look for one.

    fn with_leading_comments(file: &SourceFile, start_line: usize) -> usize {
        let mut first = start_line;
        while first > 1 && Self::is_comment(file, first - 1) {
            first -= 1;
        }
        first
    }

    fn is_comment(file: &SourceFile, line: usize) -> bool {
        file.lines()
            .get(line.saturating_sub(1))
            .is_some_and(|text| text.trim_start().starts_with("//"))
    }
}
