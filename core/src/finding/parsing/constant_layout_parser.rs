// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use syn::Ident;
use syn::Item;
use syn::ItemMacro;
use syn::parse_file;
use syn::spanned::Spanned;

use crate::finding::model::layout::constant_layout::ConstantLayout;
use crate::finding::model::layout::fix_refusal::FixRefusal;
use crate::finding::model::layout::misplaced_constant::MisplacedConstant;
use crate::finding::parsing::item_naming::ItemNaming;
use crate::source_file::SourceFile;

// A file's module-level constants and statics declared after its first item that
// is neither an import nor a constant -- the leader -- and where they would go.
//
// Imports, `extern crate` and `mod` declarations are not items here: their order
// is rustfmt's and ordered-imports'. An inline module is an item like any other,
// and what it holds is not looked at.
//
// Each item's lines are its span, attributes included, extended over the comment
// directly above it the way the test-file parser reads a block. Whether --fix may
// move them is decided here too, because it is a fact about the file: a
// `macro_rules!` makes textual order matter, a constant sharing a line with
// another item cannot be moved as lines, and with nothing above the leader a
// comment running from the top of the file into it could be the header or the
// leader's own.
pub struct ConstantLayoutParser;

impl ConstantLayoutParser {
    // None when the file does not parse -- rustc's to report -- or when no
    // constant is out of place.
    pub fn parse(file: &SourceFile) -> Option<ConstantLayout> {
        let items = parse_file(&file.contents()).ok()?.items;
        let leader = items.iter().position(Self::leads)?;
        let firsts = Self::first_lines(file, &items);
        let misplaced: Vec<MisplacedConstant> = (leader + 1..items.len())
            .filter_map(|at| Self::misplaced(&items[at], firsts[at]))
            .collect();
        if misplaced.is_empty() {
            return None;
        }
        Some(ConstantLayout::new(
            Self::label(file, &items[leader]),
            Self::insertion_line(&items, &firsts, leader),
            leader > 0 && Self::constant(&items[leader - 1]).is_some(),
            misplaced,
            Self::refusal(&items, &firsts, leader),
        ))
    }

    fn after_another(items: &[Item], firsts: &[usize], at: usize) -> bool {
        at > 0 && firsts[at] <= Self::last_line(&items[at - 1])
    }

    fn before_another(items: &[Item], firsts: &[usize], at: usize) -> bool {
        firsts
            .get(at + 1)
            .is_some_and(|next| *next <= Self::last_line(&items[at]))
    }

    fn constant(item: &Item) -> Option<&Ident> {
        match item {
            Item::Const(inner) => Some(&inner.ident),
            Item::Static(inner) => Some(&inner.ident),
            _ => None,
        }
    }

    fn first_lines(file: &SourceFile, items: &[Item]) -> Vec<usize> {
        let mut floor = 1;
        items
            .iter()
            .map(|item| {
                let first = file.block_start(item.span().start().line, floor);
                floor = Self::last_line(item) + 1;
                first
            })
            .collect()
    }

    // After the item that ends the constants group, or at the leader's own first
    // line when nothing precedes it.
    fn insertion_line(items: &[Item], firsts: &[usize], leader: usize) -> usize {
        match leader {
            0 => firsts[0],
            _ => Self::last_line(&items[leader - 1]) + 1,
        }
    }

    fn is_import(item: &Item) -> bool {
        match item {
            Item::Use(_) | Item::ExternCrate(_) => true,
            Item::Mod(inner) => inner.content.is_none(),
            _ => false,
        }
    }

    fn is_macro_rules(item: &Item) -> bool {
        matches!(item, Item::Macro(inner) if inner.mac.path.is_ident("macro_rules"))
    }

    // What a reader would look for: a declared name, the type an impl is for,
    // or the macro and the name it defines.
    fn label(file: &SourceFile, item: &Item) -> String {
        match item {
            Item::Impl(inner) => format!("impl {}", ItemNaming::implemented_type(&inner.self_ty)),
            Item::Macro(inner) => Self::macro_label(inner),
            _ => ItemNaming::identifier(item)
                .unwrap_or_else(|| ItemNaming::source_line(file, item.span().start().line)),
        }
    }

    fn last_line(item: &Item) -> usize {
        item.span().end().line
    }

    fn leads(item: &Item) -> bool {
        Self::constant(item).is_none() && !Self::is_import(item)
    }

    fn macro_label(item: &ItemMacro) -> String {
        let path = item
            .mac
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string())
            .unwrap_or_default();
        match &item.ident {
            Some(name) => format!("{path}! {name}"),
            None => format!("{path}!"),
        }
    }

    fn misplaced(item: &Item, first_line: usize) -> Option<MisplacedConstant> {
        Self::constant(item).map(|ident| {
            MisplacedConstant::new(
                ident.to_string(),
                ident.span().start().line,
                first_line,
                Self::last_line(item),
            )
        })
    }

    // The leader is checked against the item above it, since the constants are
    // inserted between the two; a misplaced constant against both its neighbours,
    // since it leaves from between them.
    fn refusal(items: &[Item], firsts: &[usize], leader: usize) -> Option<FixRefusal> {
        if items.iter().any(Self::is_macro_rules) {
            return Some(FixRefusal::MacroRules);
        }
        if leader == 0 && firsts[0] == 1 && items[0].span().start().line > 1 {
            return Some(FixRefusal::OpeningComment);
        }
        let crowded = Self::after_another(items, firsts, leader)
            || (leader + 1..items.len()).any(|at| {
                Self::constant(&items[at]).is_some()
                    && (Self::after_another(items, firsts, at)
                        || Self::before_another(items, firsts, at))
            });
        crowded.then_some(FixRefusal::SharedLine)
    }
}
