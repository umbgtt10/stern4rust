// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use syn::Ident;
use syn::Item;
use syn::Visibility;
use syn::parse_file;

use crate::finding::model::file_stem::FileStem;
use crate::finding::model::name_readers::NameReaders;
use crate::finding::model::type_name::TypeName;
use crate::finding::parsing::impl_blocks::ImplBlocks;
use crate::finding::parsing::read_names::ReadNames;
use crate::finding::parsing::shapes::type_shape::TypeShape;
use crate::finding::parsing::shapes::type_shapes::TypeShapes;
use crate::finding::parsing::signatures::public_api_names::PublicApiNames;
use crate::finding::parsing::unit_test_finder::UnitTestFinder;
use crate::reporting::offence::Offence;
use crate::reporting::rule_explanation::RuleExplanation;
use crate::rule::Rule;
use crate::source_file::SourceFile;

// A record or a data enum lives where its consumers put it: private, inside its
// only consumer, or `pub`, in a file of its own.
//
// Nothing else decides it. A record shared by five files may sit in one of
// them, and a record only one file uses may have a file of its own; both pass
// every other rule, so which a codebase gets would be taste. A consumer is a file
// of the workspace that names the type, tests and other crates included; the
// declaring file's own code is a user too, its declaration and the type's impls
// are not. Named in the crate's public API, used nowhere, or used by two or more
// files or by any outside the crate's src/, the type is `pub`, in a file named
// after it. Used by exactly one file of src/, it is private, inside that file.
//
// A type in a binary target -- src/main.rs, src/bin/ -- has no consumer in another
// crate and no public API, since no other crate can reach it: a same-named type
// elsewhere in the workspace is only a name.
//
// Each record and data enum declared at the top of a src/ file is judged once, at
// its name. Objects, an enum with behaviour among them, are single-implemented-type's.
//
// See [R037-ADR-RecordHomeRule](../../../../docs/ADRs/R037-ADR-RecordHomeRule.md).
pub struct RecordHomeRule {
    readers: NameReaders,
}

impl RecordHomeRule {
    pub const NAME: &'static str = "record-home";
    pub const SOURCE_ROOT: &'static str = "src/";
    const BINARIES: &'static str = "src/bin/";
    const BINARY: &'static str = "src/main.rs";

    pub fn new(readers: NameReaders) -> Self {
        Self { readers }
    }

    fn belongs_to(item: &Item, name: &str) -> bool {
        match item {
            Item::Struct(declared) => declared.ident == name,
            Item::Enum(declared) => declared.ident == name,
            Item::Impl(block) => {
                ImplBlocks::target(&block.self_ty).is_some_and(|target| target == name)
            }
            Item::Mod(module) => module.attrs.iter().any(UnitTestFinder::is_cfg_test),
            _ => false,
        }
    }

    fn declared(item: &Item) -> Option<(&Ident, &Visibility)> {
        match item {
            Item::Struct(declared) if !declared.attrs.iter().any(UnitTestFinder::is_cfg_test) => {
                Some((&declared.ident, &declared.vis))
            }
            Item::Enum(declared) if !declared.attrs.iter().any(UnitTestFinder::is_cfg_test) => {
                Some((&declared.ident, &declared.vis))
            }
            _ => None,
        }
    }

    // A binary target is no other crate's to reach: its types have no consumer
    // outside it, and it has no public API.
    fn is_binary(path: &str) -> bool {
        path == Self::BINARY || path.starts_with(Self::BINARIES)
    }

    fn offence(
        &self,
        path: &str,
        items: &[Item],
        declared: &Ident,
        visibility: &Visibility,
        in_api: bool,
    ) -> Option<Offence> {
        let name = declared.to_string();
        let local = self.readers.others_under(&name, path, Self::SOURCE_ROOT);
        let outside = if Self::is_binary(path) {
            0
        } else {
            self.readers.others(&name, path).len() - local.len()
        };
        let own_use =
            ReadNames::in_items(items.iter().filter(|item| !Self::belongs_to(item, &name)))
                .contains(&name);
        let users = local.len() + outside + usize::from(own_use);
        let reason = match users {
            _ if in_api => String::from("is named in the crate's public API"),
            0 => String::from("is used nowhere"),
            1 if outside == 0 => return Self::single(path, declared, visibility, local.first()),
            1 => String::from("is used by 1 file"),
            count => format!("is used by {count} files"),
        };
        Self::shared(path, declared, visibility, &reason)
    }

    fn offences_in(
        &self,
        path: &str,
        items: &[Item],
        shapes: &TypeShapes,
        api: &BTreeSet<String>,
    ) -> Vec<Offence> {
        items
            .iter()
            .filter_map(Self::declared)
            .filter(|(declared, _)| {
                shapes.types().iter().any(|shaped| {
                    shaped.path() == path
                        && *declared == shaped.name()
                        && shaped.shape() == TypeShape::Record
                })
            })
            .filter_map(|(declared, visibility)| {
                self.offence(
                    path,
                    items,
                    declared,
                    visibility,
                    api.contains(&declared.to_string()),
                )
            })
            .collect()
    }

    fn reported(path: &str, declared: &Ident, description: String, correction: String) -> Offence {
        Offence::new(
            path,
            declared.span().start().line,
            Self::NAME,
            description,
            correction,
        )
        .with_subject(&declared.to_string())
    }

    fn shared(
        path: &str,
        declared: &Ident,
        visibility: &Visibility,
        reason: &str,
    ) -> Option<Offence> {
        let name = declared.to_string();
        if !FileStem::of(path).names(&name) {
            let file = TypeName::new(&name).file_name();
            let home = path
                .rsplit_once('/')
                .map_or(file.clone(), |(directory, _)| format!("{directory}/{file}"));
            return Some(Self::reported(
                path,
                declared,
                format!("`{name}` {reason} but declared in `{path}`"),
                format!("move {name} into {home} and make it pub"),
            ));
        }
        (!matches!(visibility, Visibility::Public(_))).then(|| {
            Self::reported(
                path,
                declared,
                format!("`{name}` {reason} but is not `pub`"),
                String::from("make it pub"),
            )
        })
    }

    fn single(
        path: &str,
        declared: &Ident,
        visibility: &Visibility,
        consumer: Option<&String>,
    ) -> Option<Offence> {
        let name = declared.to_string();
        let Some(consumer) = consumer else {
            return (!matches!(visibility, Visibility::Inherited)).then(|| {
                Self::reported(
                    path,
                    declared,
                    format!("`{name}` lives in its only consumer, `{path}`, but is `pub`"),
                    String::from("drop its pub: no other file uses it"),
                )
            });
        };
        let found = if FileStem::of(path).names(&name) {
            String::from("has a file of its own")
        } else {
            format!("declared in `{path}`")
        };
        Some(Self::reported(
            path,
            declared,
            format!("`{name}` is used only by `{consumer}` but {found}"),
            format!("move {name} into {consumer} and drop its pub"),
        ))
    }
}

impl Rule for RecordHomeRule {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn check(&self, _file: &SourceFile) -> Vec<Offence> {
        Vec::new()
    }

    fn check_workspace(&self, files: &[SourceFile]) -> Vec<Offence> {
        let sources: Vec<SourceFile> = files
            .iter()
            .filter(|file| file.relative_path().starts_with(Self::SOURCE_ROOT))
            .cloned()
            .collect();
        let shapes = TypeShapes::of(&sources);
        let libraries: Vec<SourceFile> = sources
            .iter()
            .filter(|file| !Self::is_binary(file.relative_path()))
            .cloned()
            .collect();
        let api = PublicApiNames::of(&libraries);
        sources
            .iter()
            .filter_map(|file| {
                parse_file(&file.contents()).ok().map(|syntax| {
                    self.offences_in(file.relative_path(), &syntax.items, &shapes, &api)
                })
            })
            .flatten()
            .collect()
    }

    fn requirement(&self) -> Option<&'static str> {
        None
    }

    fn is_configured(&self) -> bool {
        true
    }

    fn explanation(&self) -> RuleExplanation {
        RuleExplanation {
            name: self.name(),
            summary: "A record or a data enum lives private in its only consumer, or pub in a file of its own.",
            breaks: "// window.rs, used only by scheduler.rs\npub struct Window {\n    pub size: u32,\n}",
            instead: "// scheduler.rs\nstruct Window {\n    pub size: u32,\n}",
        }
    }
}
