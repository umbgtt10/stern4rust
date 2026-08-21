// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::reporting::offence::Offence;
use crate::reporting::rule_explanation::RuleExplanation;
use crate::source_file::SourceFile;

// One rule, one file, one implementation. A rule sees a single source file and
// answers with what is wrong with it -- it does not walk, does not print, and
// does not know which other rules exist.
//
// That is what keeps the set open: adding a rule is a new file implementing this
// trait plus one line in the registry, and the rule is testable on a string of
// source without a workspace behind it.
//
// Nothing declared here has a body, and that is the point. Every rule answers
// all six questions in its own file, including the answers that are "nothing".
//
// Defaults used to spare a rule from saying so, and what they cost was the
// ability to read a rule and know what it does. A file with no `check` meant
// either "this rule's subject is the tree" or "this rule has not been finished",
// and the two were the same absence. `is_configured` was worse: it made every
// new rule configured without anybody choosing it, so a rule that could not
// possibly run would still join the set and report nothing wrong -- the silent
// pass this tool exists to catch, in the tool itself.
//
// The other half of "a trait declares" -- that every implementor implements
// every method -- costs no code here. With no body to fall back on, `rustc`
// rejects an incomplete impl outright.
pub trait Rule {
    // Appears verbatim in the report's rule column, so it is kebab-case and
    // reads as the thing being required rather than the thing being forbidden.
    fn name(&self) -> &'static str;

    // What is wrong with this one file, judged on its own. A rule whose subject
    // is the tree answers with no offences, deliberately and in its own file.
    fn check(&self, file: &SourceFile) -> Vec<Offence>;

    // What is wrong with the set of files taken together.
    //
    // Some rules are not about a file at all. "There is exactly one all_tests.rs"
    // and "every subfolder has a mod.rs" are facts about a tree, and the file
    // that would carry the offence is precisely the one that does not exist --
    // so there is nothing for check() to be handed. The registry asks both
    // questions of every rule without caring which one a rule answers.
    fn check_workspace(&self, files: &[SourceFile]) -> Vec<Offence>;

    // Whether this rule has what it needs to say anything.
    //
    // Most rules always do. The header rule does not: it has no idea what your
    // header says until `--header-file` tells it, and registering it anyway
    // would let a run report "all rules satisfied" for a rule that never looked
    // at a single file.
    //
    // A rule answers this for itself so the registry does not have to name any
    // rule in particular. The alternative -- an `if` in the registry that knows
    // about the header rule -- is how the registry ends up with a second,
    // hand-maintained idea of which rules exist.
    fn is_configured(&self) -> bool;

    // What this rule needs before it can run, for the report to print when it
    // could not. `None` where a rule needs nothing and is therefore never
    // unconfigured.
    //
    // The rule answers for itself so the printer names no rule in particular.
    // The alternative -- a hardcoded reason in the printer -- was there until
    // a second rule could go unconfigured, at which point it started telling
    // readers to pass `--header-file` for a rule waiting on a manifest field.
    // A line saying only that something is unset says what is wrong without
    // saying what to do, which is the one thing every finding here must do.
    fn requirement(&self) -> Option<&'static str>;

    // What this rule wants, for `--rules` to print when nobody has run it yet.
    // A sentence, a scrap of source that breaks the rule, and the same scrap
    // put right.
    //
    // Here rather than in the printer for the reason `requirement` is here: a
    // rule that changed its mind would otherwise leave a printer describing
    // what it used to want, and nothing would say so.
    fn explanation(&self) -> RuleExplanation;
}
