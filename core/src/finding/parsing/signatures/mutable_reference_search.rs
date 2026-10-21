// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use syn::Type;
use syn::TypeReference;

use crate::finding::parsing::signatures::type_walk::TypeWalk;

// Whether a type holds a mutable reference anywhere it structurally can: through
// references, tuples, arrays, slices and generic arguments.
//
// A callback's own contract is not descended into. `impl FnMut(&mut T)` and
// `fn(&mut T)` describe what a closure receives, not what the type holds.
//
// A search that skips statics passes over `&'static mut` -- a static handed over
// once, which an embedded executor requires, rather than a type lending its
// internals -- but still looks inside it, so a borrow it points at is found.
pub struct MutableReferenceSearch {
    skips_static: bool,
}

impl MutableReferenceSearch {
    pub fn new(skips_static: bool) -> Self {
        Self { skips_static }
    }

    pub fn finds_in(&self, ty: &Type) -> bool {
        match ty {
            Type::Reference(reference) if self.lends(reference) => true,
            _ => TypeWalk::children(ty)
                .into_iter()
                .any(|child| self.finds_in(child)),
        }
    }

    fn is_static(reference: &TypeReference) -> bool {
        reference
            .lifetime
            .as_ref()
            .is_some_and(|lifetime| lifetime.ident == "static")
    }

    fn lends(&self, reference: &TypeReference) -> bool {
        reference.mutability.is_some() && !(self.skips_static && Self::is_static(reference))
    }
}
