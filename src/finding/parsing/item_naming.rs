// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use syn::Item;

use crate::source_file::SourceFile;

// The identifier an item declares, and the source line to fall back on when it
// declares none.
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

    // The line as written, because that is what a reader would point at.
    pub fn source_line(file: &SourceFile, line: usize) -> String {
        file.lines()
            .get(line.saturating_sub(1))
            .map(|text| text.trim().to_string())
            .unwrap_or_default()
    }
}
