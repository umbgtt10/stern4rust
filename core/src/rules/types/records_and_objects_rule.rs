// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use syn::Fields;
use syn::Item;
use syn::ItemStruct;
use syn::Visibility;
use syn::parse_file;

use crate::finding::parsing::shapes::shaped_type::ShapedType;
use crate::finding::parsing::shapes::type_shape::TypeShape;
use crate::finding::parsing::shapes::type_shapes::TypeShapes;
use crate::finding::parsing::unit_test_finder::UnitTestFinder;
use crate::reporting::offence::Offence;
use crate::reporting::rule_explanation::RuleExplanation;
use crate::rule::Rule;
use crate::source_file::SourceFile;

// Every struct is a record or an object, and each has one shape.
//
// A record has no behaviour -- no inherent method but getters, plain constructors and `with_` steps, no impl of a
// trait outside the value traits -- so it has no invariant of its own to keep: its fields are all `pub` and it has no
// accessors, being built by literal. An object has behaviour, which privacy protects: its fields are all private,
// `pub(self)` counting as private. Which a struct is depends on its impl blocks wherever they are in the package, so
// the rule reads the package at once.
//
// Each struct under src/ is reported once, at its name: a record with accessors as getter-only, whatever its fields;
// a record with a field that is not `pub`; an object with a field that is not private, a tuple struct's by position.
// Enum variants' fields are public by the language and not this rule's; tests/ and `#[cfg(test)]` code are test
// support rather than an API.
//
// See [R035-ADR-RecordsAndObjectsRule](../../../../docs/ADRs/R035-ADR-RecordsAndObjectsRule.md).
pub struct RecordsAndObjectsRule;

impl RecordsAndObjectsRule {
    pub const NAME: &'static str = "records-and-objects";
    pub const SOURCE_ROOT: &'static str = "src/";
    // Past three, a description names the first two and counts the rest.
    const NAMED_IN_FULL: usize = 3;

    pub fn new() -> Self {
        Self
    }

    fn counted(count: usize, noun: &str) -> Option<String> {
        match count {
            0 => None,
            1 => Some(format!("1 {noun}")),
            _ => Some(format!("{count} {noun}s")),
        }
    }

    fn exposing(path: &str, declared: &ItemStruct) -> Option<Offence> {
        let exposed = Self::named(&declared.fields, Self::is_exposed);
        let pronoun = Self::pronoun(exposed.len());
        (!exposed.is_empty()).then(|| {
            Self::reported(
                path,
                declared,
                format!(
                    "`{}` is an object exposing {}; an object's fields are private",
                    declared.ident,
                    Self::listed(&exposed)
                ),
                format!(
                    "make {pronoun} private: give a reader a getter and a builder a constructor"
                ),
            )
        })
    }

    fn getter_only(path: &str, declared: &ItemStruct, shaped: &ShapedType) -> Offence {
        let counts = [
            Self::counted(shaped.getters().len(), "getter"),
            Self::counted(shaped.constructors(), "constructor"),
            Self::counted(shaped.steps(), "`with_` step"),
        ];
        let kinds = [
            Self::kind(shaped.getters().len(), "getter"),
            Self::kind(shaped.constructors(), "constructor"),
            Self::kind(shaped.steps(), "step"),
        ];
        let described: Vec<String> = counts
            .into_iter()
            .flatten()
            .chain([String::from("no behaviour")])
            .collect();
        let deleted: Vec<String> = kinds.into_iter().flatten().collect();
        Self::reported(
            path,
            declared,
            format!(
                "`{}` is getter-only -- {}",
                declared.ident,
                Self::joined(&described)
            ),
            format!(
                "delete the {}, make the fields pub and build it by literal",
                Self::joined(&deleted)
            ),
        )
    }

    fn hiding(path: &str, declared: &ItemStruct) -> Option<Offence> {
        let hidden = Self::named(&declared.fields, Self::is_hidden);
        let pronoun = Self::pronoun(hidden.len());
        let fields = if hidden.len() == 1 {
            "a private field"
        } else {
            "private fields"
        };
        (!hidden.is_empty()).then(|| {
            Self::reported(
                path,
                declared,
                format!(
                    "`{}` is a record with {fields} {}; a record's fields are public",
                    declared.ident,
                    Self::listed(&hidden)
                ),
                format!("make {pronoun} pub: a struct with no behaviour has no invariant to keep"),
            )
        })
    }

    fn is_exposed(visibility: &Visibility) -> bool {
        match visibility {
            Visibility::Inherited => false,
            Visibility::Restricted(restriction) => !restriction.path.is_ident("self"),
            Visibility::Public(_) => true,
        }
    }

    fn is_hidden(visibility: &Visibility) -> bool {
        !matches!(visibility, Visibility::Public(_))
    }

    fn joined(parts: &[String]) -> String {
        match parts {
            [only] => only.clone(),
            [leading @ .., last] => format!("{} and {last}", leading.join(", ")),
            [] => String::new(),
        }
    }

    fn kind(count: usize, noun: &str) -> Option<String> {
        match count {
            0 => None,
            1 => Some(noun.to_string()),
            _ => Some(format!("{noun}s")),
        }
    }

    fn listed(names: &[String]) -> String {
        let quoted: Vec<String> = names.iter().map(|name| format!("`{name}`")).collect();
        match quoted.as_slice() {
            [first, second, rest @ ..] if quoted.len() > Self::NAMED_IN_FULL => {
                format!("{first}, {second} and {} more fields", rest.len())
            }
            _ => Self::joined(&quoted),
        }
    }

    fn named(fields: &Fields, selected: fn(&Visibility) -> bool) -> Vec<String> {
        fields
            .iter()
            .enumerate()
            .filter(|(_, field)| selected(&field.vis))
            .map(|(at, field)| {
                field
                    .ident
                    .as_ref()
                    .map_or_else(|| at.to_string(), ToString::to_string)
            })
            .collect()
    }

    fn offence(path: &str, declared: &ItemStruct, shapes: &TypeShapes) -> Option<Offence> {
        if declared.attrs.iter().any(UnitTestFinder::is_cfg_test) {
            return None;
        }
        let name = declared.ident.to_string();
        let shaped = shapes
            .types()
            .iter()
            .find(|shaped| shaped.path() == path && shaped.name() == name)?;
        match shaped.shape() {
            TypeShape::Record if shaped.accessors() > 0 => {
                Some(Self::getter_only(path, declared, shaped))
            }
            TypeShape::Record => Self::hiding(path, declared),
            TypeShape::Builder | TypeShape::Object => Self::exposing(path, declared),
        }
    }

    fn offences_in(path: &str, items: &[Item], shapes: &TypeShapes) -> Vec<Offence> {
        items
            .iter()
            .flat_map(|item| match item {
                Item::Struct(declared) => {
                    Self::offence(path, declared, shapes).into_iter().collect()
                }
                Item::Mod(module) if !module.attrs.iter().any(UnitTestFinder::is_cfg_test) => {
                    module
                        .content
                        .as_ref()
                        .map(|(_, inner)| Self::offences_in(path, inner, shapes))
                        .unwrap_or_default()
                }
                _ => Vec::new(),
            })
            .collect()
    }

    fn pronoun(count: usize) -> &'static str {
        if count == 1 { "it" } else { "them" }
    }

    fn reported(
        path: &str,
        declared: &ItemStruct,
        description: String,
        correction: String,
    ) -> Offence {
        Offence::new(
            path,
            declared.ident.span().start().line,
            Self::NAME,
            description,
            correction,
        )
        .with_subject(&declared.ident.to_string())
    }
}

impl Default for RecordsAndObjectsRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for RecordsAndObjectsRule {
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
        sources
            .iter()
            .filter_map(|file| {
                parse_file(&file.contents())
                    .ok()
                    .map(|syntax| Self::offences_in(file.relative_path(), &syntax.items, &shapes))
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
            summary: "A record's fields are public and it has no accessors; an object's fields are private.",
            breaks: "pub struct Term {\n    number: u64,\n}\n\nimpl Term {\n    pub fn number(&self) -> u64 {\n        self.number\n    }\n}",
            instead: "pub struct Term {\n    pub number: u64,\n}",
        }
    }
}
