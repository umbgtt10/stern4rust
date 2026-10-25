// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use syn::Block;
use syn::Expr;
use syn::FnArg;
use syn::Local;
use syn::Pat;
use syn::Signature;
use syn::visit::Visit;
use syn::visit::visit_local;

use crate::finding::parsing::dependencies::built_text::BuiltText;

// Whether a value a method stores came from its caller.
//
// Passed in is a parameter, as it is or as `Some(p)`, `Box::new(p)`,
// `Rc::new(p)`, `Arc::new(p)`, `p.clone()`, `Rc::clone(&p)`, `Arc::clone(&p)` or
// `p.into()`, or a parameter's field -- a part handed over inside a bundle of
// them. `None`, `Vec::new()` and `vec![]` hold nothing, so they build nothing.
// A local counts as what it was bound to; a chain of locals is followed only so
// far, so a name rebound to itself is not followed forever.
pub struct PassedIn {
    parameters: BTreeSet<String>,
    locals: BTreeMap<String, Expr>,
}

impl PassedIn {
    const CARRIERS: [&'static str; 6] = [
        "Some",
        "Box::new",
        "Rc::new",
        "Arc::new",
        "Rc::clone",
        "Arc::clone",
    ];
    const EMPTY: [&'static str; 2] = ["None", "Vec::new"];
    const FOLLOWED: usize = 8;
    const RELAYS: [&'static str; 2] = ["clone", "into"];

    pub fn of(signature: &Signature, body: &Block) -> Self {
        let parameters = signature
            .inputs
            .iter()
            .filter_map(|input| match input {
                FnArg::Typed(typed) => match typed.pat.as_ref() {
                    Pat::Ident(name) => Some(name.ident.to_string()),
                    _ => None,
                },
                FnArg::Receiver(_) => None,
            })
            .collect();
        let mut found = Self {
            parameters,
            locals: BTreeMap::new(),
        };
        found.visit_block(body);
        found
    }

    pub fn holds(&self, value: &Expr) -> bool {
        self.holds_within(value, Self::FOLLOWED)
    }

    pub fn origin<'a>(&'a self, value: &'a Expr) -> &'a Expr {
        (0..Self::FOLLOWED).fold(value, |current, _| self.bound(current).unwrap_or(current))
    }

    fn bound(&self, value: &Expr) -> Option<&Expr> {
        let Expr::Path(path) = value else {
            return None;
        };
        let name = path.path.get_ident()?.to_string();
        if self.parameters.contains(&name) {
            return None;
        }
        self.locals.get(&name)
    }

    fn holds_within(&self, value: &Expr, left: usize) -> bool {
        if left == 0 {
            return false;
        }
        match value {
            Expr::Path(_) => {
                let named = BuiltText::of(value);
                self.parameters.contains(&named)
                    || Self::EMPTY.contains(&named.as_str())
                    || self
                        .bound(value)
                        .is_some_and(|local| self.holds_within(local, left - 1))
            }
            Expr::Call(call) => match (call.func.as_ref(), call.args.first()) {
                (Expr::Path(_), None) => Self::EMPTY.contains(&BuiltText::of(&call.func).as_str()),
                (Expr::Path(_), Some(carried)) => {
                    call.args.len() == 1
                        && Self::CARRIERS.contains(&BuiltText::of(&call.func).as_str())
                        && self.holds_within(carried, left - 1)
                }
                _ => false,
            },
            Expr::MethodCall(call) => {
                call.args.is_empty()
                    && Self::RELAYS.contains(&call.method.to_string().as_str())
                    && self.holds_within(&call.receiver, left - 1)
            }
            Expr::Field(field) => self.holds_within(&field.base, left - 1),
            Expr::Reference(reference) => self.holds_within(&reference.expr, left - 1),
            Expr::Paren(inner) => self.holds_within(&inner.expr, left - 1),
            Expr::Macro(invocation) => {
                invocation.mac.path.is_ident("vec") && invocation.mac.tokens.is_empty()
            }
            _ => false,
        }
    }
}

impl<'ast> Visit<'ast> for PassedIn {
    fn visit_local(&mut self, local: &'ast Local) {
        let name = match &local.pat {
            Pat::Ident(name) => Some(name.ident.to_string()),
            Pat::Type(typed) => match typed.pat.as_ref() {
                Pat::Ident(name) => Some(name.ident.to_string()),
                _ => None,
            },
            _ => None,
        };
        if let (Some(name), Some(init)) = (name, &local.init) {
            self.locals.insert(name, init.expr.as_ref().clone());
        }
        visit_local(self, local);
    }
}
