// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use syn::GenericArgument;
use syn::PathArguments;
use syn::Type;
use syn::TypeParamBound;

use crate::finding::parsing::signatures::type_text::TypeText;
use crate::finding::parsing::signatures::type_walk::TypeWalk;

// The first `Result` in a type whose error is a sentence rather than a kind: a
// `String`, a `&str`, or a boxed trait object whose trait ends in `Error` --
// `Box<dyn Error>`, `Box<dyn std::error::Error + Send + Sync>`. Wherever the
// `Result` sits, the outer one first, and named as it is written.
//
// A `Result` is a path ending in `Result` with two type arguments. An alias with
// one -- `io::Result<u8>` -- names its error elsewhere, and is not judged.
pub struct StringlyErrorFinder;

impl StringlyErrorFinder {
    pub fn first_in(ty: &Type) -> Option<String> {
        Self::error_of(ty)
            .filter(|error| Self::is_stringly(error))
            .map(TypeText::of)
            .or_else(|| TypeWalk::children(ty).into_iter().find_map(Self::first_in))
    }

    fn boxes_an_error(arguments: &PathArguments) -> bool {
        let PathArguments::AngleBracketed(arguments) = arguments else {
            return false;
        };
        arguments.args.iter().any(|argument| {
            matches!(argument, GenericArgument::Type(Type::TraitObject(object))
                if object.bounds.iter().any(Self::is_error_bound))
        })
    }

    fn error_of(ty: &Type) -> Option<&Type> {
        let Type::Path(path) = ty else {
            return None;
        };
        let segment = path.path.segments.last()?;
        let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
            return None;
        };
        let types: Vec<&Type> = arguments
            .args
            .iter()
            .filter_map(|argument| match argument {
                GenericArgument::Type(inner) => Some(inner),
                _ => None,
            })
            .collect();
        match types.as_slice() {
            [_, error] if segment.ident == "Result" => Some(error),
            _ => None,
        }
    }

    fn is_error_bound(bound: &TypeParamBound) -> bool {
        matches!(bound, TypeParamBound::Trait(found)
            if found.path.segments.last().is_some_and(|segment| segment.ident.to_string().ends_with("Error")))
    }

    fn is_stringly(ty: &Type) -> bool {
        match ty {
            Type::Path(path) => path.path.segments.last().is_some_and(|segment| {
                segment.ident == "String"
                    || (segment.ident == "Box" && Self::boxes_an_error(&segment.arguments))
            }),
            Type::Reference(reference) => {
                matches!(&*reference.elem, Type::Path(path) if path.path.is_ident("str"))
            }
            _ => false,
        }
    }
}
