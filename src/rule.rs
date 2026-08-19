// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::reporting::offence::Offence;
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

    // What is wrong with this one file, judged on its own.
    fn check(&self, _file: &SourceFile) -> Vec<Offence> {
        Vec::new()
    }

    // What is wrong with the set of files taken together.
    //
    // Some rules are not about a file at all. "There is exactly one all_tests.rs"
    // and "every subfolder has a mod.rs" are facts about a tree, and the file
    // that would carry the offence is precisely the one that does not exist --
    // so there is nothing for check() to be handed.
    //
    // Both methods default to reporting nothing, so a rule implements whichever
    // question it actually answers and the registry calls both without caring
    // which.
    fn check_workspace(&self, _files: &[SourceFile]) -> Vec<Offence> {
        Vec::new()
    }

    // Whether this rule has what it needs to say anything.
    //
    // Most rules always do, so the default is true. The header rule does not:
    // it has no idea what your header says until `--header-file` tells it, and
    // registering it anyway would let a run report "all rules satisfied" for a
    // rule that never looked at a single file.
    //
    // A rule answers this for itself so the registry does not have to name any
    // rule in particular. The alternative -- an `if` in the registry that knows
    // about the header rule -- is how the registry ends up with a second,
    // hand-maintained idea of which rules exist.
    fn is_configured(&self) -> bool {
        true
    }
}
