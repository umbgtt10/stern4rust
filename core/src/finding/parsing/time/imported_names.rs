// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use core::iter::once;

use syn::ImplItem;
use syn::Item;
use syn::Stmt;
use syn::UseTree;

// Where the names a file uses come from, read off its `use` items wherever they stand: a name imported alone, in a
// group, renamed, or as `self` in a group maps to the path it names; a glob may supply any name from its module.
// So a bare `sleep` imported from `tokio::time` is Tokio's, and one the file defines is its own.
pub struct ImportedNames {
    names: BTreeMap<String, String>,
    globs: Vec<String>,
}

impl ImportedNames {
    pub fn of(items: &[Item]) -> Self {
        let trees = Self::uses_in(items);
        Self {
            names: trees
                .iter()
                .flat_map(|tree| Self::names_in(tree, ""))
                .collect(),
            globs: trees
                .iter()
                .flat_map(|tree| Self::globs_in(tree, ""))
                .collect(),
        }
    }

    // The full paths a path written in the file may stand for: through an import of its first segment, or as written
    // when nothing imports it, and through each glob in either case.
    pub fn resolve(&self, written: &[String]) -> Vec<String> {
        let Some((first, rest)) = written.split_first() else {
            return Vec::new();
        };
        let tail = rest
            .iter()
            .map(|segment| format!("::{segment}"))
            .collect::<String>();
        let direct = self
            .names
            .get(first)
            .map_or_else(|| written.join("::"), |path| format!("{path}{tail}"));
        let globbed = self
            .globs
            .iter()
            .map(|module| format!("{module}::{}", written.join("::")));
        once(direct).chain(globbed).collect()
    }

    fn globs_in(tree: &UseTree, prefix: &str) -> Vec<String> {
        match tree {
            UseTree::Path(path) => {
                Self::globs_in(&path.tree, &Self::joined(prefix, &path.ident.to_string()))
            }
            UseTree::Glob(_) => vec![String::from(prefix)],
            UseTree::Group(group) => group
                .items
                .iter()
                .flat_map(|inner| Self::globs_in(inner, prefix))
                .collect(),
            UseTree::Name(_) | UseTree::Rename(_) => Vec::new(),
        }
    }

    fn joined(prefix: &str, name: &str) -> String {
        if prefix.is_empty() {
            String::from(name)
        } else {
            format!("{prefix}::{name}")
        }
    }

    fn names_in(tree: &UseTree, prefix: &str) -> BTreeMap<String, String> {
        match tree {
            UseTree::Path(path) => {
                Self::names_in(&path.tree, &Self::joined(prefix, &path.ident.to_string()))
            }
            UseTree::Name(name) if name.ident == "self" => prefix
                .rsplit("::")
                .next()
                .map(|last| BTreeMap::from([(String::from(last), String::from(prefix))]))
                .unwrap_or_default(),
            UseTree::Name(name) => BTreeMap::from([(
                name.ident.to_string(),
                Self::joined(prefix, &name.ident.to_string()),
            )]),
            UseTree::Rename(rename) => BTreeMap::from([(
                rename.rename.to_string(),
                Self::joined(prefix, &rename.ident.to_string()),
            )]),
            UseTree::Glob(_) => BTreeMap::new(),
            UseTree::Group(group) => group
                .items
                .iter()
                .flat_map(|inner| Self::names_in(inner, prefix))
                .collect(),
        }
    }

    // Every `use` item among these items, a module's and a function body's included.
    fn uses_in(items: &[Item]) -> Vec<UseTree> {
        items
            .iter()
            .flat_map(|item| match item {
                Item::Use(used) => vec![used.tree.clone()],
                Item::Mod(module) => module
                    .content
                    .as_ref()
                    .map(|(_, inner)| Self::uses_in(inner))
                    .unwrap_or_default(),
                Item::Fn(function) => Self::uses_in_statements(&function.block.stmts),
                Item::Impl(block) => block
                    .items
                    .iter()
                    .flat_map(|inner| match inner {
                        ImplItem::Fn(method) => Self::uses_in_statements(&method.block.stmts),
                        _ => Vec::new(),
                    })
                    .collect(),
                _ => Vec::new(),
            })
            .collect()
    }

    fn uses_in_statements(statements: &[Stmt]) -> Vec<UseTree> {
        let items: Vec<Item> = statements
            .iter()
            .filter_map(|statement| match statement {
                Stmt::Item(item) => Some(item.clone()),
                _ => None,
            })
            .collect();
        Self::uses_in(&items)
    }
}
