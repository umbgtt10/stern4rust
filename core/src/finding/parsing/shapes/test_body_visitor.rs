// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use proc_macro2::Delimiter;
use proc_macro2::TokenStream;
use proc_macro2::TokenTree;
use syn::Block;
use syn::Expr;
use syn::ExprCall;
use syn::ExprMethodCall;
use syn::ItemFn;
use syn::Macro;
use syn::Path;
use syn::visit::Visit;
use syn::visit::visit_expr_call;
use syn::visit::visit_expr_method_call;
use syn::visit::visit_path;

use crate::finding::parsing::shapes::package_surface::PackageSurface;
use crate::finding::parsing::shapes::standard_macro::StandardMacro;
use crate::finding::parsing::shapes::test_subject::TestSubject;

// One test's walk: its body, the helpers and local methods of its own file it
// calls, and the arguments of the standard macros. A macro outside that set
// expands to code the walk never sees, so it counts as behaviour -- the test is
// never accused on a guess.
pub struct TestBodyVisitor<'a> {
    surface: &'a PackageSurface,
    helpers: &'a BTreeMap<String, &'a Block>,
    local_methods: &'a BTreeMap<String, &'a Block>,
    records: BTreeSet<String>,
    behaviour: bool,
    expanded: BTreeSet<String>,
}

impl<'a> TestBodyVisitor<'a> {
    pub fn new(
        surface: &'a PackageSurface,
        helpers: &'a BTreeMap<String, &'a Block>,
        local_methods: &'a BTreeMap<String, &'a Block>,
    ) -> Self {
        Self {
            surface,
            helpers,
            local_methods,
            records: BTreeSet::new(),
            behaviour: false,
            expanded: BTreeSet::new(),
        }
    }

    pub fn examine(mut self, test: &'a ItemFn) -> TestSubject {
        self.visit_block(&test.block);
        TestSubject {
            name: test.sig.ident.to_string(),
            line: test.sig.ident.span().start().line,
            records: self.records.into_iter().collect(),
            exercises_behaviour: self.behaviour,
        }
    }

    fn called(&mut self, name: &str) {
        if self.surface.is_free_function(name) || self.surface.is_behaviour_method(name) {
            self.behaviour = true;
        }
        let helpers = self.helpers;
        self.expand(helpers, "fn ", name);
        let local_methods = self.local_methods;
        self.expand(local_methods, "method ", name);
    }

    fn expand(&mut self, bodies: &'a BTreeMap<String, &'a Block>, kind: &str, name: &str) {
        if let Some(body) = bodies.get(name) {
            if self.expanded.insert(String::from(kind) + name) {
                self.visit_block(body);
            }
        }
    }

    fn named(&mut self, path: &Path) {
        for segment in &path.segments {
            let ident = segment.ident.to_string();
            if self.surface.is_record(&ident) {
                self.records.insert(ident);
            }
        }
    }

    fn opens_call(next: Option<&TokenTree>) -> bool {
        matches!(next, Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Parenthesis)
    }

    fn scan_tokens(&mut self, tokens: TokenStream) {
        let trees: Vec<TokenTree> = tokens.into_iter().collect();
        for (index, tree) in trees.iter().enumerate() {
            match tree {
                TokenTree::Ident(ident) => self.scanned(&ident.to_string(), trees.get(index + 1)),
                TokenTree::Group(group) => self.scan_tokens(group.stream()),
                _ => {}
            }
        }
    }

    fn scanned(&mut self, name: &str, next: Option<&TokenTree>) {
        if self.surface.is_record(name) {
            self.records.insert(name.to_string());
        }
        if Self::opens_call(next) {
            self.called(name);
        }
    }
}

impl<'a: 'ast, 'ast> Visit<'ast> for TestBodyVisitor<'a> {
    fn visit_expr_call(&mut self, node: &'ast ExprCall) {
        if let Expr::Path(called) = node.func.as_ref() {
            let segments: Vec<String> = called
                .path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect();
            match segments.as_slice() {
                [name] => self.called(name),
                [.., qualifier, name] => {
                    let record = self.surface.is_record(qualifier);
                    if self.surface.is_object(qualifier)
                        || self.surface.is_free_function(name)
                        || (!record && self.surface.is_behaviour_method(name))
                    {
                        self.behaviour = true;
                    } else if !record {
                        let local_methods = self.local_methods;
                        self.expand(local_methods, "method ", name);
                    }
                }
                [] => {}
            }
        }
        visit_expr_call(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast ExprMethodCall) {
        let name = node.method.to_string();
        self.called(&name);
        visit_expr_method_call(self, node);
    }

    fn visit_macro(&mut self, node: &'ast Macro) {
        let Some(standard) = StandardMacro::of(node) else {
            self.behaviour = true;
            return;
        };
        match standard.arguments() {
            Some(arguments) => {
                for argument in &arguments {
                    Visit::visit_expr(self, argument);
                }
            }
            None => self.scan_tokens(node.tokens.clone()),
        }
    }

    fn visit_path(&mut self, node: &'ast Path) {
        self.named(node);
        visit_path(self, node);
    }
}
