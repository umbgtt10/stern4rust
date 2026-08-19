// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::finding::section::Section;
use crate::finding::test_file_item::TestFileItem;
use crate::finding::test_file_parser::TestFileParser;
use crate::source_file::SourceFile;
use std::collections::BTreeMap;

// A test file put back into the order the structure rule asks for.
//
// This exists because the alternative was done by hand four times in one day
// and produced three separate string-handling bugs -- a reordering script that
// tracked raw strings but not ordinary ones, then one that missed `\n` escapes,
// then one desynchronised by a `'"'` char literal. Every one of them was found
// only because the tool re-checked the result afterwards.
//
// The reason a rewriter can be safe where those were not is that it never looks
// at the text. `syn` says where each item begins and ends; whole line ranges are
// moved without being read, so a string literal containing something that looks
// like Rust is carried along like any other line.
//
// Grouping by section fixes the section order too, so a constant sitting below a
// helper moves up with the same pass rather than needing a second one.
pub struct TestFileRewriter;

impl TestFileRewriter {
    pub const ORDER: [Section; 4] = [
        Section::Imports,
        Section::Constants,
        Section::Helpers,
        Section::Tests,
    ];

    pub const REGISTRIES: [&'static str; 2] = ["all_tests.rs", "mod.rs"];
    pub const TESTS_ROOT: &'static str = "tests/";

    // None when the file is not this rewriter's to touch, does not parse, holds
    // no items, or is already correct.
    //
    // The scope check is first and is the important one. It mirrors
    // test-file-structure exactly, because a fixer must never edit a file no
    // rule governs -- the first version of this did, and reordered the imports
    // of thirty `src/` files into one alphabetical block, destroying a grouping
    // convention that no rule checks and so no rule would have restored. It
    // produced a green run and a tree nobody had reviewed.
    //
    // "Already correct" is None rather than an unchanged string so a caller
    // cannot rewrite a file it did not need to touch, which would show up as a
    // spurious diff in somebody's review.
    pub fn rewrite(file: &SourceFile) -> Option<String> {
        if !Self::governs(file) {
            return None;
        }
        let items = TestFileParser::parse(file)?;
        if items.is_empty() {
            return None;
        }
        let rewritten = Self::assemble(file, &items);
        (rewritten != file.contents()).then_some(rewritten)
    }

    // The same scope test-file-structure applies, and deliberately a copy of
    // its shape rather than a call into it: a rule answers questions about a
    // file, and asking it to also authorise edits would give it a second job.
    fn governs(file: &SourceFile) -> bool {
        let path = file.relative_path();
        path.starts_with(Self::TESTS_ROOT)
            && !path
                .rsplit('/')
                .next()
                .is_some_and(|name| Self::REGISTRIES.contains(&name))
    }

    // Chunks are joined rather than appended with separators, so the file ends
    // with exactly one newline instead of whatever the last section happened to
    // leave behind.
    fn assemble(file: &SourceFile, items: &[TestFileItem]) -> String {
        let grouped = Self::grouped(items);
        let chunks: Vec<String> = Self::ORDER
            .iter()
            .filter_map(|section| {
                let members = grouped.get(section)?;
                let gap = "\n".repeat(section.blank_lines_between_entries() + 1);
                Some(
                    members
                        .iter()
                        .map(|item| Self::block(file, item))
                        .collect::<Vec<String>>()
                        .join(&gap),
                )
            })
            .collect();
        let mut out = Self::preamble(file, items);
        out.push_str(&chunks.join("\n\n"));
        out.push('\n');
        let tail = Self::tail(file, items);
        if !tail.is_empty() {
            out.push('\n');
            out.push_str(&tail);
        }
        out
    }

    // Everything above the first item: the header and any file-level commentary,
    // which belong where their author put them and are never reordered.
    fn preamble(file: &SourceFile, items: &[TestFileItem]) -> String {
        let first = items.iter().map(|item| item.first_line).min().unwrap_or(1);
        let kept: Vec<&String> = file.lines().iter().take(first - 1).collect();
        let trimmed: Vec<&&String> = kept
            .iter()
            .rev()
            .skip_while(|line| line.trim().is_empty())
            .collect();
        if trimmed.is_empty() {
            return String::new();
        }
        let mut preamble: Vec<String> = trimmed.iter().rev().map(|line| (**line).clone()).collect();
        preamble.push(String::new());
        preamble.join("\n") + "\n"
    }

    // Anything below the last item. Kept rather than dropped: a trailing comment
    // belongs to nobody and losing it would be the rewriter destroying content
    // it was asked to tidy.
    fn tail(file: &SourceFile, items: &[TestFileItem]) -> String {
        let last = items.iter().map(|item| item.last_line).max().unwrap_or(0);
        let rest: Vec<String> = file
            .lines()
            .iter()
            .skip(last)
            .skip_while(|line| line.trim().is_empty())
            .cloned()
            .collect();
        // A file ending in a newline splits with a trailing empty element, so
        // the blank lines have to come off both ends or the result gains one.
        let Some(end) = rest.iter().rposition(|line| !line.trim().is_empty()) else {
            return String::new();
        };
        rest[..=end].join("\n") + "\n"
    }

    fn block(file: &SourceFile, item: &TestFileItem) -> String {
        file.lines()
            .iter()
            .skip(item.first_line - 1)
            .take(item.last_line + 1 - item.first_line)
            .cloned()
            .collect::<Vec<String>>()
            .join("\n")
    }

    // Imports keep the order they were written in; everything else is sorted.
    //
    // Not an oversight. rustfmt owns import order and disagrees with a plain
    // alphabet on two shapes -- `self`/`super`/`crate`, and a pair diverging at
    // a segment of differing case -- which is why test-file-structure stands
    // down on exactly those pairs. A fixer that sorted imports anyway would
    // write an order `cargo fmt` undoes on the next run, which is the
    // unsatisfiable loop that stand-down exists to prevent. The first version
    // of this did precisely that to `use serde_json::Value` sitting beside
    // `use serde_json::from_str`.
    //
    // They are still *moved*, as a block, into the imports section. Grouping
    // them is safe; ordering them is rustfmt's business.
    fn grouped(items: &[TestFileItem]) -> BTreeMap<Section, Vec<&TestFileItem>> {
        let mut grouped: BTreeMap<Section, Vec<&TestFileItem>> = BTreeMap::new();
        for item in items {
            grouped.entry(item.section).or_default().push(item);
        }
        for (section, members) in grouped.iter_mut() {
            if *section != Section::Imports {
                members.sort_by_key(|item| item.sort_key());
            }
        }
        grouped
    }
}
