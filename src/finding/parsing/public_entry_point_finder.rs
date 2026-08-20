// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use syn::FnArg;
use syn::ImplItem;
use syn::Item;
use syn::Signature;
use syn::TraitItem;
use syn::Visibility;
use syn::parse_file;

use crate::finding::model::public_entry_point::PublicEntryPoint;
use crate::source_file::SourceFile;

// Everything a source file exposes that a test could call.
//
// Three shapes count. A free `pub fn`. A `pub fn` in an inherent impl block. And
// every method of a `pub trait`, which needs no `pub` of its own because a
// trait's methods are as public as the trait.
//
// A method implementing a trait is deliberately not counted. It carries no
// visibility of its own and is reached through the trait rather than named
// directly, so requiring a test to call it by name would ask for something the
// caller does not usually write.
pub struct PublicEntryPointFinder;

impl PublicEntryPointFinder {
    pub fn find(file: &SourceFile) -> Option<Vec<PublicEntryPoint>> {
        let syntax = parse_file(&file.contents()).ok()?;
        Some(
            syntax
                .items
                .iter()
                .flat_map(Self::of_item)
                .collect::<Vec<PublicEntryPoint>>(),
        )
    }

    fn of_item(item: &Item) -> Vec<PublicEntryPoint> {
        match item {
            Item::Fn(function) if Self::is_public(&function.vis) => {
                vec![Self::of_signature(&function.sig)]
            }
            Item::Impl(block) if block.trait_.is_none() => block
                .items
                .iter()
                .filter_map(|inner| match inner {
                    ImplItem::Fn(method) if Self::is_public(&method.vis) => {
                        Some(Self::of_signature(&method.sig))
                    }
                    _ => None,
                })
                .collect(),
            Item::Trait(declared) if Self::is_public(&declared.vis) => declared
                .items
                .iter()
                .filter_map(|inner| match inner {
                    TraitItem::Fn(method) => Some(Self::of_signature(&method.sig)),
                    _ => None,
                })
                .collect(),
            Item::Mod(module) => module
                .content
                .as_ref()
                .map(|(_, inner)| inner.iter().flat_map(Self::of_item).collect())
                .unwrap_or_default(),
            _ => Vec::new(),
        }
    }

    fn of_signature(signature: &Signature) -> PublicEntryPoint {
        let arity = signature
            .inputs
            .iter()
            .filter(|input| !matches!(input, FnArg::Receiver(_)))
            .count();
        PublicEntryPoint::new(&signature.ident.to_string(), arity)
    }

    fn is_public(visibility: &Visibility) -> bool {
        matches!(visibility, Visibility::Public(_))
    }
}
