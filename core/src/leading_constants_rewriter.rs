// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::finding::model::layout::constant_layout::ConstantLayout;
use crate::finding::parsing::constant_layout_parser::ConstantLayoutParser;
use crate::source_file::SourceFile;

// A source file's misplaced constants moved to the end of its constants group,
// straight after the imports when there is none.
//
// Safe for the reason the test-file rewriter is: it never reads the text. syn
// says where each constant begins and ends, its attributes and the comment
// directly above it included, and whole line ranges are moved without being
// inspected. Nothing else moves -- every other line keeps its place and its
// bytes -- and the blank lines around the constants group, and where a constant
// left, become one.
//
// A file the parser says cannot be moved as lines is left exactly as it is, and
// the rule's offence, checked again after this, says why.
pub struct LeadingConstantsRewriter;

impl LeadingConstantsRewriter {
    pub const REGISTRIES: [&'static str; 2] = ["lib.rs", "mod.rs"];
    pub const SOURCE_ROOT: &'static str = "src/";

    // None when the file is not this rewriter's to touch, does not parse, has
    // nothing out of place, or is one the fixer declines.
    pub fn rewrite(file: &SourceFile) -> Option<String> {
        if !Self::governs(file) {
            return None;
        }
        let layout = ConstantLayoutParser::parse(file)?;
        if layout.refusal.is_some() {
            return None;
        }
        Some(Self::assemble(file, &layout))
    }

    // The scope leading-constants applies, asked of the file rather than of the
    // rule, as the test-file rewriter asks its own: a rule answers questions
    // about a file, and authorising edits would be a second job.
    fn governs(file: &SourceFile) -> bool {
        file.is_under(Self::SOURCE_ROOT, &Self::REGISTRIES)
    }

    fn assemble(file: &SourceFile, layout: &ConstantLayout) -> String {
        let lines = file.lines();
        let head = Self::without_trailing_blanks(
            lines[..layout.insertion_line - 1]
                .iter()
                .map(String::as_str)
                .collect(),
        );
        let moved: Vec<&str> = layout
            .misplaced
            .iter()
            .flat_map(|constant| &lines[constant.first_line - 1..constant.last_line])
            .map(String::as_str)
            .collect();
        let mut out = String::new();
        if !head.is_empty() {
            out.push_str(&head.join("\n"));
            out.push_str(if layout.follows_constant {
                "\n"
            } else {
                "\n\n"
            });
        }
        out.push_str(&moved.join("\n"));
        out.push_str("\n\n");
        out.push_str(&Self::rest(lines, layout).join("\n"));
        out.push('\n');
        out
    }

    // Everything from the insertion point on, less the moved constants. A blank
    // line left beside one already kept where a constant was is dropped, so the
    // gap a constant leaves is one blank line, not two.
    fn rest<'a>(lines: &'a [String], layout: &ConstantLayout) -> Vec<&'a str> {
        let mut kept: Vec<&str> = Vec::new();
        let mut after_removal = false;
        for (index, line) in lines.iter().enumerate().skip(layout.insertion_line - 1) {
            if layout.moves(index + 1) {
                after_removal = true;
                continue;
            }
            let blank = line.trim().is_empty();
            if blank && after_removal && kept.last().is_none_or(|last| last.trim().is_empty()) {
                continue;
            }
            after_removal &= blank;
            kept.push(line);
        }
        Self::trimmed(kept)
    }

    fn trimmed(lines: Vec<&str>) -> Vec<&str> {
        let start = lines
            .iter()
            .position(|line| !line.trim().is_empty())
            .unwrap_or(lines.len());
        Self::without_trailing_blanks(lines[start..].to_vec())
    }

    fn without_trailing_blanks(lines: Vec<&str>) -> Vec<&str> {
        let end = lines
            .iter()
            .rposition(|line| !line.trim().is_empty())
            .map_or(0, |last| last + 1);
        lines[..end].to_vec()
    }
}
