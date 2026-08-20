// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::iter::once;

use syn::Attribute;
use syn::Expr;
use syn::Item;
use syn::ItemMod;
use syn::Lit;
use syn::Meta;
use syn::parse_file;

use crate::reporting::offence::Offence;
use crate::rule::Rule;
use crate::source_file::SourceFile;

// A module is declared by name: `mod alpha;` reaches `alpha.rs` or
// `alpha/mod.rs`, and nothing else decides which file that is.
//
// This is not a rule about taste. `#[path = "..."]` is the one attribute that
// makes another rule here give a **confident wrong answer**:
// `registry-completeness` resolves a declaration to the file it names by
// convention, so a file reached through an explicit path is reported as never
// compiled when it compiles perfectly well. That rule accepted the gap on the
// grounds that the house standard forbids `#[path]` and nothing in the family
// uses it -- a convention nothing enforced. This enforces it.
//
// It applies to the whole package rather than to registries. The house standard
// names `all_tests.rs` because that is where the temptation is, but the harm is
// the same wherever the attribute appears: a `mod` in an ordinary source file is
// resolved by the same convention and misread in the same way.
//
// `#[cfg_attr(unix, path = "...")]` is deliberately left alone. A platform-gated
// module is the one honest use of the attribute, it cannot resolve by name on
// every platform anyway, and reporting it would accuse correct code -- the
// direction every rule here refuses to lean.
pub struct DeclaredByNameRule;

impl DeclaredByNameRule {
    pub const ATTRIBUTE: &'static str = "path";

    pub fn new() -> Self {
        Self
    }

    // Descends into inline modules: an attribute one level down is as invisible
    // to name resolution as one at the top.
    fn declarations(items: &[Item]) -> Vec<&ItemMod> {
        items
            .iter()
            .flat_map(|item| match item {
                Item::Mod(module) => once(module)
                    .chain(
                        Self::inside(module)
                            .map(Self::declarations)
                            .unwrap_or_default(),
                    )
                    .collect::<Vec<&ItemMod>>(),
                _ => Vec::new(),
            })
            .collect()
    }

    fn inside(module: &ItemMod) -> Option<&[Item]> {
        module.content.as_ref().map(|(_, items)| items.as_slice())
    }

    // Only the bare `#[path = "..."]`. A `cfg_attr` wrapping one is a
    // `Meta::List` named `cfg_attr`, so it never matches here.
    fn target_of(attrs: &[Attribute]) -> Option<String> {
        attrs.iter().find_map(|attr| match &attr.meta {
            Meta::NameValue(pair) if pair.path.is_ident(Self::ATTRIBUTE) => match &pair.value {
                Expr::Lit(literal) => match &literal.lit {
                    Lit::Str(text) => Some(text.value()),
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        })
    }

    // The correction names both files, because the fix is a move and a deletion
    // rather than an edit to the declaration.
    fn offence(&self, file: &SourceFile, module: &ItemMod, target: &str) -> Offence {
        let name = module.ident.to_string();
        let expected = format!("{name}.rs");
        Offence::new(
            file.relative_path(),
            module.ident.span().start().line,
            self.name(),
            format!(
                "`mod {name}` is reached through `#[path = \"{target}\"]`, so the file it \
                 declares cannot be found from its name"
            ),
            format!(
                "move `{target}` to `{expected}` beside this file and drop the `#[path]` attribute"
            ),
        )
        .with_subject(&name)
        .with_expected(&expected)
    }
}

impl Default for DeclaredByNameRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for DeclaredByNameRule {
    fn name(&self) -> &'static str {
        "declared-by-name"
    }

    fn check(&self, file: &SourceFile) -> Vec<Offence> {
        let Ok(syntax) = parse_file(&file.contents()) else {
            return Vec::new();
        };
        Self::declarations(&syntax.items)
            .into_iter()
            .filter_map(|module| {
                Self::target_of(&module.attrs).map(|target| self.offence(file, module, &target))
            })
            .collect()
    }

    fn check_workspace(&self, _files: &[SourceFile]) -> Vec<Offence> {
        Vec::new()
    }

    fn is_configured(&self) -> bool {
        true
    }
}
