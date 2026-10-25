// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::finding::model::layout::impl_layout::ImplLayout;
use crate::finding::parsing::impl_layout_parser::ImplLayoutParser;
use crate::reporting::offence::Offence;
use crate::reporting::rule_explanation::RuleExplanation;
use crate::rule::Rule;
use crate::source_file::SourceFile;

// An inherent impl block lists its functions in a fixed order: constructors,
// fewest parameters first, then its `pub` functions, then the rest, each
// alphabetical, with its constants and types ahead of them all.
//
// Where a function sat was decided by whoever wrote it, and in a repository
// written mostly by agents the author changes every session, so narrative order
// drifts and every placement is a judgement. A fixed order is the same in every
// session: it takes the decision out of every change and keeps a new function's
// diff to the lines it adds. Order within an impl means nothing to the
// compiler, which is what lets --fix repair it with no semantic risk.
//
// One offence per block, at the impl, naming the first member out of place and
// the one it belongs before. Trait impls keep the trait's order and are left
// alone; tests/ is test-file-structure's.
pub struct OrderedImplFunctionsRule;

impl OrderedImplFunctionsRule {
    pub const NAME: &'static str = "ordered-impl-functions";
    pub const SOURCE_ROOT: &'static str = "src/";

    pub fn new() -> Self {
        Self
    }

    fn correction(layout: &ImplLayout) -> String {
        let order = "constructors first, fewest parameters first, then its `pub` functions, then \
                     the rest, each alphabetical, its constants and types ahead";
        match layout.refusal() {
            None => format!("list {order} -- or run `--fix`"),
            Some(refusal) => format!(
                "list {order}, by hand: `--fix` leaves alone {}",
                refusal.reason()
            ),
        }
    }

    fn offence(file: &SourceFile, layout: &ImplLayout) -> Option<Offence> {
        let at = layout.first_misplaced()?;
        Some(
            Offence::new(
                file.relative_path(),
                layout.line(),
                Self::NAME,
                format!(
                    "`impl {}` is out of order: `{}` belongs before `{}`",
                    layout.owner(),
                    layout.ordered()[at].name(),
                    layout.members()[at].name()
                ),
                Self::correction(layout),
            )
            .with_subject(layout.owner()),
        )
    }
}

impl Default for OrderedImplFunctionsRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for OrderedImplFunctionsRule {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn check(&self, file: &SourceFile) -> Vec<Offence> {
        if !file.relative_path().starts_with(Self::SOURCE_ROOT) {
            return Vec::new();
        }
        ImplLayoutParser::parse(file)
            .iter()
            .filter_map(|layout| Self::offence(file, layout))
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
            summary: "An inherent impl lists constructors, then `pub` functions, then the rest, each alphabetical.",
            breaks: "impl Store {\n    pub fn len(&self) -> usize {\n        0\n    }\n\n    pub fn new() -> Self {\n        \
             Self\n    }\n}",
            instead: "impl Store {\n    pub fn new() -> Self {\n        Self\n    }\n\n    pub fn len(&self) -> usize {\n        \
             0\n    }\n}",
        }
    }
}
