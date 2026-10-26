// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use syn::Item;
use syn::parse_file;
use syn::spanned::Spanned;

use crate::finding::model::layout::section::Section;
use crate::finding::model::layout::test_file_item::TestFileItem;
use crate::finding::parsing::item_naming::ItemNaming;
use crate::source_file::SourceFile;
use syn::Attribute;

// Turns a test file into the list of items the structure rule reasons about.
//
// Plain `//` comments never reach the syntax tree, so the header and every
// explanatory comment are invisible here -- which is why the rule does not need
// to know where the header ends. What it does need is for a comment introducing
// an item to count as part of that item, so each block is extended upwards over
// the comment lines directly above it -- but never past the item above, whose
// own lines can look like comments: a string literal's continuation lines may
// start with `//`.
pub struct TestFileParser;

impl TestFileParser {
    // None means the file does not parse. That is rustc's to report, far more
    // clearly than this could, and guessing at a shape from broken source would
    // pile noise on top of a compile error.
    pub fn parse(file: &SourceFile) -> Option<Vec<TestFileItem>> {
        let syntax = parse_file(&file.contents()).ok()?;
        let mut floor = 1;
        Some(
            syntax
                .items
                .iter()
                .map(|item| {
                    let parsed = Self::item(file, item, floor);
                    floor = parsed.last_line() + 1;
                    parsed
                })
                .collect(),
        )
    }

    // `#[test]`, `#[tokio::test]` and any other harness spelled the same way.
    // Matching the last path segment rather than the whole path is what keeps
    // this from having to enumerate test frameworks.
    fn is_test(attrs: &[Attribute]) -> bool {
        attrs.iter().any(|attr| {
            attr.path()
                .segments
                .last()
                .is_some_and(|segment| segment.ident == "test")
        })
    }

    fn item(file: &SourceFile, item: &Item, floor: usize) -> TestFileItem {
        let span = item.span();
        let first_line = file.block_start(span.start().line, floor);
        TestFileItem::new(
            Self::section(item),
            Self::name(file, item, span.start().line),
            first_line,
            span.end().line,
        )
    }

    fn name(file: &SourceFile, item: &Item, start_line: usize) -> String {
        // An impl block belongs to the type it implements, so it sorts under
        // that name and sits beside the struct rather than drifting to the end
        // of the section. That is this parser's own answer; the rest is shared.
        if let Item::Impl(inner) = item {
            return ItemNaming::implemented_type(&inner.self_ty);
        }
        // An import by its `use` line, as rustfmt sorts it and a reader looks
        // for it -- not by the attribute its span opens on.
        if let Item::Use(import) = item {
            return ItemNaming::import_line(file, import);
        }
        ItemNaming::identifier(item).unwrap_or_else(|| ItemNaming::source_line(file, start_line))
    }

    fn section(item: &Item) -> Section {
        match item {
            Item::Use(_) => Section::Imports,
            Item::Const(_) | Item::Static(_) => Section::Constants,
            Item::Fn(function) if Self::is_test(&function.attrs) => Section::Tests,
            _ => Section::Helpers,
        }
    }
}
