// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::string::ToString;
use alloc::sync::Arc;
use alloc::vec::Vec;

use syn::Item;
use syn::ItemTrait;
use syn::TypeParamBound;
use syn::parse_file;

use crate::finding::parsing::shapes::value_traits::ValueTraits;
use crate::finding::parsing::source_items::SourceItems;
use crate::source_file::SourceFile;

// The traits that say only how a value is copied, compared or printed: the
// value traits, and every marker trait the files declare -- a trait with no
// items whose every supertrait is a value trait or another such marker, as
// `trait PeerId: Copy + Eq + Debug + Send {}`. A type parameter bounded by
// nothing else is data a type carries, not a seam it opens.
//
// Read from the whole workspace, as a marker one member declares bounds the
// parameters of another; a trait no file declares is not one.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ValueMarkers {
    markers: Arc<BTreeSet<String>>,
}

impl ValueMarkers {
    // The markers settle as a fixpoint: a marker over a marker is one once the
    // marker it rests on is, whichever file declares either.
    pub fn of(files: &[SourceFile]) -> Self {
        let candidates: BTreeMap<String, Vec<String>> = files
            .iter()
            .filter_map(|file| parse_file(&file.contents()).ok())
            .flat_map(|syntax| {
                SourceItems::live(&syntax.items)
                    .into_iter()
                    .filter_map(|item| match item {
                        Item::Trait(declared) if declared.items.is_empty() => {
                            Some((declared.ident.to_string(), Self::supertraits(declared)))
                        }
                        _ => None,
                    })
                    .collect::<Vec<_>>()
            })
            .collect();
        let mut markers = BTreeSet::new();
        loop {
            let settled: BTreeSet<String> = candidates
                .iter()
                .filter(|(_, supertraits)| {
                    supertraits
                        .iter()
                        .all(|name| ValueTraits::contains(name) || markers.contains(name))
                })
                .map(|(name, _)| name.clone())
                .collect();
            if settled == markers {
                break;
            }
            markers = settled;
        }
        Self {
            markers: Arc::new(markers),
        }
    }

    pub fn is_value(&self, name: &str) -> bool {
        ValueTraits::contains(name) || self.markers.contains(name)
    }

    fn supertraits(declared: &ItemTrait) -> Vec<String> {
        declared
            .supertraits
            .iter()
            .filter_map(|bound| match bound {
                TypeParamBound::Trait(named) => named
                    .path
                    .segments
                    .last()
                    .map(|last| last.ident.to_string()),
                _ => None,
            })
            .collect()
    }
}
