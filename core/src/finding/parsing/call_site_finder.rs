// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::collections::BTreeSet;
use alloc::string::ToString;
use alloc::vec::Vec;

use proc_macro2::Delimiter;
use proc_macro2::TokenStream;
use proc_macro2::TokenTree;
use syn::Expr;
use syn::ExprCall;
use syn::ExprMethodCall;
use syn::Macro;
use syn::parse_file;
use syn::visit::Visit;
use syn::visit::visit_expr_call;
use syn::visit::visit_expr_method_call;
use syn::visit::visit_macro;

use crate::finding::model::public_entry_point::PublicEntryPoint;
use crate::source_file::SourceFile;

// Every call a file makes, as a name and a count of arguments.
//
// Macro arguments are scanned as well as parsed expressions, and that is not an
// optional refinement: a Rust test puts its assertion in `assert!` or
// `assert_eq!`, whose contents never become syntax. A collector that skipped
// them would miss the one call most tests care about and report tested code as
// untested.
//
// Inside a macro the tokens are counted rather than parsed -- an identifier
// followed by a parenthesised group, with arity the arguments its top-level
// commas part. That is looser than the parsed side and errs toward
// finding a call, which is the safe direction for a rule that would otherwise
// accuse tested code of being untested.
pub struct CallSiteFinder {
    found: BTreeSet<PublicEntryPoint>,
}

impl CallSiteFinder {
    pub fn find(file: &SourceFile) -> Option<BTreeSet<PublicEntryPoint>> {
        let syntax = parse_file(&file.contents()).ok()?;
        let mut finder = Self {
            found: BTreeSet::new(),
        };
        finder.visit_file(&syntax);
        Some(finder.found)
    }

    // An argument is counted where it opens: its first token after the group's
    // start or a comma. A trailing comma -- rustfmt's, on a call laid out one
    // argument a line -- closes the last argument and opens none.
    fn arity_of(tokens: TokenStream) -> usize {
        tokens
            .into_iter()
            .fold((0, false), |(arguments, open), tree| match tree {
                TokenTree::Punct(punct) if punct.as_char() == ',' => (arguments, false),
                _ if open => (arguments, true),
                _ => (arguments + 1, true),
            })
            .0
    }

    fn take_tokens(&mut self, tokens: TokenStream) {
        let trees: Vec<TokenTree> = tokens.into_iter().collect();
        for (index, tree) in trees.iter().enumerate() {
            if let TokenTree::Group(group) = tree {
                if group.delimiter() == Delimiter::Parenthesis {
                    if let Some(TokenTree::Ident(name)) = index.checked_sub(1).map(|at| &trees[at])
                    {
                        let arity = Self::arity_of(group.stream());
                        self.found
                            .insert(PublicEntryPoint::new(&name.to_string(), arity));
                    }
                }
                self.take_tokens(group.stream());
            }
        }
    }
}

impl<'ast> Visit<'ast> for CallSiteFinder {
    fn visit_expr_call(&mut self, node: &'ast ExprCall) {
        if let Expr::Path(path) = node.func.as_ref() {
            if let Some(segment) = path.path.segments.last() {
                self.found.insert(PublicEntryPoint::new(
                    &segment.ident.to_string(),
                    node.args.len(),
                ));
            }
        }
        visit_expr_call(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast ExprMethodCall) {
        self.found.insert(PublicEntryPoint::new(
            &node.method.to_string(),
            node.args.len(),
        ));
        visit_expr_method_call(self, node);
    }

    fn visit_macro(&mut self, node: &'ast Macro) {
        self.take_tokens(node.tokens.clone());
        visit_macro(self, node);
    }
}
