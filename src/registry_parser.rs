// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use syn::Item;
use syn::spanned::Spanned;

use crate::registry_item::RegistryItem;
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
    pub fn strays(file: &SourceFile) -> Option<Vec<RegistryItem>> {
        let syntax = syn::parse_file(&file.contents()).ok()?;
        Some(
            syntax
                .items
                .iter()
                .filter(|item| !Self::is_declaration(item))
                .map(|item| Self::stray(file, item))
                .collect(),
        )
    }

    // A declaration points at a file, whether or not it is `pub` -- a private
    // `mod name;` compiles that file just as well, and being compiled is the
    // whole point. A module with a body declares nothing: it is code hiding in
    // the one file a reader scans expecting a list.
    fn is_declaration(item: &Item) -> bool {
        matches!(item, Item::Mod(module) if module.content.is_none())
    }

    fn stray(file: &SourceFile, item: &Item) -> RegistryItem {
        let line = item.span().start().line;
        RegistryItem::new(line, &Self::label(file, item, line))
    }

    // Named by identifier wherever there is one, because "remove the constant
    // LIMIT" is a whole instruction and "remove the item on line 5" is half of
    // one. The two kinds without an identifier fall back to the line as
    // written, which is what a reader would search the file for anyway.
    fn label(file: &SourceFile, item: &Item, line: usize) -> String {
        match item {
            Item::Const(inner) => format!("the constant `{}`", inner.ident),
            Item::Enum(inner) => format!("the enum `{}`", inner.ident),
            Item::Fn(inner) => format!("the function `{}`", inner.sig.ident),
            Item::Impl(_) => format!("the impl block `{}`", Self::source_line(file, line)),
            Item::Mod(inner) => format!("the inline module `{}`", inner.ident),
            Item::Static(inner) => format!("the static `{}`", inner.ident),
            Item::Struct(inner) => format!("the struct `{}`", inner.ident),
            Item::Trait(inner) => format!("the trait `{}`", inner.ident),
            Item::Type(inner) => format!("the type alias `{}`", inner.ident),
            Item::Use(_) => format!("the import `{}`", Self::source_line(file, line)),
            _ => format!("`{}`", Self::source_line(file, line)),
        }
    }

    fn source_line(file: &SourceFile, line: usize) -> String {
        file.lines()
            .get(line.saturating_sub(1))
            .map_or_else(String::new, |text| text.trim().to_string())
    }
}
