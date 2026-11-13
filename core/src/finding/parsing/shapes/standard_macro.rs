// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use core::iter::once;

use syn::Error;
use syn::Expr;
use syn::Macro;
use syn::Pat;
use syn::Token;
use syn::parse::ParseStream;
use syn::punctuated::Punctuated;

// A macro of the standard library whose arguments are expressions --
// assertions, formatting, `vec!`, `matches!` and the rest of their family --
// and so can be read as the code they are. A macro outside the family expands
// to its author's code, which the syntax tree never shows, so it is never
// read: nothing is judged on a guess. Named by its last segment, as
// `std::assert!` is `assert!`.
pub struct StandardMacro<'a> {
    node: &'a Macro,
    name: String,
}

impl<'a> StandardMacro<'a> {
    const NAMES: [&'static str; 20] = [
        "assert",
        "assert_eq",
        "assert_ne",
        "dbg",
        "debug_assert",
        "debug_assert_eq",
        "debug_assert_ne",
        "eprint",
        "eprintln",
        "format",
        "matches",
        "panic",
        "print",
        "println",
        "todo",
        "unimplemented",
        "unreachable",
        "vec",
        "write",
        "writeln",
    ];

    pub fn of(node: &'a Macro) -> Option<Self> {
        let name = node.path.segments.last()?.ident.to_string();
        Self::NAMES
            .contains(&name.as_str())
            .then_some(Self { node, name })
    }

    // Its arguments, each an expression: a list separated by commas, or the
    // element and count of `vec![element; count]`. None when the body is
    // neither -- a pattern of `matches!` binding by reference, say.
    pub fn arguments(&self) -> Option<Vec<Expr>> {
        self.node
            .parse_body_with(Punctuated::<Expr, Token![,]>::parse_terminated)
            .map(|listed| listed.into_iter().collect())
            .or_else(|_| self.node.parse_body_with(Self::repeated))
            .ok()
    }

    // The expressions it holds: its arguments, but for `matches!`, whose second
    // argument is a pattern -- what it matches and its guard alone.
    pub fn expressions(&self) -> Option<Vec<Expr>> {
        if self.name == "matches" {
            self.node.parse_body_with(Self::matched).ok()
        } else {
            self.arguments()
        }
    }

    fn matched(input: ParseStream) -> Result<Vec<Expr>, Error> {
        let scrutinee: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        Pat::parse_multi_with_leading_vert(input)?;
        let guard = if input.peek(Token![if]) {
            input.parse::<Token![if]>()?;
            Some(input.parse::<Expr>()?)
        } else {
            None
        };
        input.parse::<Option<Token![,]>>()?;
        Ok(once(scrutinee).chain(guard).collect())
    }

    fn repeated(input: ParseStream) -> Result<Vec<Expr>, Error> {
        let element: Expr = input.parse()?;
        input.parse::<Token![;]>()?;
        let count: Expr = input.parse()?;
        Ok(vec![element, count])
    }
}
