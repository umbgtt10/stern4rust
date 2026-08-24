// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The imports whose order rustfmt decides rather than the alphabet.
//
// rustfmt sorts `self`, `super` and `crate` ahead of every other path. It also
// treats case as significant -- and which direction it leans depends on the
// style edition, which is the part worth measuring rather than guessing:
//
// |                                       | 2021         | 2024         |
// |---------------------------------------|--------------|--------------|
// | `Bbb::gamma` against `zzz::last`       | sorts last   | sorts first  |
// | `serde_json::Value` against `from_str` | `from_str`   | `Value`      |
//
// The two editions disagree with each other, so no single alphabet can be
// right for both, and this crate cannot know which one the crate under
// inspection compiles with. That is the argument for standing down rather than
// picking a side: declining to judge is correct under either edition, while
// demanding an order would be wrong under one of them.
//
// None of it matches a plain alphabetic sort, and unlike every other
// disagreement this tool could have with a formatter, this one has no
// resolution: `cargo fmt` writes one order, the rule would demand another, and
// stage 1 runs the formatter first. A file caught between the two cannot be
// fixed by hand at all -- each run undoes the last.
//
// One shape the two editions do agree on is an extended path, and it is handled
// below for that reason rather than this one.
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
            || Self::one_extends_the_other(previous, item)
    }

    // Whether the pair is already in order, compared the way rustfmt compares:
    // segment by segment, on the path alone.
    //
    // The line as written is the wrong thing to sort on, and in two ways. It
    // ends in `;` (0x3B), which loses to any digit or letter, so
    // `aaa::select` read as belonging *after* `aaa::select4` -- one path being
    // a prefix of the other at the last segment is enough. And it may open with
    // `pub `, whose `p` beats the `u` of `use`, so every re-export read as
    // belonging above every plain import whatever it named.
    //
    // Both were found in `etheram-raft-embassy`, on `select` beside `select4`,
    // `Either` beside `Either4`, and a `pub use` among ordinary ones.
    // `cargo fmt --check` was clean on all of them, so this rule was demanding
    // an order the formatter would immediately undo -- the deadlock the
    // stand-downs above exist to prevent, reached through the comparison
    // instead.
    pub fn is_ordered(previous: &str, item: &str) -> bool {
        Self::segments(previous) <= Self::segments(item)
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
            .is_some_and(|(left, right)| !Self::share_a_case_shape(left, right))
    }

    // Two segments compare the same way under both comparators only when they
    // are of the same shape, and an initial capital is not enough to say so.
    // `WAL_V2_MAGIC` and `WalRecord` both open with one and the editions still
    // disagree: measured, 2021 puts `WalRecord` first and 2024 `WAL_V2_MAGIC`,
    // the same split already recorded for `Value` against `from_str`. Reading
    // only the first character called that pair same-case and demanded the
    // alphabet, which is a file `cargo fmt` rewrites on every run -- found in
    // `etheram-ibft`, where it cost sixteen offences no edit could clear and
    // both import rules had to be stood down to keep stage 1 green.
    //
    // Shape is the initial and whether the segment is all capitals, because
    // those are the two axes the disagreement runs along. Segments sharing both
    // -- `ALPHA_TWO` against `ZETA_ONE`, `Alpha` against `Zeta`, `alpha`
    // against `zeta` -- were measured to sort identically under 2021, 2024 and
    // a plain sort, so those are still judged.
    fn share_a_case_shape(left: &str, right: &str) -> bool {
        Self::is_uppercase(left) == Self::is_uppercase(right)
            && Self::is_all_capitals(left) == Self::is_all_capitals(right)
    }

    // Capitals and no lowercase. Digits and underscores decide nothing, so `V2`
    // reads as capitals and `v2` does not.
    fn is_all_capitals(segment: &str) -> bool {
        segment.chars().any(char::is_uppercase) && !segment.chars().any(char::is_lowercase)
    }

    // One path continues the other: `alloc::vec` beside `alloc::vec::Vec`.
    //
    // `diverges_by_case` cannot see this, because it looks for the first pair of
    // segments that differ and there is no such pair -- the difference is
    // between a segment and nothing at all. Compared as written the shorter line
    // ends in `;` (59) where the longer carries on with `::` (58), so a plain
    // sort demands the longer path first no matter what follows, while rustfmt
    // demands the shorter. Every extension is a disagreement, so every extension
    // stands down.
    //
    // A rename is not an extension. `bbb as ccc` is one segment rather than
    // `bbb` followed by another, so the pair falls through to the alphabet,
    // which is what rustfmt does with it too.
    fn one_extends_the_other(previous: &str, item: &str) -> bool {
        let (left, right) = (Self::segments(previous), Self::segments(item));
        let shared = left.len().min(right.len());
        left.len() != right.len() && left[..shared] == right[..shared]
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
