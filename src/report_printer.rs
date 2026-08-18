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
        println!("stern4rust report");
        println!();

        if offences.is_empty() {
            println!("All rules are satisfied.");
            println!();
            self.print_summary(offences);
            return;
        }

        let widths = ColumnWidths::of(offences);
        println!(
            "{:<file$}  {:>line$}  {:<rule$}  offence",
            "file",
            "line",
            "rule",
            file = widths.file,
            line = widths.line,
            rule = widths.rule
        );
        println!(
            "{}  {}  {}  {}",
            "-".repeat(widths.file),
            "-".repeat(widths.line),
            "-".repeat(widths.rule),
            "-".repeat(widths.description)
        );
        for offence in offences {
            println!(
                "{:<file$}  {:>line$}  {:<rule$}  {}",
                offence.file,
                offence.line,
                offence.rule,
                offence.description,
                file = widths.file,
                line = widths.line,
                rule = widths.rule
            );
        }
        println!();
        self.print_summary(offences);
    }

    fn print_summary(&self, offences: &[Offence]) {
        let broken: BTreeSet<&str> = offences.iter().map(|offence| offence.rule).collect();
        println!(
            "summary: files_scanned={} offences={} rules_broken={}",
            self.files_scanned,
            offences.len(),
            broken.len()
        );
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
