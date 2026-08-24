// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::collections::BTreeSet;

use syn::Expr;
use syn::ExprCall;
use syn::Item;
use syn::ItemImpl;
use syn::Path;
use syn::TypePath;
use syn::UseTree;
use syn::parse_file;
use syn::spanned::Spanned;
use syn::visit;
use syn::visit::Visit;
use syn::visit::visit_expr_call;

use crate::finding::model::qualified_call::QualifiedCall;
use crate::source_file::SourceFile;

// The calls a file reaches through a path instead of an import.
//
// Three shapes are left alone. An unqualified `helper()` has nothing to import.
// A type qualifier -- `Widget::new`, `Self::inner` -- is not a path standing in
// for an import, because the type itself was imported and the qualifier says
// which type is being constructed. And a single segment this file actually
// imported is the idiomatic form: `use std::fs` followed by `fs::read_to_string`
// keeps the module visible at the call site while still stating the route once.
//
// What is left is a path doing an import's job: `syn::parse_file` works without
// anything in the file saying where `syn` came from, and `std::env::args` spells
// out at the call site a route that belongs at the top.
//
// Uppercase-initial first segments are read as types. That is a convention
// rather than a resolution -- this tool does not have type information -- and it
// is the same convention rustfmt relies on to order imports.
pub struct QualifiedCallFinder {
    imported: BTreeSet<String>,
    found: Vec<QualifiedCall>,
}

impl QualifiedCallFinder {
    // None means the file does not parse. readable-source reports that.
    pub fn find(file: &SourceFile) -> Option<Vec<QualifiedCall>> {
        let syntax = parse_file(&file.contents()).ok()?;
        let mut finder = Self {
            imported: Self::imports(&syntax.items).into_iter().collect(),
            found: Vec::new(),
        };
        finder.visit_file(&syntax);
        Some(finder.found)
    }

    // The names a file's `use` statements put in scope, which is what separates
    // an idiomatic one-segment qualifier from a path standing in for an import.
    // A glob contributes nothing: it puts names in scope that cannot be read off
    // the syntax tree, so calls relying on one are reported.
    fn imports(items: &[Item]) -> Vec<String> {
        items
            .iter()
            .flat_map(|item| match item {
                Item::Use(entry) => Self::names(&entry.tree),
                Item::Mod(module) => module
                    .content
                    .as_ref()
                    .map(|(_, inner)| Self::imports(inner))
                    .unwrap_or_default(),
                _ => Vec::new(),
            })
            .collect()
    }

    fn names(tree: &UseTree) -> Vec<String> {
        Self::names_under(tree, None)
    }

    // `parent` is the segment a `self` inside this tree stands for.
    // `use std::io::{self, Write};` puts `io` in scope, not a name spelled
    // `self`, and reading the tree literally made `io::Error` look unimported
    // -- reporting the very shape the rule exists to encourage.
    fn names_under(tree: &UseTree, parent: Option<&str>) -> Vec<String> {
        match tree {
            UseTree::Name(name) => {
                let ident = name.ident.to_string();
                if ident == "self" {
                    parent.map(ToString::to_string).into_iter().collect()
                } else {
                    vec![ident]
                }
            }
            UseTree::Rename(rename) => vec![rename.rename.to_string()],
            UseTree::Path(path) => Self::names_under(&path.tree, Some(&path.ident.to_string())),
            UseTree::Group(group) => group
                .items
                .iter()
                .flat_map(|item| Self::names_under(item, parent))
                .collect(),
            UseTree::Glob(_) => Vec::new(),
        }
    }

    fn offending(&self, node: &ExprCall) -> Option<QualifiedCall> {
        let Expr::Path(entry) = node.func.as_ref() else {
            return None;
        };
        if entry.qself.is_some() {
            return None;
        }
        self.offending_path(&entry.path, node.func.span().start().line)
    }

    // The whole of the rule, in segments. A path is fine when it is a single
    // name, when a type qualifies it, or when its one qualifier was imported;
    // anything else is a route that belongs in the import list. Call position
    // and type position ask the same question, so they share the answer.
    fn offending_path(&self, path: &Path, line: usize) -> Option<QualifiedCall> {
        let segments: Vec<String> = path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect();
        let first = segments.first()?;
        if segments.len() < 2 || Self::is_type(first) {
            return None;
        }
        if segments.len() == 2 && self.imported.contains(first) {
            return None;
        }
        Some(QualifiedCall::new(&segments.join("::"), line))
    }

    fn is_type(segment: &str) -> bool {
        segment.chars().next().is_some_and(char::is_uppercase) || Self::is_primitive(segment)
    }

    // The one lowercase set that can be known rather than guessed at. The case
    // convention reads `u64` as a module, so `u64::from_le_bytes` was reported
    // with the correction `use u64::from_le_bytes;` -- which does not compile:
    // a primitive's inherent function cannot be imported at all. An offence
    // whose correction is not Rust is worse than a missed one, and unlike a
    // user's lowercase-named type these names are fixed by the language, so
    // naming them is a fact rather than a second convention.
    //
    // Matched whole. A module named `u64_helpers` is still a module.
    fn is_primitive(segment: &str) -> bool {
        matches!(
            segment,
            "u8" | "u16"
                | "u32"
                | "u64"
                | "u128"
                | "usize"
                | "i8"
                | "i16"
                | "i32"
                | "i64"
                | "i128"
                | "isize"
                | "f32"
                | "f64"
                | "bool"
                | "char"
                | "str"
        )
    }
}

impl<'ast> Visit<'ast> for QualifiedCallFinder {
    // A type reached through a path is the same dependency a call reached
    // through one is: absent from the import list, invisible to a reader
    // scanning the top of the file for what this file needs.
    fn visit_type_path(&mut self, node: &'ast TypePath) {
        if node.qself.is_none()
            && let Some(found) = self.offending_path(&node.path, node.path.span().start().line)
        {
            self.found.push(found);
        }
        visit::visit_type_path(self, node);
    }

    // `impl std::fmt::Display for Widget` carries the trait as a bare `Path`
    // rather than a `Type`, so it needs its own visit or it is missed.
    fn visit_item_impl(&mut self, node: &'ast ItemImpl) {
        if let Some((_, path, _)) = node.trait_.as_ref()
            && let Some(found) = self.offending_path(path, path.span().start().line)
        {
            self.found.push(found);
        }
        visit::visit_item_impl(self, node);
    }

    fn visit_expr_call(&mut self, node: &'ast ExprCall) {
        if let Some(call) = self.offending(node) {
            self.found.push(call);
        }
        visit_expr_call(self, node);
    }
}
