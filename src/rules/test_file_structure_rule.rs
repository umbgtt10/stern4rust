// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::finding::import_path::ImportPath;
use crate::finding::section::Section;
use crate::finding::test_file_item::TestFileItem;
use crate::finding::test_file_parser::TestFileParser;
use crate::reporting::offence::Offence;
use crate::rule::Rule;
use crate::source_file::SourceFile;

// A test file reads top to bottom in one order: header, imports, constants,
// helpers, tests. Each group is alphabetical, and the spacing is part of the
// shape -- imports run together, everything else is separated by exactly one
// blank line.
//
// The order is what makes a test file skimmable without reading it. Once a
// constant sits below a helper, or a test lands between two others out of order,
// the file has no shape and every later addition goes wherever the last one
// happened to end.
pub struct TestFileStructureRule;

impl TestFileStructureRule {
    pub fn new() -> Self {
        Self
    }

    // Source files have a different shape and are not this rule's business.
    fn applies_to(file: &SourceFile) -> bool {
        file.relative_path().starts_with("tests/") && !Self::is_registry(file)
    }

    // all_tests.rs and mod.rs are registries, not test files. They hold nothing
    // but `pub mod` lines, and a registry reads as a list -- demanding a blank
    // line between each entry would make the one file whose whole job is to be
    // scannable the hardest one to scan.
    fn is_registry(file: &SourceFile) -> bool {
        matches!(
            file.relative_path().rsplit('/').next(),
            Some("all_tests.rs") | Some("mod.rs")
        )
    }

    fn section_order(&self, previous: &TestFileItem, item: &TestFileItem) -> Option<Offence> {
        if item.section >= previous.section {
            return None;
        }
        Some(self.offence(
            item,
            format!(
                "a {} follows a {}, but every {} belongs above them",
                item.section.label(),
                previous.section.label(),
                item.section.label()
            ),
            format!(
                "move `{}` up above the {}s",
                item.name,
                previous.section.label()
            ),
        ))
    }

    fn alphabetic_order(&self, previous: &TestFileItem, item: &TestFileItem) -> Option<Offence> {
        if previous.section != item.section || item.sort_key() >= previous.sort_key() {
            return None;
        }
        if Self::ordered_by_rustfmt(previous, item) {
            return None;
        }
        Some(self.offence(
            item,
            format!(
                "{} is out of alphabetic order; it follows {}",
                item.name, previous.name
            ),
            format!("move `{}` above `{}`", item.name, previous.name),
        ))
    }

    // A pair whose order rustfmt decides is not this rule's to judge. Demanding
    // the alphabet there would make the file unsatisfiable rather than merely
    // wrong: the formatter runs first and writes the other order back.
    fn ordered_by_rustfmt(previous: &TestFileItem, item: &TestFileItem) -> bool {
        item.section == Section::Imports && ImportPath::decides_order(&previous.name, &item.name)
    }

    fn spacing(
        &self,
        file: &SourceFile,
        previous: &TestFileItem,
        item: &TestFileItem,
    ) -> Option<Offence> {
        if previous.section != item.section {
            return None;
        }
        let expected = item.section.blank_lines_between_entries();
        let found = Self::blank_lines_between(file, previous.last_line, item.first_line);
        if found == expected {
            return None;
        }
        Some(self.offence(
            item,
            format!(
                "expected {expected} blank line(s) before {} but found {found}",
                item.name
            ),
            format!(
                "leave exactly {expected} blank line(s) between `{}` and `{}`",
                previous.name, item.name
            ),
        ))
    }

    fn blank_lines_between(file: &SourceFile, after: usize, before: usize) -> usize {
        file.lines()
            .iter()
            .skip(after)
            .take(before.saturating_sub(after).saturating_sub(1))
            .filter(|line| line.trim().is_empty())
            .count()
    }

    fn offence(&self, item: &TestFileItem, description: String, correction: String) -> Offence {
        Offence::new("", item.first_line, self.name(), description, correction)
            .with_subject(&item.name)
    }
}

impl Default for TestFileStructureRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for TestFileStructureRule {
    fn name(&self) -> &'static str {
        "test-file-structure"
    }

    // Every offence in the file, not only the first. Unlike a missing header --
    // where one report per line would bury the rest of the workspace -- these
    // are independent facts about different items, and a reader fixing the file
    // wants all of them.
    fn check(&self, file: &SourceFile) -> Vec<Offence> {
        if !Self::applies_to(file) {
            return Vec::new();
        }
        let Some(items) = TestFileParser::parse(file) else {
            return Vec::new();
        };
        let mut offences = Vec::new();
        for pair in items.windows(2) {
            let (previous, item) = (&pair[0], &pair[1]);
            offences.extend(self.section_order(previous, item));
            offences.extend(self.alphabetic_order(previous, item));
            offences.extend(self.spacing(file, previous, item));
        }
        offences
            .into_iter()
            .map(|offence| Offence {
                file: file.relative_path().to_string(),
                ..offence
            })
            .collect()
    }

    fn check_workspace(&self, _files: &[SourceFile]) -> Vec<Offence> {
        Vec::new()
    }

    fn is_configured(&self) -> bool {
        true
    }
}
