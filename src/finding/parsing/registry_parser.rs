// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use syn::Item;
use syn::parse_file;
use syn::spanned::Spanned;

use crate::finding::model::registry_item::RegistryItem;
use crate::finding::model::registry_policy::RegistryPolicy;
use crate::finding::parsing::item_naming::ItemNaming;
use crate::source_file::SourceFile;

// Finds what does not belong in a registry file, and names it.
//
// A registry -- `tests/all_tests.rs` or any `mod.rs` below it -- holds the
// header and `pub mod` declarations, nothing else. Everything else is a stray,
// and this is where each one is turned into a line and a name so the rule can
// say which thing to remove rather than only that something is wrong.
pub struct RegistryParser;

impl RegistryParser {
    // None means the file does not parse. That is rustc's to report, far more
    // clearly than this could.
    pub fn strays(file: &SourceFile, policy: RegistryPolicy) -> Option<Vec<RegistryItem>> {
        let syntax = parse_file(&file.contents()).ok()?;
        Some(
            syntax
                .items
                .iter()
                .filter(|item| !policy.is_declaration(item))
                .map(|item| Self::stray(file, item))
                .collect(),
        )
    }

    fn stray(file: &SourceFile, item: &Item) -> RegistryItem {
        let line = item.span().start().line;
        RegistryItem::new(line, &Self::label(file, item, line))
    }

    // Named by identifier wherever there is one, because "remove the constant
    // LIMIT" is a whole instruction and "remove the item on line 5" is half of
    // one. The two kinds without an identifier fall back to the line as
    // written, which is what a reader would search the file for anyway.
    // The kind words stay here rather than moving to `ItemNaming`: they are this
    // parser's wording, and every one of them is inside an offence description
    // that baselines are keyed on.
    fn label(file: &SourceFile, item: &Item, line: usize) -> String {
        let Some(kind) = Self::kind(item) else {
            return format!("`{}`", ItemNaming::source_line(file, line));
        };
        let subject =
            ItemNaming::identifier(item).unwrap_or_else(|| ItemNaming::source_line(file, line));
        format!("the {kind} `{subject}`")
    }

    fn kind(item: &Item) -> Option<&'static str> {
        match item {
            Item::Const(_) => Some("constant"),
            Item::Enum(_) => Some("enum"),
            Item::Fn(_) => Some("function"),
            Item::Impl(_) => Some("impl block"),
            Item::Mod(_) => Some("inline module"),
            Item::Static(_) => Some("static"),
            Item::Struct(_) => Some("struct"),
            Item::Trait(_) => Some("trait"),
            Item::Type(_) => Some("type alias"),
            Item::Use(_) => Some("import"),
            _ => None,
        }
    }
}
