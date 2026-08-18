// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::offence::Offence;
use crate::source_file::SourceFile;

// One rule, one file, one implementation. A rule sees a single source file and
// answers with what is wrong with it -- it does not walk, does not print, and
// does not know which other rules exist.
//
// That is what keeps the set open: adding a rule is a new file implementing this
// trait plus one line in the registry, and the rule is testable on a string of
// source without a workspace behind it.
pub trait Rule {
    // Appears verbatim in the report's rule column, so it is kebab-case and
    // reads as the thing being required rather than the thing being forbidden.
    fn name(&self) -> &'static str;

    fn check(&self, file: &SourceFile) -> Vec<Offence>;
}
