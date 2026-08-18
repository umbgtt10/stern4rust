// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::collections::BTreeSet;

use crate::offence::Offence;

// One table for every rule. Columns are sized to their contents so the report
// stays readable when a rule name or a path grows, and the summary line is
// greppable so a wrapper script can report a count without parsing the table.
pub struct ReportPrinter {
    files_scanned: usize,
}

impl ReportPrinter {
    pub fn new(files_scanned: usize) -> Self {
        Self { files_scanned }
    }

    pub fn print(&self, offences: &[Offence]) {
        println!("{}", self.render(offences));
    }

    // Returns the report rather than writing it, so the shape is assertable in a
    // test instead of being checked for not panicking.
    pub fn render(&self, offences: &[Offence]) -> String {
        let mut report = String::from("stern4rust report\n\n");
        if offences.is_empty() {
            report.push_str("All rules are satisfied.\n\n");
            report.push_str(&self.summary(offences));
            return report;
        }

        let widths = ColumnWidths::of(offences);
        report.push_str(&Self::heading(&widths));
        for offence in offences {
            report.push_str(&Self::row(offence, &widths));
            report.push_str(&Self::correction_row(offence, &widths));
        }
        report.push('\n');
        report.push_str(&self.summary(offences));
        report
    }

    fn heading(widths: &ColumnWidths) -> String {
        format!(
            "{:<file$}  {:>line$}  {:<rule$}  offence\n{}  {}  {}  {}\n",
            "file",
            "line",
            "rule",
            "-".repeat(widths.file),
            "-".repeat(widths.line),
            "-".repeat(widths.rule),
            "-".repeat(widths.description),
            file = widths.file,
            line = widths.line,
            rule = widths.rule
        )
    }

    fn row(offence: &Offence, widths: &ColumnWidths) -> String {
        format!(
            "{:<file$}  {:>line$}  {:<rule$}  {}\n",
            offence.file,
            offence.line,
            offence.rule,
            offence.description,
            file = widths.file,
            line = widths.line,
            rule = widths.rule
        )
    }

    // On its own line beneath the offence rather than in a fifth column. The
    // description column is already the widest thing in the report, and a
    // correction is a sentence rather than a field -- side by side, neither
    // would be readable.
    fn correction_row(offence: &Offence, widths: &ColumnWidths) -> String {
        let indent = widths.file + widths.line + widths.rule + 6;
        format!("{}fix: {}\n", " ".repeat(indent), offence.correction)
    }

    fn summary(&self, offences: &[Offence]) -> String {
        let broken: BTreeSet<&str> = offences.iter().map(|offence| offence.rule).collect();
        format!(
            "summary: files_scanned={} offences={} rules_broken={}",
            self.files_scanned,
            offences.len(),
            broken.len()
        )
    }
}

struct ColumnWidths {
    file: usize,
    line: usize,
    rule: usize,
    description: usize,
}

impl ColumnWidths {
    fn of(offences: &[Offence]) -> Self {
        Self {
            file: Self::widest(offences.iter().map(|offence| offence.file.len()), "file"),
            line: Self::widest(
                offences
                    .iter()
                    .map(|offence| offence.line.to_string().len()),
                "line",
            ),
            rule: Self::widest(offences.iter().map(|offence| offence.rule.len()), "rule"),
            description: Self::widest(
                offences.iter().map(|offence| offence.description.len()),
                "offence",
            ),
        }
    }

    fn widest<I: Iterator<Item = usize>>(lengths: I, heading: &str) -> usize {
        lengths.max().unwrap_or(0).max(heading.len())
    }
}
