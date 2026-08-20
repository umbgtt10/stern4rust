// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The imports whose order rustfmt decides rather than the alphabet.
//
// rustfmt sorts `self`, `super` and `crate` ahead of every other path. It also
// treats case as significant, and -- measured, because it is not what anyone
// would guess -- in opposite directions at the two levels: an uppercase-initial
// crate goes behind every lowercase one, while an uppercase-initial segment
// later in a path goes ahead of its lowercase siblings. `Bbb::gamma` sorts after
// `zzz::last`; `serde_json::Value` sorts before `serde_json::from_str`.
//
// None of that matches a plain alphabetic sort, and unlike every other
// disagreement this tool could have with a formatter, this one has no
// resolution: `cargo fmt` writes one order, the rule would demand another, and
// stage 1 runs the formatter first. A file caught between the two cannot be
// fixed by hand at all -- each run undoes the last.
//
// So the structure rule stands down on exactly those pairs. Everything else in
// the import list is still ordered, because among segments of the same case
// rustfmt's comparator and the alphabet agree.
//
// The case rule is pairwise rather than a property of one import, and that
// distinction is the whole of it. `use serde_json::Value` and
// `use serde_json::from_str` share a first segment and part company at the
// second, where one is uppercase and one is not. Neither path is remarkable on
// its own; only the pair is.
pub struct ImportPath;

impl ImportPath {
    // Whether rustfmt, rather than the alphabet, decides this pair's order.
    pub fn decides_order(previous: &str, item: &str) -> bool {
        Self::is_specially_ordered(previous)
            || Self::is_specially_ordered(item)
            || Self::diverges_by_case(previous, item)
    }

    pub fn is_specially_ordered(import: &str) -> bool {
        let first = Self::first_segment(import);
        matches!(first, "self" | "super" | "crate")
            || first.chars().next().is_some_and(char::is_uppercase)
    }

    // Where two paths first differ, and whether the segments there are of
    // different case. That is the one place the two comparators can disagree:
    // before it the paths are identical, and after it nothing is compared.
    fn diverges_by_case(previous: &str, item: &str) -> bool {
        Self::segments(previous)
            .into_iter()
            .zip(Self::segments(item))
            .find(|(left, right)| left != right)
            .is_some_and(|(left, right)| Self::is_uppercase(left) != Self::is_uppercase(right))
    }

    // The first segment, not a prefix: a crate genuinely named `crateful` sorts
    // alphabetically like anything else.
    fn first_segment(import: &str) -> &str {
        Self::segments(import).first().copied().unwrap_or_default()
    }

    fn is_uppercase(segment: &str) -> bool {
        segment.chars().next().is_some_and(char::is_uppercase)
    }

    fn segments(import: &str) -> Vec<&str> {
        import
            .trim()
            .trim_start_matches("pub ")
            .trim_start_matches("use ")
            .trim_start()
            .trim_end_matches(';')
            .trim_start_matches("::")
            .split("::")
            .map(str::trim)
            .collect()
    }
}
