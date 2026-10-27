// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use syn::FnArg;
use syn::Generics;
use syn::ImplItem;
use syn::ImplItemFn;
use syn::Item;
use syn::ItemImpl;
use syn::ItemStruct;
use syn::Signature;
use syn::Type;
use syn::parse_file;
use syn::spanned::Spanned;

use crate::finding::parsing::dependencies::built_text::BuiltText;
use crate::finding::parsing::dependencies::dependency_fills::DependencyFills;
use crate::finding::parsing::dependencies::passed_in::PassedIn;
use crate::finding::parsing::dependencies::seam_types::SeamTypes;
use crate::finding::parsing::impl_blocks::ImplBlocks;
use crate::finding::parsing::impl_layout_parser::ImplLayoutParser;
use crate::finding::parsing::shapes::type_shape::TypeShape;
use crate::finding::parsing::shapes::type_shapes::TypeShapes;
use crate::finding::parsing::source_items::SourceItems;
use crate::reporting::offence::Offence;
use crate::reporting::rule_explanation::RuleExplanation;
use crate::rule::Rule;
use crate::source_file::SourceFile;

// A struct with a field declared as a trait: those fields, by name or position,
// the generics their bounds come from, and the constructors another constructor
// may hand a part to.
struct SeamedType {
    pub fields: BTreeSet<String>,
    pub generics: Generics,
    pub constructors: BTreeMap<String, Signature>,
}

// A type does not build its own dependencies: a field declared as a trait
// receives only what the caller passed in.
//
// A field declared as a trait is a seam -- the type promises its part can be
// swapped -- and a constructor that builds the part itself closes it: the type
// is testable only through some other constructor, and hides the dependency
// from every caller of this one. The parts are built where they are composed:
// main, a builder, a test.
//
// Judged in src/: every constructor, R027's -- no receiver, a return type naming
// `Self` -- and every Default, From and TryFrom impl, for the fields they fill and
// for what they hand another constructor of the type; and every `self.field =`
// in any other method. Passed in is what `PassedIn` says. A builder -- a type
// whose `build(self)` returns another type -- is the composition mechanism, and
// is not judged: its product's constructor is.
//
// See [R038-ADR-InjectedDependenciesRule](../../../../docs/ADRs/R038-ADR-InjectedDependenciesRule.md).
pub struct InjectedDependenciesRule;

impl InjectedDependenciesRule {
    pub const NAME: &'static str = "injected-dependencies";
    pub const SOURCE_ROOT: &'static str = "src/";
    const COMPOSING: [&'static str; 3] = ["Default", "From", "TryFrom"];

    pub fn new() -> Self {
        Self
    }

    fn assigned(path: &str, owner: &str, method: &ImplItemFn, seamed: &SeamedType) -> Vec<Offence> {
        let subject = format!("{owner}::{}", method.sig.ident);
        let passed = PassedIn::of(&method.sig, &method.block, owner);
        let mut reported = BTreeSet::new();
        DependencyFills::assignments(&method.block)
            .into_iter()
            .filter(|fill| seamed.fields.contains(&fill.field) && !passed.holds(&fill.value))
            .filter(|fill| reported.insert(fill.field.clone()))
            .map(|fill| {
                Self::reported(
                    path,
                    fill.line,
                    &subject,
                    format!(
                        "`{subject}` assigns `{}` to `{}`, a field declared as a trait",
                        BuiltText::of(passed.origin(&fill.value)),
                        fill.field
                    ),
                    String::from("take the replacement as a parameter"),
                )
            })
            .collect()
    }

    fn constructed(
        path: &str,
        owner: &str,
        method: &ImplItemFn,
        seamed: &SeamedType,
    ) -> Vec<Offence> {
        let subject = format!("{owner}::{}", method.sig.ident);
        let passed = PassedIn::of(&method.sig, &method.block, owner);
        let mut filled = BTreeSet::new();
        let mut handed_over = BTreeSet::new();
        let built = DependencyFills::fills(&method.block, owner)
            .into_iter()
            .filter(|fill| seamed.fields.contains(&fill.field) && !passed.holds(&fill.value))
            .filter(|fill| filled.insert(fill.field.clone()))
            .map(|fill| {
                Self::reported(
                    path,
                    fill.line,
                    &subject,
                    format!(
                        "`{subject}` builds `{}` for `{}`, a field declared as a trait",
                        BuiltText::of(passed.origin(&fill.value)),
                        fill.field
                    ),
                    String::from(
                        "take it as a parameter and build it where the parts are composed: main, a builder, a test",
                    ),
                )
            });
        let handed = DependencyFills::delegations(&method.block, owner)
            .into_iter()
            .filter(|delegation| method.sig.ident != delegation.target)
            .flat_map(|delegation| {
                let Some(target) = seamed.constructors.get(&delegation.target) else {
                    return Vec::new();
                };
                let seams = SeamTypes::of(&[&seamed.generics, &target.generics]);
                Self::parameter_types(target)
                    .into_iter()
                    .zip(&delegation.arguments)
                    .enumerate()
                    .filter(|(_, (ty, argument))| seams.is_seam(ty) && !passed.holds(argument))
                    .filter(|(at, _)| handed_over.insert((delegation.target.clone(), *at)))
                    .map(|(_, (_, argument))| {
                        Self::reported(
                            path,
                            argument.span().start().line,
                            &subject,
                            format!(
                                "`{subject}` passes `{}`, which it built, to `{owner}::{}`",
                                BuiltText::of(passed.origin(argument)),
                                delegation.target
                            ),
                            format!(
                                "make {} the constructor and compose the parts in main",
                                delegation.target
                            ),
                        )
                    })
                    .collect()
            });
        built.chain(handed).collect()
    }

    fn declared(path: &str, items: &[Item], shapes: &TypeShapes) -> Vec<(String, SeamedType)> {
        SourceItems::live(items)
            .into_iter()
            .filter_map(|item| match item {
                Item::Struct(declared) => Some(declared),
                _ => None,
            })
            .filter(|declared| {
                !shapes.types().iter().any(|shaped| {
                    shaped.path() == path
                        && declared.ident == shaped.name()
                        && shaped.shape() == TypeShape::Builder
                })
            })
            .filter_map(|declared| {
                let fields = Self::seam_fields(declared);
                (!fields.is_empty()).then(|| {
                    (
                        declared.ident.to_string(),
                        SeamedType {
                            fields,
                            generics: declared.generics.clone(),
                            constructors: BTreeMap::new(),
                        },
                    )
                })
            })
            .collect()
    }

    fn judged(path: &str, block: &ItemImpl, seamed: &BTreeMap<String, SeamedType>) -> Vec<Offence> {
        let Some(owner) = ImplBlocks::target(&block.self_ty) else {
            return Vec::new();
        };
        let Some(seamed_type) = seamed.get(&owner) else {
            return Vec::new();
        };
        let composing = block.trait_.as_ref().is_some_and(|(_, named, _)| {
            named
                .segments
                .last()
                .is_some_and(|last| Self::COMPOSING.contains(&last.ident.to_string().as_str()))
        });
        block
            .items
            .iter()
            .filter_map(|member| match member {
                ImplItem::Fn(method) => Some(method),
                _ => None,
            })
            .flat_map(|method| {
                let constructor =
                    block.trait_.is_none() && ImplLayoutParser::is_constructor(method, &owner);
                if composing || constructor {
                    Self::constructed(path, &owner, method, seamed_type)
                } else if method.sig.receiver().is_some() {
                    Self::assigned(path, &owner, method, seamed_type)
                } else {
                    Vec::new()
                }
            })
            .collect()
    }

    fn parameter_types(signature: &Signature) -> Vec<&Type> {
        signature
            .inputs
            .iter()
            .filter_map(|input| match input {
                FnArg::Typed(typed) => Some(typed.ty.as_ref()),
                FnArg::Receiver(_) => None,
            })
            .collect()
    }

    fn reported(
        path: &str,
        line: usize,
        subject: &str,
        description: String,
        correction: String,
    ) -> Offence {
        Offence::new(path, line, Self::NAME, description, correction).with_subject(subject)
    }

    fn seam_fields(declared: &ItemStruct) -> BTreeSet<String> {
        let seams = SeamTypes::of(&[&declared.generics]);
        declared
            .fields
            .iter()
            .enumerate()
            .filter(|(_, field)| seams.is_seam(&field.ty))
            .map(|(at, field)| {
                field
                    .ident
                    .as_ref()
                    .map_or_else(|| at.to_string(), ToString::to_string)
            })
            .collect()
    }

    fn seamed(parsed: &[(String, Vec<Item>)], shapes: &TypeShapes) -> BTreeMap<String, SeamedType> {
        let declared: BTreeMap<String, SeamedType> = parsed
            .iter()
            .flat_map(|(path, items)| Self::declared(path, items, shapes))
            .collect();
        parsed
            .iter()
            .flat_map(|(_, items)| ImplBlocks::inherent(items))
            .fold(declared, |mut seamed, block| {
                if let Some(owner) = ImplBlocks::target(&block.self_ty)
                    && let Some(found) = seamed.get_mut(&owner)
                {
                    let constructors = block.items.iter().filter_map(|member| match member {
                        ImplItem::Fn(method)
                            if ImplLayoutParser::is_constructor(method, &owner) =>
                        {
                            Some((method.sig.ident.to_string(), method.sig.clone()))
                        }
                        _ => None,
                    });
                    found.constructors.extend(constructors);
                }
                seamed
            })
    }
}

impl Default for InjectedDependenciesRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for InjectedDependenciesRule {
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
        let parsed: Vec<(String, Vec<Item>)> = sources
            .iter()
            .filter_map(|file| {
                parse_file(&file.contents())
                    .ok()
                    .map(|syntax| (file.relative_path().to_string(), syntax.items))
            })
            .collect();
        let seamed = Self::seamed(&parsed, &shapes);
        parsed
            .iter()
            .flat_map(|(path, items)| {
                SourceItems::live(items)
                    .into_iter()
                    .filter_map(|item| match item {
                        Item::Impl(block) => Some(block),
                        _ => None,
                    })
                    .flat_map(|block| Self::judged(path, block, &seamed))
                    .collect::<Vec<_>>()
            })
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
            summary: "A type does not build its own dependencies: a field declared as a trait receives only what the caller passed in.",
            breaks: "impl App {\n    pub fn new() -> Self {\n        Self {\n            reporter: Box::new(StdoutReporter::new()),\n        }\n    }\n}",
            instead: "impl App {\n    pub fn new(reporter: Box<dyn Reporter>) -> Self {\n        Self { reporter }\n    }\n}",
        }
    }
}
