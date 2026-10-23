// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use syn::Fields;
use syn::Item;
use syn::ItemStruct;
use syn::Visibility;
use syn::parse_file;

use crate::finding::parsing::unit_test_finder::UnitTestFinder;
use crate::reporting::offence::Offence;
use crate::reporting::rule_explanation::RuleExplanation;
use crate::rule::Rule;
use crate::source_file::SourceFile;

// A struct's fields are private.
//
// A public field lets every file read the state and write it, so the type keeps no invariant of its own, and a
// change of representation breaks every reader at once. `pub(crate)`, `pub(super)` and `pub(in ...)` are the same
// with a shorter list of readers. Reading goes through a getter, building through a constructor -- or, past seven
// fields, a `Default` and `with_<field>` steps -- and a change of state through a method that says what it does.
//
// Each struct under src/ with a field that is not private is reported once, at its name, naming the fields, a tuple
// struct's by position. `pub(self)` already means private. Enum variants' fields are public by the language and not
// this rule's; tests/ and `#[cfg(test)]` code are test support rather than an API.
//
// See [R035-ADR-PrivateFieldsRule](../../../../docs/ADRs/R035-ADR-PrivateFieldsRule.md).
pub struct PrivateFieldsRule;

impl PrivateFieldsRule {
    pub const NAME: &'static str = "private-fields";
    pub const SOURCE_ROOT: &'static str = "src/";
    // Past three, a description names the first two and counts the rest.
    const NAMED_IN_FULL: usize = 3;

    pub fn new() -> Self {
        Self
    }

    fn correction(exposed: usize) -> String {
        let pronoun = if exposed == 1 { "it" } else { "them" };
        format!("make {pronoun} private: give a reader a getter and a builder a constructor")
    }

    fn exposed(fields: &Fields) -> Vec<String> {
        fields
            .iter()
            .enumerate()
            .filter(|(_, field)| Self::is_exposed(&field.vis))
            .map(|(at, field)| {
                field
                    .ident
                    .as_ref()
                    .map_or_else(|| at.to_string(), ToString::to_string)
            })
            .collect()
    }

    fn is_exposed(visibility: &Visibility) -> bool {
        match visibility {
            Visibility::Inherited => false,
            Visibility::Restricted(restriction) => !restriction.path.is_ident("self"),
            Visibility::Public(_) => true,
        }
    }

    fn listed(names: &[String]) -> String {
        let quoted: Vec<String> = names.iter().map(|name| format!("`{name}`")).collect();
        match quoted.as_slice() {
            [only] => only.clone(),
            [first, second, rest @ ..] if quoted.len() > Self::NAMED_IN_FULL => {
                format!("{first}, {second} and {} more fields", rest.len())
            }
            [leading @ .., last] => format!("{} and {last}", leading.join(", ")),
            [] => String::new(),
        }
    }

    fn offence(path: &str, declared: &ItemStruct) -> Option<Offence> {
        if declared.attrs.iter().any(UnitTestFinder::is_cfg_test) {
            return None;
        }
        let exposed = Self::exposed(&declared.fields);
        if exposed.is_empty() {
            return None;
        }
        let name = declared.ident.to_string();
        Some(
            Offence::new(
                path,
                declared.ident.span().start().line,
                Self::NAME,
                format!(
                    "`{name}` exposes {}; a struct's fields are private",
                    Self::listed(&exposed)
                ),
                Self::correction(exposed.len()),
            )
            .with_subject(&name),
        )
    }

    fn offences_in(path: &str, items: &[Item]) -> Vec<Offence> {
        items
            .iter()
            .flat_map(|item| match item {
                Item::Struct(declared) => Self::offence(path, declared).into_iter().collect(),
                Item::Mod(module) if !module.attrs.iter().any(UnitTestFinder::is_cfg_test) => {
                    module
                        .content
                        .as_ref()
                        .map(|(_, inner)| Self::offences_in(path, inner))
                        .unwrap_or_default()
                }
                _ => Vec::new(),
            })
            .collect()
    }
}

impl Default for PrivateFieldsRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for PrivateFieldsRule {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn check(&self, file: &SourceFile) -> Vec<Offence> {
        let path = file.relative_path();
        if !path.starts_with(Self::SOURCE_ROOT) {
            return Vec::new();
        }
        let Ok(syntax) = parse_file(&file.contents()) else {
            return Vec::new();
        };
        Self::offences_in(path, &syntax.items)
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
            "A struct's fields are private.",
            "pub struct Term {\n    pub number: u64,\n}",
            "pub struct Term {\n    number: u64,\n}\n\nimpl Term {\n    pub fn number(&self) -> u64 {\n        self.number\n    }\n}",
        )
    }
}
