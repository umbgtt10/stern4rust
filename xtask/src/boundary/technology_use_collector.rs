// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use syn::visit::{Visit, visit_expr_method_call, visit_macro, visit_path};
use syn::{ExprMethodCall, File, ItemExternCrate, ItemUse, Macro, Path, UseTree};

use crate::boundary::boundary_rule::BoundaryRule;

// What one parsed file reaches of a rule's forbidden modules, macros and
// methods, and any `extern crate std`, which would hand a `no_std` crate the
// standard library back -- each named once, in order. Read from the syntax tree, so a string
// literal that mentions `std::fs` is data and not a use; a macro's arguments
// are tokens syn never parses, so a use inside `assert!(...)` is not seen.
pub struct TechnologyUseCollector<'a> {
    rule: &'a BoundaryRule,
    found: Vec<String>,
}

impl<'a> TechnologyUseCollector<'a> {
    pub fn collect(file: &File, rule: &'a BoundaryRule) -> Vec<String> {
        let mut collector = Self {
            rule,
            found: Vec::new(),
        };
        collector.visit_file(file);
        collector.found.sort();
        collector.found.dedup();
        collector.found
    }

    fn record_path(&mut self, segments: &[String]) {
        let joined = segments.join("::");
        if let Some(module) = self
            .rule
            .forbidden_modules
            .iter()
            .find(|module| joined == **module || joined.starts_with(&format!("{module}::")))
        {
            self.found.push(module.clone());
        }
    }

    fn use_paths(tree: &UseTree, prefix: &[String]) -> Vec<Vec<String>> {
        let extended = |ident: String| [prefix, &[ident]].concat();
        match tree {
            UseTree::Path(path) => Self::use_paths(&path.tree, &extended(path.ident.to_string())),
            UseTree::Name(name) => vec![extended(name.ident.to_string())],
            UseTree::Rename(rename) => vec![extended(rename.ident.to_string())],
            UseTree::Glob(_) => vec![prefix.to_vec()],
            UseTree::Group(group) => group
                .items
                .iter()
                .flat_map(|item| Self::use_paths(item, prefix))
                .collect(),
        }
    }
}

impl<'ast> Visit<'ast> for TechnologyUseCollector<'_> {
    fn visit_item_extern_crate(&mut self, node: &'ast ItemExternCrate) {
        if node.ident == "std" {
            self.found.push(String::from("extern crate std"));
        }
    }

    fn visit_item_use(&mut self, node: &'ast ItemUse) {
        for path in Self::use_paths(&node.tree, &[]) {
            self.record_path(&path);
        }
    }

    fn visit_path(&mut self, node: &'ast Path) {
        let segments: Vec<String> = node
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect();
        self.record_path(&segments);
        visit_path(self, node);
    }

    fn visit_macro(&mut self, node: &'ast Macro) {
        if let Some(name) = node
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string())
            && self.rule.forbidden_macros.contains(&name)
        {
            self.found.push(format!("{name}!"));
        }
        visit_macro(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast ExprMethodCall) {
        let method = node.method.to_string();
        if self.rule.forbidden_methods.contains(&method) {
            self.found.push(format!(".{method}()"));
        }
        visit_expr_method_call(self, node);
    }
}
