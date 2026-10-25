// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::collections::BTreeSet;

use syn::GenericArgument;
use syn::GenericParam;
use syn::Generics;
use syn::PathArguments;
use syn::Type;
use syn::TypeParamBound;
use syn::WherePredicate;
use syn::punctuated::Punctuated;
use syn::token::Plus;

use crate::finding::parsing::shapes::value_traits::ValueTraits;

// Which types are seams: a trait object, an `impl Trait`, or a type parameter
// bounded by a trait -- inline or in a `where` clause -- as they are or wrapped in
// a Box, an Rc, an Arc, an Option, a Vec or a reference.
//
// A parameter bounded only by value traits -- `T: Clone + Debug` -- is data the
// type carries, not a seam it opens; its bound says how it is copied or printed,
// not what it does.
pub struct SeamTypes {
    bounded: BTreeSet<String>,
}

impl SeamTypes {
    const WRAPPERS: [&'static str; 5] = ["Arc", "Box", "Option", "Rc", "Vec"];

    pub fn of(generics: &[&Generics]) -> Self {
        let inline = generics
            .iter()
            .flat_map(|declared| declared.params.iter())
            .filter_map(|param| match param {
                GenericParam::Type(declared) if Self::is_behaviour(&declared.bounds) => {
                    Some(declared.ident.to_string())
                }
                _ => None,
            });
        let clauses = generics
            .iter()
            .filter_map(|declared| declared.where_clause.as_ref())
            .flat_map(|clause| clause.predicates.iter())
            .filter_map(|predicate| match predicate {
                WherePredicate::Type(bound) if Self::is_behaviour(&bound.bounds) => {
                    match &bound.bounded_ty {
                        Type::Path(path) => path.path.get_ident().map(ToString::to_string),
                        _ => None,
                    }
                }
                _ => None,
            });
        Self {
            bounded: inline.chain(clauses).collect(),
        }
    }

    pub fn is_seam(&self, ty: &Type) -> bool {
        match ty {
            Type::TraitObject(_) | Type::ImplTrait(_) => true,
            Type::Reference(reference) => self.is_seam(&reference.elem),
            Type::Paren(inner) => self.is_seam(&inner.elem),
            Type::Path(path) if path.qself.is_none() => {
                if let Some(name) = path.path.get_ident() {
                    return self.bounded.contains(&name.to_string());
                }
                path.path.segments.last().is_some_and(|last| {
                    Self::WRAPPERS.contains(&last.ident.to_string().as_str())
                        && self.wraps_seam(&last.arguments)
                })
            }
            _ => false,
        }
    }

    fn is_behaviour(bounds: &Punctuated<TypeParamBound, Plus>) -> bool {
        bounds.iter().any(|bound| match bound {
            TypeParamBound::Trait(declared) => declared
                .path
                .segments
                .last()
                .is_some_and(|last| !ValueTraits::contains(&last.ident.to_string())),
            _ => false,
        })
    }

    fn wraps_seam(&self, arguments: &PathArguments) -> bool {
        match arguments {
            PathArguments::AngleBracketed(angled) => {
                angled.args.iter().any(|argument| match argument {
                    GenericArgument::Type(inner) => self.is_seam(inner),
                    _ => false,
                })
            }
            _ => false,
        }
    }
}
