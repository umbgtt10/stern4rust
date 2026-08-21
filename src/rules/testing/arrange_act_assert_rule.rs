// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::collections::BTreeSet;

use proc_macro2::TokenStream;
use proc_macro2::TokenTree;
use quote::ToTokens;
use syn::Attribute;
use syn::Item;
use syn::ItemFn;
use syn::parse_file;
use syn::spanned::Spanned;

use crate::finding::model::test_marker::MarkerPhase;
use crate::finding::model::test_marker::TestMarker;
use crate::reporting::offence::Offence;
use crate::reporting::rule_explanation::RuleExplanation;
use crate::rule::Rule;
use crate::source_file::SourceFile;

// A test reads `Arrange`, then one or more `Act`/`Assert` pairs.
//
// The name of a test says what it claims; this says whether the body is laid out
// so a reader can check the claim. It is the oldest of this tool's rules by
// intent -- the original motivating example -- and the last to be built, because
// of one problem that has nothing to do with AAA.
//
// **The markers are comments, and comments never reach the syntax tree.** `syn`
// discards them, so this rule reads lines. And a line scanner cannot tell code
// from a string that contains code -- which matters here more than anywhere,
// because this repository's own tests are built from Rust source embedded in raw
// strings. Scanning lines naively reports the fixtures rather than the tests: it
// finds seven offences in this crate that are all string literals, and the rule
// fails the gate it exists to pass.
//
// So the lines a literal occupies are taken from the token stream and skipped.
// Comments are not tokens and literals are, which is exactly the distinction
// needed. Walking the tokens rather than visiting the syntax tree also reaches
// inside macros, where `assert_eq!("// Act", x)` would otherwise hide one.
//
// The grammar is deliberately small. Every marker expands to the phases it
// names, and the expansion must read `Arrange` followed by one or more
// `Act`/`Assert` pairs. That single check covers every legal shape --
// `// Arrange & Act`, `// Act & Assert` and `// Arrange & Act & Assert` all
// expand into the same sequence as the three separate markers do -- and rejects
// a test whose Act has no Assert, whose Assert has no Act, or whose Arrange was
// dropped instead of merged.
pub struct ArrangeActAssertRule;

impl ArrangeActAssertRule {
    pub const REGISTRIES: [&'static str; 2] = ["all_tests.rs", "mod.rs"];
    pub const TESTS_ROOT: &'static str = "tests/";

    pub fn new() -> Self {
        Self
    }

    fn applies_to(file: &SourceFile) -> bool {
        let path = file.relative_path();
        path.starts_with(Self::TESTS_ROOT)
            && !path
                .rsplit('/')
                .next()
                .is_some_and(|name| Self::REGISTRIES.contains(&name))
    }

    fn is_test(attrs: &[Attribute]) -> bool {
        attrs.iter().any(|attr| {
            attr.path()
                .segments
                .last()
                .is_some_and(|segment| segment.ident == "test")
        })
    }

    // Every line any literal occupies. Comments are not tokens, so nothing a
    // marker could sit on is lost, and every line of a multi-line raw string is
    // covered.
    fn literal_lines(tokens: TokenStream) -> BTreeSet<usize> {
        tokens
            .into_iter()
            .flat_map(|tree| match tree {
                TokenTree::Group(group) => Self::literal_lines(group.stream()),
                TokenTree::Literal(literal) => {
                    let span = literal.span();
                    (span.start().line..=span.end().line).collect()
                }
                _ => BTreeSet::new(),
            })
            .collect()
    }

    fn markers_of(file: &SourceFile, function: &ItemFn) -> Vec<TestMarker> {
        let body = function.block.span();
        let skipped = Self::literal_lines(function.block.to_token_stream());
        (body.start().line..=body.end().line)
            .filter(|line| !skipped.contains(line))
            .filter_map(|line| {
                let text = file.lines().get(line - 1)?;
                TestMarker::parse(text, line)
            })
            .collect()
    }

    // Arrange, then one or more Act/Assert pairs. An odd length with Arrange
    // first and clean pairs after is the whole grammar.
    fn is_legal(phases: &[MarkerPhase]) -> bool {
        if phases.len() < 3 || phases.len() % 2 == 0 || phases[0] != MarkerPhase::Arrange {
            return false;
        }
        phases[1..]
            .chunks(2)
            .all(|pair| pair == [MarkerPhase::Act, MarkerPhase::Assert])
    }

    fn named(phases: &[MarkerPhase]) -> String {
        if phases.is_empty() {
            return "no AAA markers".to_string();
        }
        phases
            .iter()
            .map(|phase| match phase {
                MarkerPhase::Arrange => "Arrange",
                MarkerPhase::Act => "Act",
                MarkerPhase::Assert => "Assert",
            })
            .collect::<Vec<&str>>()
            .join(", ")
    }

    fn sequence_offence(&self, file: &SourceFile, function: &ItemFn, found: &str) -> Offence {
        let name = function.sig.ident.to_string();
        Offence::new(
            file.relative_path(),
            function.sig.ident.span().start().line,
            self.name(),
            format!(
                "`{name}` reads {found}; a test is `Arrange` followed by one or more \
                 `Act`/`Assert` pairs"
            ),
            "label the sections `// Arrange`, `// Act` and `// Assert`, merging adjacent ones \
             as `// Arrange & Act`, `// Act & Assert` or `// Arrange & Act & Assert`"
                .to_string(),
        )
        .with_subject(&name)
    }

    // Every marker after the first opens a section, and a section that does not
    // start on its own is one a reader has to find rather than see.
    fn spacing_offences(
        &self,
        file: &SourceFile,
        function: &ItemFn,
        markers: &[TestMarker],
    ) -> Vec<Offence> {
        let name = function.sig.ident.to_string();
        markers
            .iter()
            .skip(1)
            .filter(|marker| !Self::is_blank_above(file, marker.line))
            .map(|marker| {
                Offence::new(
                    file.relative_path(),
                    marker.line,
                    self.name(),
                    format!(
                        "`{}` in `{name}` is not preceded by a blank line",
                        marker.label
                    ),
                    format!("put a blank line before `{}`", marker.label),
                )
                .with_subject(&name)
            })
            .collect()
    }

    // Comment lines above a marker are folded into it, the same way
    // `TestFileParser` folds them into the item they document. Without that, a
    // marker that explains itself over two lines reads as a spacing offence --
    // and the explanation is the thing worth keeping.
    fn is_blank_above(file: &SourceFile, line: usize) -> bool {
        let lines = file.lines();
        (1..line)
            .rev()
            .map(|above| lines[above - 1].trim())
            .find(|text| !text.starts_with("//"))
            .is_none_or(str::is_empty)
    }

    fn offences_of(&self, file: &SourceFile, function: &ItemFn) -> Vec<Offence> {
        let markers = Self::markers_of(file, function);
        let phases: Vec<MarkerPhase> = markers
            .iter()
            .flat_map(|marker| marker.phases.clone())
            .collect();
        if !Self::is_legal(&phases) {
            return vec![self.sequence_offence(file, function, &Self::named(&phases))];
        }
        self.spacing_offences(file, function, &markers)
    }
}

impl Default for ArrangeActAssertRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for ArrangeActAssertRule {
    fn name(&self) -> &'static str {
        "arrange-act-assert"
    }

    fn check(&self, file: &SourceFile) -> Vec<Offence> {
        if !Self::applies_to(file) {
            return Vec::new();
        }
        let Ok(syntax) = parse_file(&file.contents()) else {
            return Vec::new();
        };
        syntax
            .items
            .iter()
            .filter_map(|item| match item {
                Item::Fn(function) if Self::is_test(&function.attrs) => Some(function),
                _ => None,
            })
            .flat_map(|function| self.offences_of(file, function))
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
            "A test reads Arrange, then one or more Act/Assert pairs.",
            "#[test]\nfn adds_two_numbers() {\n    assert_eq!(add(1, 1), 2);\n}",
            "#[test]\nfn adds_two_numbers() {\n    // Arrange & Act & Assert\n    assert_eq!(add(1, 1), 2);\n}",
        )
    }
}
