// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use syn::Item;
use syn::ItemUse;
use syn::Type;
use syn::spanned::Spanned;

use crate::source_file::SourceFile;

// The identifier an item declares, and the source line to fall back on when it
// declares none -- for an import, the line its `use` is on, which both import
// rules read.
//
// `TestFileParser::name`, `RegistryParser::label` and `UnitTestFinder::describe`
// each carried both, with the same arms and the same fallback --
// `OPEN_POINTS.md` recorded it as "three copies drift" and asked for the
// extraction to be deliberate rather than incidental to a bug fix.
//
// Only the shared half moved. What the three legitimately differ on is the
// **wording around the name** -- `the constant `X``, `constant `X``, or a bare
// `X` -- and that stays with each of them. Sharing it would have meant one
// caller changing an offence description to match another, which is a breaking
// change for every baseline holding that description.
pub struct ItemNaming;

impl ItemNaming {
    // None for an item that declares no name of its own: an impl block belongs
    // to the type it implements, and an import names nothing. Each caller says
    // what it wants about those.
    pub fn identifier(item: &Item) -> Option<String> {
        match item {
            Item::Const(inner) => Some(inner.ident.to_string()),
            Item::Enum(inner) => Some(inner.ident.to_string()),
            Item::Fn(inner) => Some(inner.sig.ident.to_string()),
            Item::Mod(inner) => Some(inner.ident.to_string()),
            Item::Static(inner) => Some(inner.ident.to_string()),
            Item::Struct(inner) => Some(inner.ident.to_string()),
            Item::Trait(inner) => Some(inner.ident.to_string()),
            Item::Type(inner) => Some(inner.ident.to_string()),
            Item::Union(inner) => Some(inner.ident.to_string()),
            _ => None,
        }
    }

    // The type an impl block implements, by the last segment of its path and
    // without its arguments, so `impl<T> crate::store::Store<T>` is `Store`.
    // Empty for a type that is not a path, which no reader could look up by name.
    pub fn implemented_type(ty: &Type) -> String {
        match ty {
            Type::Path(path) => path
                .path
                .segments
                .last()
                .map_or_else(String::new, |segment| segment.ident.to_string()),
            _ => String::new(),
        }
    }

    // An import as written, taken from the line rather than rebuilt from the
    // syntax tree, so an offence quotes what the reader will search for.
    //
    // Anchored on the `use` keyword, not on the item. An item's span begins at
    // its first attribute, so a gated import read from `span().start()` was
    // judged -- and quoted -- by its `#[cfg(...)]` line instead of its path.
    //
    // The corrections that produced were not merely noisy, they were wrong.
    // `embassy-logging` gates four imports on three features, and following
    // every correction would have ordered them by feature name: `rtt_sink`
    // above `qemu_sink` above `host_sink`, the paths in reverse, by the rule
    // that exists to alphabetise them. It cut both ways -- two imports genuinely
    // out of order went unreported because their attributes happened to sort.
    // `ordered-imports` was fixed for it in 0.10.3; `test-file-structure` kept
    // reading the attribute until iceberg4rust's gated imports met `cargo fmt`,
    // which sorts them by path, and no order could satisfy both.
    pub fn import_line(file: &SourceFile, import: &ItemUse) -> String {
        Self::source_line(file, import.use_token.span().start().line)
    }

    // The line as written, because that is what a reader would point at.
    pub fn source_line(file: &SourceFile, line: usize) -> String {
        file.lines()
            .get(line.saturating_sub(1))
            .map(|text| text.trim().to_string())
            .unwrap_or_default()
    }
}
