// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use syn::GenericArgument;
use syn::PathArguments;
use syn::PathSegment;
use syn::Type;

// The types a type is made of, one level down: what a reference points at, a
// tuple's elements, an array's or a slice's element, and a path's angle-bracketed
// type arguments. Every search over a type walks this shape.
//
// A callback's parameter list -- `Fn(A, B)`, `fn(A, B)` -- and an `impl Trait` or
// `dyn` bound are not part of it: what a closure receives is the callback's
// contract, not something this type holds.
pub struct TypeWalk;

impl TypeWalk {
    pub fn children(ty: &Type) -> Vec<&Type> {
        match ty {
            Type::Reference(reference) => vec![&*reference.elem],
            Type::Tuple(tuple) => tuple.elems.iter().collect(),
            Type::Paren(inner) => vec![&*inner.elem],
            Type::Group(inner) => vec![&*inner.elem],
            Type::Array(array) => vec![&*array.elem],
            Type::Slice(slice) => vec![&*slice.elem],
            Type::Path(path) => path
                .path
                .segments
                .iter()
                .flat_map(Self::arguments)
                .collect(),
            _ => Vec::new(),
        }
    }

    fn arguments(segment: &PathSegment) -> Vec<&Type> {
        match &segment.arguments {
            PathArguments::AngleBracketed(arguments) => arguments
                .args
                .iter()
                .filter_map(|argument| match argument {
                    GenericArgument::Type(ty) => Some(ty),
                    _ => None,
                })
                .collect(),
            _ => Vec::new(),
        }
    }
}
