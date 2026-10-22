// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use syn::Item;
use syn::ItemConst;
use syn::VisRestricted;
use syn::Visibility;
use syn::parse_file;

use crate::finding::model::name_readers::NameReaders;
use crate::reporting::offence::Offence;
use crate::reporting::rule_explanation::RuleExplanation;
use crate::rule::Rule;
use crate::source_file::SourceFile;

// A module-level constant is private, or public and shared from a constants
// file.
//
// A constant has two honest states, as a method does under public-or-private.
// It is one file's, private and read where it is declared; or it is a fact
// several files share, public, and then its home is not any one of its readers
// but a file of constants a reader finds by name -- one holding only imports,
// constants, statics and type aliases. So a `pub` constant is reported when no
// other file reads it, when one does, which could hold it privately, and when
// two or more do but it sits beside a type or a function. Restricted
// visibility is the third state, reported on its own.
//
// Readers are counted across the workspace, tests/ and sibling crates
// included, by name -- the runner hands every package the same index -- since a
// constant only a sibling crate reads is shared all the same. Statics are state
// and live with their owner; associated constants are their type's; inline
// modules are not descended into; registries and tests/ are exempt.
pub struct SharedConstantsRule {
    readers: NameReaders,
}

impl SharedConstantsRule {
    pub const NAME: &'static str = "shared-constants";
    pub const REGISTRIES: [&'static str; 2] = ["lib.rs", "mod.rs"];
    pub const SOURCE_ROOT: &'static str = "src/";
    const SHARED: &'static str = "move it into a constants file -- imports, constants, statics and \
                                  type aliases only --, such as `<topic>_constants.rs`";

    pub fn new(readers: NameReaders) -> Self {
        Self { readers }
    }

    fn is_constants_file(items: &[Item]) -> bool {
        items.iter().all(|item| {
            matches!(
                item,
                Item::Use(_) | Item::Const(_) | Item::Static(_) | Item::Type(_)
            )
        })
    }

    fn judged(
        &self,
        file: &SourceFile,
        constant: &ItemConst,
        in_constants_file: bool,
    ) -> Option<Offence> {
        match &constant.vis {
            Visibility::Inherited => None,
            Visibility::Restricted(restricted) => {
                Some(Self::restricted(file, constant, restricted))
            }
            Visibility::Public(_) => self.public(file, constant, in_constants_file),
        }
    }

    fn offence(
        file: &SourceFile,
        constant: &ItemConst,
        description: String,
        correction: String,
    ) -> Offence {
        Offence::new(
            file.relative_path(),
            constant.ident.span().start().line,
            Self::NAME,
            description,
            correction,
        )
        .with_subject(&constant.ident.to_string())
    }

    fn public(
        &self,
        file: &SourceFile,
        constant: &ItemConst,
        in_constants_file: bool,
    ) -> Option<Offence> {
        let name = constant.ident.to_string();
        let others = self.readers.others(&name, file.relative_path());
        match others.as_slice() {
            [] => Some(Self::offence(
                file,
                constant,
                format!("`{name}` is public, and no other file reads it"),
                String::from("make it private, or delete it if nothing reads it"),
            )),
            [only] if !in_constants_file && self.readers.reads(&name, file.relative_path()) => {
                Some(Self::offence(
                    file,
                    constant,
                    format!(
                        "`{name}` is read here and by `{only}`, from a file that is not a \
                         constants file"
                    ),
                    format!(
                        "{}, or make it private if `{only}` can do without it",
                        Self::SHARED
                    ),
                ))
            }
            [only] => Some(Self::offence(
                file,
                constant,
                format!("`{name}` is public, and only `{only}` reads it"),
                format!("move it into `{only}` and make it private"),
            )),
            _ if in_constants_file => None,
            shared => Some(Self::offence(
                file,
                constant,
                format!(
                    "`{name}` is read by {} other files, from a file that is not a constants file",
                    shared.len()
                ),
                String::from(Self::SHARED),
            )),
        }
    }

    fn restricted(file: &SourceFile, constant: &ItemConst, restricted: &VisRestricted) -> Offence {
        let path = restricted
            .path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>()
            .join("::");
        let shown = match restricted.in_token {
            Some(_) => format!("pub(in {path})"),
            None => format!("pub({path})"),
        };
        Self::offence(
            file,
            constant,
            format!(
                "`{}` is `{shown}`; a constant is private, or public and shared",
                constant.ident
            ),
            String::from("make it private, or `pub` and shared from a constants file"),
        )
    }
}

impl Rule for SharedConstantsRule {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn check(&self, file: &SourceFile) -> Vec<Offence> {
        if !file.is_under(Self::SOURCE_ROOT, &Self::REGISTRIES) {
            return Vec::new();
        }
        let Ok(syntax) = parse_file(&file.contents()) else {
            return Vec::new();
        };
        let in_constants_file = Self::is_constants_file(&syntax.items);
        syntax
            .items
            .iter()
            .filter_map(|item| match item {
                Item::Const(constant) => self.judged(file, constant, in_constants_file),
                _ => None,
            })
            .collect()
    }

    fn check_workspace(&self, _files: &[SourceFile]) -> Vec<Offence> {
        Vec::new()
    }

    fn requirement(&self) -> Option<&'static str> {
        None
    }

    fn is_configured(&self) -> bool {
        true
    }

    fn explanation(&self) -> RuleExplanation {
        RuleExplanation::new(
            self.name(),
            "A module-level constant is private, or public and shared from a constants file.",
            "pub const LIMIT: u8 = 3;\n\npub struct Pool;",
            "const LIMIT: u8 = 3;\n\npub struct Pool;",
        )
    }
}
