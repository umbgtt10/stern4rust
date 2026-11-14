// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use proc_macro2::Delimiter;
use proc_macro2::Group;
use proc_macro2::TokenStream;
use proc_macro2::TokenTree;
use syn::Block;
use syn::Expr;
use syn::Macro;
use syn::Token;
use syn::parse2;
use syn::punctuated::Punctuated;

use crate::finding::parsing::shapes::standard_macro::StandardMacro;

// The body of a macro outside the standard family, read as code where it reads
// as code: a list of expressions, as `prop_assert_eq!` and `prop_oneof!` hold;
// else the brace groups that parse as blocks, as a `proptest!` test's body and
// an arm of `tokio::select!` are, searched for inside any group that is not
// one. Where nothing parses, nothing is judged.
//
// Two kinds of macro are never read, since their body is not code the file
// wrote: `quote!` and its kind, which spell a generated path in full on
// purpose -- often as `::std::...` -- and the compiler's built-ins whose body is
// tokens or text, `stringify!` and `include_str!` among them.
pub struct MacroBody<'a> {
    node: &'a Macro,
}

impl<'a> MacroBody<'a> {
    const NOT_CODE: [&'static str; 22] = [
        "asm",
        "cfg",
        "column",
        "compile_error",
        "concat",
        "env",
        "file",
        "format_ident",
        "global_asm",
        "include",
        "include_bytes",
        "include_str",
        "line",
        "macro_rules",
        "module_path",
        "naked_asm",
        "option_env",
        "parse_quote",
        "parse_quote_spanned",
        "quote",
        "quote_spanned",
        "stringify",
    ];

    pub fn of(node: &'a Macro) -> Option<Self> {
        let name = node.path.segments.last()?.ident.to_string();
        let read = StandardMacro::of(node).is_none() && !Self::NOT_CODE.contains(&name.as_str());
        read.then_some(Self { node })
    }

    // The brace groups of its body that parse as blocks, in the order written.
    pub fn blocks(&self) -> Vec<Block> {
        Self::blocks_in(self.node.tokens.clone())
    }

    // Its body as a list of expressions, when it is one.
    pub fn expressions(&self) -> Option<Vec<Expr>> {
        self.node
            .parse_body_with(Punctuated::<Expr, Token![,]>::parse_terminated)
            .map(|listed| listed.into_iter().collect())
            .ok()
    }

    fn blocks_in(tokens: TokenStream) -> Vec<Block> {
        tokens
            .into_iter()
            .flat_map(|tree| match tree {
                TokenTree::Group(group) => Self::blocks_of(&group),
                _ => Vec::new(),
            })
            .collect()
    }

    fn blocks_of(group: &Group) -> Vec<Block> {
        let whole = TokenStream::from(TokenTree::Group(group.clone()));
        match parse2::<Block>(whole) {
            Ok(block) if group.delimiter() == Delimiter::Brace => vec![block],
            _ => Self::blocks_in(group.stream()),
        }
    }
}
