// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// What one rule wants, said in the rule's own words: a sentence, a scrap of
// source that breaks it, and the same scrap put right.
//
// The rule answers for itself, the way it already answers `requirement`. The
// alternative -- a table in the printer mapping name to prose -- is a second
// idea of which rules exist, kept in step by hand, and it would fall out of
// date the first time a rule changed its mind without the table being told.
//
// `breaks` and `instead` are source rather than description because the
// question a reader arrives with is "what does this look like", and a sentence
// answering it is longer and less exact than two lines of Rust.
pub struct RuleExplanation {
    pub name: &'static str,
    pub summary: &'static str,
    pub breaks: &'static str,
    pub instead: &'static str,
}

impl RuleExplanation {
    pub fn new(
        name: &'static str,
        summary: &'static str,
        breaks: &'static str,
        instead: &'static str,
    ) -> Self {
        Self {
            name,
            summary,
            breaks,
            instead,
        }
    }
}
