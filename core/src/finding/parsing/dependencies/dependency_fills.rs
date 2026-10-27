// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use syn::Block;
use syn::Expr;
use syn::ExprAssign;
use syn::ExprCall;
use syn::ExprStruct;
use syn::Member;
use syn::Path;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::visit::visit_expr_assign;
use syn::visit::visit_expr_call;
use syn::visit::visit_expr_struct;

use crate::finding::parsing::dependencies::delegation::Delegation;
use crate::finding::parsing::dependencies::fill::Fill;

// Where a method body fills a field of its own type, the type named `Self` or by
// its name: each field of a struct literal, each position of a tuple struct's
// constructor, each `self.field = ...`, and each call to another associated
// function of the type with what it hands over. Closures and nested blocks are
// read too; another type's literals and calls are not.
pub struct DependencyFills {
    owner: String,
    fills: Vec<Fill>,
    delegations: Vec<Delegation>,
    assignments: Vec<Fill>,
}

impl DependencyFills {
    fn read(body: &Block, owner: &str) -> Self {
        let mut found = Self {
            owner: owner.to_string(),
            fills: Vec::new(),
            delegations: Vec::new(),
            assignments: Vec::new(),
        };
        found.visit_block(body);
        found
    }

    pub fn assignments(body: &Block) -> Vec<Fill> {
        Self::read(body, "").assignments
    }

    pub fn delegations(body: &Block, owner: &str) -> Vec<Delegation> {
        Self::read(body, owner).delegations
    }

    pub fn fills(body: &Block, owner: &str) -> Vec<Fill> {
        Self::read(body, owner).fills
    }

    fn fill(field: String, value: &Expr) -> Fill {
        Fill {
            field,
            value: value.clone(),
            line: value.span().start().line,
        }
    }

    fn is_owner(&self, path: &Path) -> bool {
        path.is_ident("Self") || path.is_ident(&self.owner)
    }

    fn member(member: &Member) -> String {
        match member {
            Member::Named(name) => name.to_string(),
            Member::Unnamed(index) => index.index.to_string(),
        }
    }
}

impl<'ast> Visit<'ast> for DependencyFills {
    fn visit_expr_assign(&mut self, assign: &'ast ExprAssign) {
        if let Expr::Field(field) = assign.left.as_ref()
            && let Expr::Path(base) = field.base.as_ref()
            && base.path.is_ident("self")
        {
            let found = Self::fill(Self::member(&field.member), &assign.right);
            self.assignments.push(found);
        }
        visit_expr_assign(self, assign);
    }

    fn visit_expr_call(&mut self, call: &'ast ExprCall) {
        if let Expr::Path(function) = call.func.as_ref() {
            let segments: Vec<String> = function
                .path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect();
            if self.is_owner(&function.path) {
                let filled = call
                    .args
                    .iter()
                    .enumerate()
                    .map(|(at, value)| Self::fill(at.to_string(), value));
                self.fills.extend(filled);
            } else if let [owner, target] = segments.as_slice()
                && (owner == "Self" || *owner == self.owner)
            {
                self.delegations.push(Delegation {
                    target: target.clone(),
                    arguments: call.args.iter().cloned().collect(),
                });
            }
        }
        visit_expr_call(self, call);
    }

    fn visit_expr_struct(&mut self, literal: &'ast ExprStruct) {
        if self.is_owner(&literal.path) {
            let filled = literal
                .fields
                .iter()
                .map(|field| Self::fill(Self::member(&field.member), &field.expr));
            self.fills.extend(filled);
        }
        visit_expr_struct(self, literal);
    }
}
