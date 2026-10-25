// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::collections::BTreeSet;

use syn::Block;
use syn::Expr;
use syn::ExprCall;
use syn::ExprMethodCall;
use syn::FnArg;
use syn::Local;
use syn::Pat;
use syn::Path;
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
// What only repackages passed-in values is passed in too: `p?`, an `into_*`
// conversion of one -- Rust's name for a conversion of what it is given -- and a
// function of the type's own, `Self::validated(p, q)`, handed nothing else. Any
// other method on a parameter, `settings.reporter()`, builds.
// A local counts as what it was bound to, by name or by destructuring -- a tuple
// pattern over a tuple by position, any other pattern each of its names to the
// whole value -- and a local shadows a parameter of its name. A name rebound in
// its own initialiser reads the binding before, so `let parts = parts.into()`
// follows the earlier `parts`; a chain of locals is followed only so far.
pub struct PassedIn {
    owner: String,
    parameters: BTreeSet<String>,
    locals: Vec<(String, Expr)>,
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
    const CONVERSION: &'static str = "into_";
    const EMPTY: [&'static str; 2] = ["None", "Vec::new"];
    const FOLLOWED: usize = 8;
    const RELAYS: [&'static str; 2] = ["clone", "into"];

    pub fn of(signature: &Signature, body: &Block, owner: &str) -> Self {
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
            owner: owner.to_string(),
            parameters,
            locals: Vec::new(),
        };
        found.visit_block(body);
        found
    }

    pub fn holds(&self, value: &Expr) -> bool {
        self.holds_within(value, self.locals.len(), Self::FOLLOWED)
    }

    pub fn origin<'a>(&'a self, value: &'a Expr) -> &'a Expr {
        (0..Self::FOLLOWED)
            .fold((value, self.locals.len()), |(current, before), _| {
                self.bound(current, before)
                    .map_or((current, before), |(at, init)| (init, at))
            })
            .0
    }

    fn bindings(pattern: &Pat, value: &Expr) -> Vec<(String, Expr)> {
        match (pattern, value) {
            (Pat::Type(typed), _) => Self::bindings(&typed.pat, value),
            (Pat::Tuple(tuple), Expr::Tuple(values)) if tuple.elems.len() == values.elems.len() => {
                tuple
                    .elems
                    .iter()
                    .zip(values.elems.iter())
                    .flat_map(|(element, part)| Self::bindings(element, part))
                    .collect()
            }
            _ => Self::names(pattern)
                .into_iter()
                .map(|name| (name, value.clone()))
                .collect(),
        }
    }

    // The latest binding of the name before `before`, with where it sits.
    fn bound(&self, value: &Expr, before: usize) -> Option<(usize, &Expr)> {
        let Expr::Path(path) = value else {
            return None;
        };
        let name = path.path.get_ident()?.to_string();
        let at = self.locals[..before]
            .iter()
            .rposition(|(bound, _)| *bound == name)?;
        Some((at, &self.locals[at].1))
    }

    // A call holds a passed-in value when it only carries one, when it is a
    // function of the type's own handed nothing else, or when it makes an empty
    // vector.
    fn holds_call(&self, call: &ExprCall, before: usize, left: usize) -> bool {
        let Expr::Path(function) = call.func.as_ref() else {
            return false;
        };
        let named = BuiltText::of(&call.func);
        let Some(carried) = call.args.first() else {
            return Self::EMPTY.contains(&named.as_str());
        };
        let carries = call.args.len() == 1
            && Self::CARRIERS.contains(&named.as_str())
            && self.holds_within(carried, before, left);
        carries
            || (self.is_own(&function.path)
                && call
                    .args
                    .iter()
                    .all(|argument| self.holds_within(argument, before, left)))
    }

    // A method call relays its receiver when it clones it, converts it with
    // `into`, or is an `into_*` conversion of it.
    fn holds_relayed(&self, call: &ExprMethodCall, before: usize, left: usize) -> bool {
        let method = call.method.to_string();
        let relays = (call.args.is_empty() && Self::RELAYS.contains(&method.as_str()))
            || method.starts_with(Self::CONVERSION);
        relays && self.holds_within(&call.receiver, before, left)
    }

    fn holds_within(&self, value: &Expr, before: usize, left: usize) -> bool {
        if left == 0 {
            return false;
        }
        match value {
            Expr::Path(_) => match self.bound(value, before) {
                Some((at, init)) => self.holds_within(init, at, left - 1),
                None => {
                    let named = BuiltText::of(value);
                    self.parameters.contains(&named) || Self::EMPTY.contains(&named.as_str())
                }
            },
            Expr::Call(call) => self.holds_call(call, before, left - 1),
            Expr::MethodCall(call) => self.holds_relayed(call, before, left - 1),
            Expr::Try(attempt) => self.holds_within(&attempt.expr, before, left - 1),
            Expr::Field(field) => self.holds_within(&field.base, before, left - 1),
            Expr::Reference(reference) => self.holds_within(&reference.expr, before, left - 1),
            Expr::Paren(inner) => self.holds_within(&inner.expr, before, left - 1),
            Expr::Macro(invocation) => {
                invocation.mac.path.is_ident("vec") && invocation.mac.tokens.is_empty()
            }
            _ => false,
        }
    }

    fn is_own(&self, path: &Path) -> bool {
        match path.segments.iter().collect::<Vec<_>>().as_slice() {
            [owner, _] => owner.ident == "Self" || owner.ident == self.owner,
            _ => false,
        }
    }

    fn names(pattern: &Pat) -> Vec<String> {
        match pattern {
            Pat::Ident(bound) => [bound.ident.to_string()]
                .into_iter()
                .chain(
                    bound
                        .subpat
                        .iter()
                        .flat_map(|(_, inner)| Self::names(inner)),
                )
                .collect(),
            Pat::Type(typed) => Self::names(&typed.pat),
            Pat::Reference(reference) => Self::names(&reference.pat),
            Pat::Paren(inner) => Self::names(&inner.pat),
            Pat::Struct(fields) => fields
                .fields
                .iter()
                .flat_map(|field| Self::names(&field.pat))
                .collect(),
            Pat::Tuple(tuple) => tuple.elems.iter().flat_map(Self::names).collect(),
            Pat::TupleStruct(tuple) => tuple.elems.iter().flat_map(Self::names).collect(),
            Pat::Slice(slice) => slice.elems.iter().flat_map(Self::names).collect(),
            _ => Vec::new(),
        }
    }
}

impl<'ast> Visit<'ast> for PassedIn {
    fn visit_local(&mut self, local: &'ast Local) {
        if let Some(init) = &local.init {
            let bound = Self::bindings(&local.pat, &init.expr);
            self.locals.extend(bound);
        }
        visit_local(self, local);
    }
}
