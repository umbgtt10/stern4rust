// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::finding::model::constant_layout::ConstantLayout;
use crate::finding::model::misplaced_constant::MisplacedConstant;
use crate::finding::parsing::constant_layout_parser::ConstantLayoutParser;
use crate::reporting::offence::Offence;
use crate::reporting::rule_explanation::RuleExplanation;
use crate::rule::Rule;
use crate::source_file::SourceFile;

// A source file reads header, imports, constants, then everything else.
//
// A test file already reads in one order, and a source file had none for its
// constants: the common shape was a type, then a constant, then the impl that
// uses it, the constant parked beside its first reader -- and a reader looking
// for a file's numbers had to read the whole file to find them. Module-level
// items are order-independent in Rust, so where a constant sits is pure layout,
// which is what makes it a placement a tool can enforce and repair.
//
// A module-level `const` or `static` declared after any item that is neither an
// import nor a constant is an offence, once per file, at the first one. Imports
// and `mod` declarations are not judged here; an inline module is one item, and
// what it holds is not looked at. The correction offers --fix, or says why the
// fixer declines the file.
//
// Registries (`lib.rs`, `mod.rs`) are module-registry's, and tests/ is
// test-file-structure's.
pub struct LeadingConstantsRule;

impl LeadingConstantsRule {
    pub const NAME: &'static str = "leading-constants";
    pub const REGISTRIES: [&'static str; 2] = ["lib.rs", "mod.rs"];
    pub const SOURCE_ROOT: &'static str = "src/";

    pub fn new() -> Self {
        Self
    }

    fn correction(layout: &ConstantLayout) -> String {
        match layout.refusal {
            None => format!(
                "move it, and every constant below `{}`, straight after the imports, or run \
                 `--fix`",
                layout.leader
            ),
            Some(refusal) => format!(
                "move it, and every constant below `{}`, straight after the imports by hand: \
                 `--fix` leaves alone {}",
                layout.leader,
                refusal.reason()
            ),
        }
    }

    fn governs(file: &SourceFile) -> bool {
        file.is_under(Self::SOURCE_ROOT, &Self::REGISTRIES)
    }

    fn offence(file: &SourceFile, layout: &ConstantLayout, first: &MisplacedConstant) -> Offence {
        Offence::new(
            file.relative_path(),
            first.line,
            Self::NAME,
            format!(
                "`{}` is declared after `{}`; constants come straight after the imports",
                first.name, layout.leader
            ),
            Self::correction(layout),
        )
        .with_subject(&first.name)
    }
}

impl Default for LeadingConstantsRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for LeadingConstantsRule {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn check(&self, file: &SourceFile) -> Vec<Offence> {
        if !Self::governs(file) {
            return Vec::new();
        }
        ConstantLayoutParser::parse(file)
            .and_then(|layout| {
                layout
                    .misplaced
                    .first()
                    .map(|first| Self::offence(file, &layout, first))
            })
            .into_iter()
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
            "A source file reads header, imports, constants, then everything else.",
            "use std::fmt;\n\npub struct Widget;\n\nconst LIMIT: u8 = 3;",
            "use std::fmt;\n\nconst LIMIT: u8 = 3;\n\npub struct Widget;",
        )
    }
}
