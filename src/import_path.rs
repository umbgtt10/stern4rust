// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The imports whose order rustfmt decides rather than the alphabet.
//
// rustfmt sorts `self`, `super` and `crate` ahead of every other path, and an
// uppercase-initial path behind them all. Neither matches a plain alphabetic
// sort, and unlike every other disagreement this tool could have with a
// formatter, this one has no resolution: `cargo fmt` writes one order, the rule
// would demand another, and stage 1 runs the formatter first. A file caught
// between the two cannot be fixed by hand at all.
//
// So the structure rule stands down on exactly those pairs. Everything else in
// the import list is still ordered, because among ordinary paths rustfmt's
// comparator and the alphabet agree -- verified against the 168 distinct import
// lines in the sibling tools' test suites, which rustfmt leaves exactly as an
// alphabetic sort produces them.
//
// The shape this exists for is a shared helper inside the tests tree. Everything
// under tests/ is one crate rooted at all_tests.rs, so a sibling reaches a
// helper through `use crate::support::...`, and that is the trigger.
pub struct ImportPath;

impl ImportPath {
    pub fn is_specially_ordered(import: &str) -> bool {
        let first = Self::first_segment(import);
        matches!(first, "self" | "super" | "crate")
            || first.chars().next().is_some_and(char::is_uppercase)
    }

    // The first segment, not a prefix: a crate genuinely named `crateful` sorts
    // alphabetically like anything else.
    fn first_segment(import: &str) -> &str {
        import
            .trim()
            .trim_start_matches("pub ")
            .trim_start_matches("use ")
            .trim_start()
            .trim_start_matches("::")
            .split(|character: char| !character.is_alphanumeric() && character != '_')
            .next()
            .unwrap_or_default()
    }
}
