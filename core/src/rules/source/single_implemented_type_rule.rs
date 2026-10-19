// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::finding::model::file_stem::FileStem;
use crate::finding::model::implemented_type::ImplementedType;
use crate::finding::parsing::implemented_type_finder::ImplementedTypeFinder;
use crate::reporting::offence::Offence;
use crate::reporting::rule_explanation::RuleExplanation;
use crate::rule::Rule;
use crate::source_file::SourceFile;

// A source file has one subject: at most one type that carries behaviour.
//
// Plain data declarations are not subjects, so a file may hold as many structs
// and enums without impl blocks as its subject needs. What it may not hold is a
// second type with an impl block, because then the file has two things to be
// named after and its name can only describe one of them. A reader who opens
// `report_printer.rs` looking for how a report is printed should not have to
// step over a second type on the way.
//
// It is also what keeps a mirrored test file meaningful: `src/foo.rs` answering
// to `tests/foo_tests.rs` only says something when `foo.rs` has one subject to
// test.
//
// tests/ is exempt, and not as a concession. A test file legitimately holds
// several fakes that each carry an impl block, and that is the shape
// test-file-structure asks for rather than something to be split up.
pub struct SingleImplementedTypeRule;

impl SingleImplementedTypeRule {
    pub const TESTS_ROOT: &'static str = "tests/";

    pub fn new() -> Self {
        Self
    }

    fn applies_to(file: &SourceFile) -> bool {
        !file.relative_path().starts_with(Self::TESTS_ROOT)
    }

    // The subject is the type the file is named after, wherever it stands, and
    // every other one is reported, so the offence names the type to move rather
    // than saying the file has too many. Taking the first instead blamed the
    // subject whenever a companion was declared above it. With no type named for
    // the file, the first stands in. "Named after" is named-subject's reading --
    // the stem is the type's name or its last words -- so the two rules agree on
    // which type a file is about.
    fn subject<'a>(file: &SourceFile, found: &'a [ImplementedType]) -> Option<&'a ImplementedType> {
        let stem = FileStem::of(file.relative_path());
        found
            .iter()
            .find(|candidate| stem.names(&candidate.name))
            .or_else(|| found.first())
    }

    fn offence(&self, file: &SourceFile, subject: &str, extra: &ImplementedType) -> Offence {
        Offence::new(
            file.relative_path(),
            extra.line,
            self.name(),
            format!(
                "`{}` is a second type with an impl block; this file's subject is \
                 already `{subject}`",
                extra.name
            ),
            format!(
                "move `{}` and its impl blocks into {}",
                extra.name,
                extra.suggested_file()
            ),
        )
        .with_subject(&extra.name)
    }
}

impl Default for SingleImplementedTypeRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for SingleImplementedTypeRule {
    fn name(&self) -> &'static str {
        "single-implemented-type"
    }

    fn check(&self, file: &SourceFile) -> Vec<Offence> {
        if !Self::applies_to(file) {
            return Vec::new();
        }
        let found = ImplementedTypeFinder::find(file).unwrap_or_default();
        let Some(subject) = Self::subject(file, &found) else {
            return Vec::new();
        };
        found
            .iter()
            .filter(|extra| *extra != subject)
            .map(|extra| self.offence(file, &subject.name, extra))
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
            "A source file has one subject: at most one type that carries behaviour.",
            "pub struct Widget;\nimpl Widget { }\n\npub struct Gadget;\nimpl Gadget { }",
            "// widget.rs\npub struct Widget;\nimpl Widget { }\n\n// gadget.rs\npub struct Gadget;\nimpl Gadget { }",
        )
    }
}
