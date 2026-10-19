// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::path::Path;

use syn::Item;
use syn::parse_file;

use crate::finding::model::declared_item::DeclaredItem;
use crate::finding::model::file_stem::FileStem;
use crate::finding::model::type_name::TypeName;
use crate::finding::parsing::declared_item_finder::DeclaredItemFinder;
use crate::finding::parsing::implemented_type_finder::ImplementedTypeFinder;
use crate::reporting::offence::Offence;
use crate::reporting::rule_explanation::RuleExplanation;
use crate::rule::Rule;
use crate::source_file::SourceFile;

// A source file is named after its subject.
//
// single-implemented-type bounds a file to one subject and never asks whether
// the file's name says what it is. A reader opening `listener.rs` expects to find
// a listener; finding `Backoff` there, or `LogRecord` beside a `LogSeverity` that
// does all the work, is the drift this rule exists to catch.
//
// The subject is a type the file implements, as single-implemented-type counts
// them. A file with none is named after one of the traits and types it declares.
// A trait beside an implemented type is that type's companion and not a
// candidate. Named after is `FileStem::names`: the type's name, or its last
// words, since the words in front are usually the crate's.
//
// One offence per file, against the first candidate, with the file it would be
// named for. Traits come first among the declarations, because a trait declares
// behaviour and a data type does not. A file with two subjects, one of them named
// for it, is single-implemented-type's and silent here.
//
// Exempt: entry points and module roots, named for their role in the package,
// and tests/, benches/, examples/ and src/bin/, named for what they exercise or
// run. A file of functions and constants alone has nothing to be named after.
pub struct NamedSubjectRule;

impl NamedSubjectRule {
    const EXEMPT_ROOTS: [&'static str; 4] = ["tests/", "benches/", "examples/", "src/bin/"];
    const EXEMPT_FILES: [&'static str; 4] = ["lib.rs", "main.rs", "mod.rs", "build.rs"];
    const IMPLEMENTED: &'static str = "the types it implements";
    const DECLARED: &'static str = "the traits and types it declares";

    pub fn new() -> Self {
        Self
    }

    fn applies_to(file: &SourceFile) -> bool {
        let path = file.relative_path();
        !Self::EXEMPT_ROOTS.iter().any(|root| path.starts_with(root))
            && !Self::EXEMPT_FILES.contains(&Self::file_name(path))
    }

    fn file_name(path: &str) -> &str {
        Path::new(path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
    }

    // The candidates in the order the offence names them, and what they are.
    fn candidates(items: &[Item]) -> (Vec<DeclaredItem>, &'static str) {
        let implemented: Vec<DeclaredItem> = ImplementedTypeFinder::among(items)
            .into_iter()
            .map(|found| DeclaredItem::new(&found.name, found.line))
            .collect();
        if !implemented.is_empty() {
            return (implemented, Self::IMPLEMENTED);
        }
        let declared = DeclaredItemFinder::traits(items)
            .into_iter()
            .chain(DeclaredItemFinder::types(items))
            .collect();
        (declared, Self::DECLARED)
    }

    fn offence(
        &self,
        file: &SourceFile,
        candidates: &[DeclaredItem],
        kind: &str,
    ) -> Option<Offence> {
        let subject = candidates.first()?;
        let listed = candidates
            .iter()
            .map(|candidate| format!("`{}`", candidate.name))
            .collect::<Vec<String>>()
            .join(", ");
        let offence = Offence::new(
            file.relative_path(),
            subject.line,
            self.name(),
            format!(
                "`{}` is named after none of {kind}: {listed}",
                Self::file_name(file.relative_path())
            ),
            format!(
                "rename the file to {}, or rename `{}` after the file",
                TypeName::new(&subject.name).file_name(),
                subject.name
            ),
        );
        Some(offence.with_subject(&subject.name))
    }
}

impl Default for NamedSubjectRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for NamedSubjectRule {
    fn name(&self) -> &'static str {
        "named-subject"
    }

    fn check(&self, file: &SourceFile) -> Vec<Offence> {
        if !Self::applies_to(file) {
            return Vec::new();
        }
        let Ok(syntax) = parse_file(&file.contents()) else {
            return Vec::new();
        };
        let (candidates, kind) = Self::candidates(&syntax.items);
        let stem = FileStem::of(file.relative_path());
        if candidates
            .iter()
            .any(|candidate| stem.names(&candidate.name))
        {
            return Vec::new();
        }
        self.offence(file, &candidates, kind).into_iter().collect()
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
            "A source file is named after its subject: a type it implements, or with none, a trait or type it declares.",
            "// listener.rs\npub struct Backoff;\nimpl Backoff { }",
            "// backoff.rs\npub struct Backoff;\nimpl Backoff { }",
        )
    }
}
