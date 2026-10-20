// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use syn::GenericArgument;
use syn::PathArguments;
use syn::PathSegment;
use syn::Type;
use syn::TypeReference;

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
            Type::Reference(reference) => self.lends(reference) || self.finds_in(&reference.elem),
            Type::Tuple(tuple) => tuple.elems.iter().any(|elem| self.finds_in(elem)),
            Type::Paren(inner) => self.finds_in(&inner.elem),
            Type::Group(inner) => self.finds_in(&inner.elem),
            Type::Array(array) => self.finds_in(&array.elem),
            Type::Slice(slice) => self.finds_in(&slice.elem),
            Type::Path(path) => path
                .path
                .segments
                .iter()
                .any(|segment| self.in_segment(segment)),
            _ => false,
        }
    }

    fn in_argument(&self, argument: &GenericArgument) -> bool {
        match argument {
            GenericArgument::Type(ty) => self.finds_in(ty),
            _ => false,
        }
    }

    fn in_segment(&self, segment: &PathSegment) -> bool {
        match &segment.arguments {
            PathArguments::AngleBracketed(arguments) => arguments
                .args
                .iter()
                .any(|argument| self.in_argument(argument)),
            _ => false,
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
