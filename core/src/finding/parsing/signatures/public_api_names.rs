// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use syn::Fields;
use syn::ImplItem;
use syn::Item;
use syn::ItemImpl;
use syn::ItemTrait;
use syn::Signature;
use syn::TraitItem;
use syn::Type;
use syn::Visibility;
use syn::parse_file;

use crate::finding::parsing::impl_blocks::ImplBlocks;
use crate::finding::parsing::read_names::ReadNames;
use crate::finding::parsing::source_items::SourceItems;
use crate::finding::parsing::unit_test_finder::UnitTestFinder;
use crate::source_file::SourceFile;

// The names a crate's public API hands another crate: what another crate can be
// given, and so may depend on. The signature of a pub function, of a pub type's
// pub method and of a pub trait's methods; the type of a pub struct's pub field;
// a pub enum's variants; a pub alias, constant or static. A pub method on a type
// the crate keeps to itself hands nothing over. Read by syntax: `pub` only,
// `pub(crate)` and narrower being the crate's own, and trait impls left out --
// their signatures are the trait's.
pub struct PublicApiNames {
    names: BTreeSet<String>,
    public_types: BTreeSet<String>,
}

impl PublicApiNames {
    pub fn of(files: &[SourceFile]) -> BTreeSet<String> {
        let parsed: Vec<Vec<Item>> = files
            .iter()
            .filter_map(|file| parse_file(&file.contents()).ok().map(|syntax| syntax.items))
            .collect();
        let public_types = parsed
            .iter()
            .flat_map(|items| SourceItems::live(items))
            .filter_map(Self::public_type)
            .collect();
        parsed
            .iter()
            .fold(
                Self {
                    names: BTreeSet::new(),
                    public_types,
                },
                |mut found, items| {
                    found.read_items(items);
                    found
                },
            )
            .names
    }

    fn is_public(visibility: &Visibility) -> bool {
        matches!(visibility, Visibility::Public(_))
    }

    fn public_type(item: &Item) -> Option<String> {
        match item {
            Item::Struct(declared) if Self::is_public(&declared.vis) => {
                Some(declared.ident.to_string())
            }
            Item::Enum(declared) if Self::is_public(&declared.vis) => {
                Some(declared.ident.to_string())
            }
            _ => None,
        }
    }

    fn read_fields(&mut self, fields: &Fields, every: bool) {
        fields
            .iter()
            .filter(|field| every || Self::is_public(&field.vis))
            .for_each(|field| self.read_type(&field.ty));
    }

    fn read_impl(&mut self, block: &ItemImpl) {
        let public = ImplBlocks::target(&block.self_ty)
            .is_some_and(|target| self.public_types.contains(&target));
        if block.trait_.is_some() || !public {
            return;
        }
        block.items.iter().for_each(|member| {
            if let ImplItem::Fn(method) = member
                && Self::is_public(&method.vis)
            {
                self.read_signature(&method.sig);
            }
        });
    }

    fn read_item(&mut self, item: &Item) {
        match item {
            Item::Fn(function) if Self::is_public(&function.vis) => {
                self.read_signature(&function.sig);
            }
            Item::Impl(block) => self.read_impl(block),
            Item::Trait(declared) if Self::is_public(&declared.vis) => self.read_trait(declared),
            Item::Struct(declared) if Self::is_public(&declared.vis) => {
                self.read_fields(&declared.fields, false);
            }
            Item::Enum(declared) if Self::is_public(&declared.vis) => declared
                .variants
                .iter()
                .for_each(|variant| self.read_fields(&variant.fields, true)),
            Item::Type(alias) if Self::is_public(&alias.vis) => self.read_type(&alias.ty),
            Item::Const(constant) if Self::is_public(&constant.vis) => {
                self.read_type(&constant.ty);
            }
            Item::Static(declared) if Self::is_public(&declared.vis) => {
                self.read_type(&declared.ty);
            }
            Item::Mod(module) if !module.attrs.iter().any(UnitTestFinder::is_cfg_test) => {
                if let Some((_, inner)) = &module.content {
                    self.read_items(inner);
                }
            }
            _ => {}
        }
    }

    fn read_items(&mut self, items: &[Item]) {
        items.iter().for_each(|item| self.read_item(item));
    }

    fn read_signature(&mut self, signature: &Signature) {
        self.names.extend(ReadNames::in_signature(signature));
    }

    fn read_trait(&mut self, declared: &ItemTrait) {
        declared.items.iter().for_each(|member| match member {
            TraitItem::Fn(method) => self.read_signature(&method.sig),
            TraitItem::Const(constant) => self.read_type(&constant.ty),
            TraitItem::Type(alias) => {
                if let Some((_, default)) = &alias.default {
                    self.read_type(default);
                }
            }
            _ => {}
        });
    }

    fn read_type(&mut self, declared: &Type) {
        self.names.extend(ReadNames::in_type(declared));
    }
}
