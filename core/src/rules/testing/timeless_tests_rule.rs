// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use quote::ToTokens;
use syn::Attribute;
use syn::Block;
use syn::Ident;
use syn::ImplItem;
use syn::Item;
use syn::parse_file;

use crate::finding::model::time::time_call::TimeCall;
use crate::finding::model::time::time_call_kind::TimeCallKind;
use crate::finding::parsing::time::imported_names::ImportedNames;
use crate::finding::parsing::time::time_call_finder::TimeCallFinder;
use crate::finding::parsing::unit_test_finder::UnitTestFinder;
use crate::reporting::offence::Offence;
use crate::reporting::rule_explanation::RuleExplanation;
use crate::rule::Rule;
use crate::source_file::SourceFile;

// A test reads no clock and waits on no timer.
//
// A test whose verdict depends on how fast the machine is measures the machine: a stopwatch asserted on, a sleep
// standing in for "until it has happened", a timeout proving that something blocks, a clock as a source of
// uniqueness, randomness nobody seeded. Whether a test is deterministic cannot be read off its syntax -- it may wait
// on a clock inside the code it calls -- but whether the test itself reads a clock or waits on a timer can, and that
// is the part its author controls.
//
// Every function of tests/ and benches/, and in src/ every function under a `#[cfg(test)]` module or carrying a test
// attribute, is judged, helpers as well as tests. Its first clock read, wait, race or unseeded draw is reported at
// its name, resolved through the file's imports. On Tokio's paused clock -- `#[tokio::test(start_paused = true)]` --
// `tokio::time` is virtual and allowed; a `Duration` is a value and never an offence; a seeded generator is
// deterministic. A harness whose subject is real time skips the rule in `stern4rust.toml`.
pub struct TimelessTestsRule;

impl TimelessTestsRule {
    pub const NAME: &'static str = "timeless-tests";
    const MACHINE: &'static str = "a test's verdict must not depend on how fast the machine is";
    const SOURCE_ROOT: &'static str = "src/";
    const TEST_ROOTS: [&'static str; 2] = ["tests/", "benches/"];
    const UNREPEATABLE: &'static str =
        "a test's verdict must not depend on a draw it cannot repeat";

    pub fn new() -> Self {
        Self
    }

    fn correction(kind: TimeCallKind) -> &'static str {
        match kind {
            TimeCallKind::Random => {
                "seed the generator, `StdRng::seed_from_u64`, so every run draws the same"
            }
            _ => {
                "wait for the event itself -- a channel, a barrier, a rendezvous of the parties --, run on Tokio's \
                 paused clock, or inject a clock into the code under test and drive it from the test; name a temp \
                 directory after the test, or let a temp-directory crate choose"
            }
        }
    }

    // The functions of these items that are test code: all of them in a test file, and in src/ those under a
    // `#[cfg(test)]` module or carrying a test attribute, each with whether it runs on Tokio's paused clock.
    fn functions(items: &[Item], in_test: bool) -> Vec<(&Ident, &Block, bool)> {
        items
            .iter()
            .flat_map(|item| match item {
                Item::Fn(function)
                    if in_test || function.attrs.iter().any(UnitTestFinder::is_test) =>
                {
                    vec![(
                        &function.sig.ident,
                        function.block.as_ref(),
                        Self::is_paused(&function.attrs),
                    )]
                }
                Item::Mod(module) => module
                    .content
                    .as_ref()
                    .map(|(_, inner)| {
                        Self::functions(
                            inner,
                            in_test || module.attrs.iter().any(UnitTestFinder::is_cfg_test),
                        )
                    })
                    .unwrap_or_default(),
                Item::Impl(block) if in_test => block
                    .items
                    .iter()
                    .filter_map(|inner| match inner {
                        ImplItem::Fn(method) => Some((
                            &method.sig.ident,
                            &method.block,
                            Self::is_paused(&method.attrs),
                        )),
                        _ => None,
                    })
                    .collect(),
                _ => Vec::new(),
            })
            .collect()
    }

    fn is_paused(attrs: &[Attribute]) -> bool {
        attrs.iter().any(|attr| {
            UnitTestFinder::is_test(attr)
                && attr
                    .to_token_stream()
                    .to_string()
                    .contains("start_paused = true")
        })
    }

    fn offence(path: &str, function: &Ident, call: &TimeCall) -> Offence {
        let name = function.to_string();
        let reason = match call.kind() {
            TimeCallKind::Random => Self::UNREPEATABLE,
            _ => Self::MACHINE,
        };
        Offence::new(
            path,
            function.span().start().line,
            Self::NAME,
            format!(
                "`{name}` {} with `{}`; {reason}",
                call.kind().verb(),
                call.written()
            ),
            String::from(Self::correction(call.kind())),
        )
        .with_subject(&name)
    }
}

impl Default for TimelessTestsRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for TimelessTestsRule {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn check(&self, file: &SourceFile) -> Vec<Offence> {
        let path = file.relative_path();
        let whole = Self::TEST_ROOTS.iter().any(|root| path.starts_with(root));
        if !whole && !path.starts_with(Self::SOURCE_ROOT) {
            return Vec::new();
        }
        let Ok(syntax) = parse_file(&file.contents()) else {
            return Vec::new();
        };
        let names = ImportedNames::of(&syntax.items);
        Self::functions(&syntax.items, whole)
            .into_iter()
            .filter_map(|(function, body, paused)| {
                TimeCallFinder::new(&names, paused)
                    .first_in(body)
                    .map(|call| Self::offence(path, function, &call))
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
        RuleExplanation {
            name: self.name(),
            summary: "A test reads no clock and waits on no timer.",
            breaks: "#[test]\nfn finishes_in_time() {\n    let start = std::time::Instant::now();\n    run();\n    \
             assert!(start.elapsed() < LIMIT);\n}\n",
            instead: "#[test]\nfn finishes() {\n    let (sender, receiver) = std::sync::mpsc::channel();\n    run(sender);\n    \
             assert!(receiver.recv().is_ok());\n}\n",
        }
    }
}
