// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::blank_lines::BlankLines;
use crate::finding::model::layout::impl_layout::ImplLayout;
use crate::finding::model::layout::member_kind::MemberKind;
use crate::finding::parsing::impl_layout_parser::ImplLayoutParser;
use crate::rules::source::ordered_impl_functions_rule::OrderedImplFunctionsRule;
use crate::source_file::SourceFile;
use crate::source_rewriter::SourceRewriter;

// Every inherent impl block out of order put back in order.
//
// Safe for the reason the other fixers are: it never reads the text. syn says
// where each member begins and ends -- its attributes, doc comments and the
// comment directly above it included -- and whole line ranges are moved without
// being inspected. Only the lines between a block's braces change: members are
// one blank line apart, constants and types none, a comment before the first
// member stays first and one after the last stays last. A block the parser says
// cannot be moved as lines is left exactly as it is, and its offence says why.
pub struct OrderedImplFunctionsRewriter;

impl OrderedImplFunctionsRewriter {
    pub const SOURCE_ROOT: &'static str = "src/";

    pub fn new() -> Self {
        Self
    }

    fn body(lines: &[String], layout: &ImplLayout) -> Vec<String> {
        let (Some(first), Some(last)) = (layout.members().first(), layout.members().last()) else {
            return Vec::new();
        };
        let leading =
            BlankLines::without_trailing(&lines[layout.open_line()..first.first_line() - 1]);
        let trailing =
            BlankLines::without_leading(&lines[last.last_line()..layout.close_line() - 1]);
        let mut body = leading.clone();
        if !leading.is_empty() {
            body.push(String::new());
        }
        let mut previous: Option<MemberKind> = None;
        for member in layout.ordered() {
            if previous
                .is_some_and(|kind| kind != MemberKind::Head || member.kind() != MemberKind::Head)
            {
                body.push(String::new());
            }
            body.extend_from_slice(&lines[member.first_line() - 1..member.last_line()]);
            previous = Some(member.kind());
        }
        if !trailing.is_empty() {
            body.push(String::new());
            body.extend(trailing);
        }
        body
    }
}

impl Default for OrderedImplFunctionsRewriter {
    fn default() -> Self {
        Self::new()
    }
}

impl SourceRewriter for OrderedImplFunctionsRewriter {
    fn rule(&self) -> &'static str {
        OrderedImplFunctionsRule::NAME
    }

    // Blocks are replaced from the last to the first, so each replacement leaves
    // the lines of the blocks above it where the parser found them.
    fn rewrite(&self, file: &SourceFile) -> Option<String> {
        if !file.relative_path().starts_with(Self::SOURCE_ROOT) {
            return None;
        }
        let mut fixable: Vec<ImplLayout> = ImplLayoutParser::parse(file)
            .into_iter()
            .filter(|layout| layout.refusal().is_none() && layout.first_misplaced().is_some())
            .collect();
        if fixable.is_empty() {
            return None;
        }
        fixable.sort_by_key(|layout| layout.open_line());
        let mut lines = file.lines().to_vec();
        for layout in fixable.iter().rev() {
            let body = Self::body(file.lines(), layout);
            lines.splice(layout.open_line()..layout.close_line() - 1, body);
        }
        Some(lines.join("\n"))
    }
}
