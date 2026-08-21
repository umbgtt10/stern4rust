// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::finding::model::qualified_call::QualifiedCall;
use crate::finding::parsing::qualified_call_finder::QualifiedCallFinder;
use crate::reporting::offence::Offence;
use crate::reporting::rule_explanation::RuleExplanation;
use crate::rule::Rule;
use crate::source_file::SourceFile;

// A function is called through a name this file imported, not through a path.
//
// `use` statements are a file's list of dependencies. A path written inline at
// the call site is a dependency that never appears on that list, so the list
// stops being an answer to what this file needs -- and the reader who scans the
// top of the file to find out is quietly given a wrong answer.
//
// One imported segment is allowed and is the point rather than an exception.
// `use std::fs` followed by `fs::read_to_string` names the route once at the top
// and still says at the call site which module the function came from. What the
// rule removes is the route being respelt at every call: `std::env::args`, or a
// `syn::parse_file` whose crate no line in the file mentions.
//
// Both productive and test files are checked. A test file has the same reader.
pub struct ImportedPathsRule;

impl ImportedPathsRule {
    pub fn new() -> Self {
        Self
    }

    fn offence(&self, file: &SourceFile, call: &QualifiedCall) -> Offence {
        Offence::new(
            file.relative_path(),
            call.line,
            self.name(),
            format!(
                "`{}` is reached through a path; no import of this file names it",
                call.path
            ),
            format!("add `use {};` and call `{}`", call.import(), call.call()),
        )
        .with_subject(&call.path)
    }
}

impl Default for ImportedPathsRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for ImportedPathsRule {
    fn name(&self) -> &'static str {
        "imported-paths"
    }

    fn check(&self, file: &SourceFile) -> Vec<Offence> {
        QualifiedCallFinder::find(file)
            .unwrap_or_default()
            .iter()
            .map(|call| self.offence(file, call))
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
            "A function is called through a name this file imported, not through a path.",
            "let parsed = syn::parse_file(text);",
            "use syn::parse_file;\n\nlet parsed = parse_file(text);",
        )
    }
}
