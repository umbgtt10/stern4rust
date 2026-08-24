// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use syn::Item;
use syn::ItemMod;
use syn::ItemTrait;
use syn::TraitItem;
use syn::TraitItemFn;
use syn::parse_file;

use crate::reporting::offence::Offence;
use crate::reporting::rule_explanation::RuleExplanation;
use crate::rule::Rule;
use crate::source_file::SourceFile;

// A trait declares; it does not implement.
//
// A default body reads as a convenience and works as a decision nobody made.
// The implementor that says nothing about a method is indistinguishable from the
// one that considered it and found the default right, so the question of which
// of the two you are looking at cannot be answered by reading either file. Make
// the body a declaration and every implementor has to answer, in its own file,
// where the answer is.
//
// The other half of the requirement -- that every implementor implements every
// method -- needs no rule. With no default to fall back on, `rustc` refuses to
// compile an incomplete impl (`E0046`), immediately and more precisely than this
// tool could. Only the half the compiler is silent about is checked here.
//
// Only methods are reported. An associated type and an associated constant may
// carry a default without any of this being true of them: neither is behaviour,
// so neither lets an implementor inherit a decision while appearing to have made
// one.
//
// tests/ is exempt. A test file declares traits to stand in for real ones, and
// a stand-in with a body is the shape those fakes are supposed to have.
pub struct PureTraitsRule;

impl PureTraitsRule {
    pub const SOURCE_ROOT: &'static str = "src/";

    pub fn new() -> Self {
        Self
    }

    fn applies_to(file: &SourceFile) -> bool {
        file.relative_path().starts_with(Self::SOURCE_ROOT)
    }

    // Descends into inline modules: a body does not stop being a body for
    // sitting one level down.
    fn declarations(items: &[Item]) -> Vec<&ItemTrait> {
        items
            .iter()
            .flat_map(|item| match item {
                Item::Trait(declaration) => vec![declaration],
                Item::Mod(module) => Self::inside(module)
                    .map(Self::declarations)
                    .unwrap_or_default(),
                _ => Vec::new(),
            })
            .collect()
    }

    fn inside(module: &ItemMod) -> Option<&[Item]> {
        module.content.as_ref().map(|(_, items)| items.as_slice())
    }

    // Every body is reported rather than the trait once, because each one is a
    // separate edit in a different set of files.
    fn bodies(&self, file: &SourceFile, declaration: &ItemTrait) -> Vec<Offence> {
        let declared_by = declaration.ident.to_string();
        declaration
            .items
            .iter()
            .filter_map(|item| match item {
                TraitItem::Fn(method) if method.default.is_some() => Some(method),
                _ => None,
            })
            .map(|method| self.offence(file, &declared_by, method))
            .collect()
    }

    // Reported against the method rather than the trait, because the body is
    // what has to go.
    fn offence(&self, file: &SourceFile, declared_by: &str, method: &TraitItemFn) -> Offence {
        let subject = format!("{declared_by}::{}", method.sig.ident);
        Offence::new(
            file.relative_path(),
            method.sig.ident.span().start().line,
            self.name(),
            format!(
                "`{subject}` has a default body, so an implementor that says nothing about it \
                 cannot be told from one that chose it"
            ),
            "move the body into each implementor".to_string(),
        )
        .with_subject(&subject)
    }
}

impl Default for PureTraitsRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for PureTraitsRule {
    fn name(&self) -> &'static str {
        "pure-traits"
    }

    fn check(&self, file: &SourceFile) -> Vec<Offence> {
        if !Self::applies_to(file) {
            return Vec::new();
        }
        let Ok(syntax) = parse_file(&file.contents()) else {
            return Vec::new();
        };
        Self::declarations(&syntax.items)
            .into_iter()
            .flat_map(|declaration| self.bodies(file, declaration))
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
            "A trait declares; it does not implement.",
            "trait Store {\n    fn commit(&self) -> bool {\n        true\n    }\n}",
            "trait Store {\n    fn commit(&self) -> bool;\n}",
        )
    }
}
