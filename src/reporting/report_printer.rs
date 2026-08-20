// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::reporting::column_widths::ColumnWidths;
use crate::reporting::offence::Offence;
use crate::reporting::offence_threshold::OffenceThreshold;
use std::collections::BTreeSet;

// One table for every rule. Columns are sized to their contents so the report
// stays readable when a rule name or a path grows, and the summary line is
// greppable so a wrapper script can report a count without parsing the table.
pub struct ReportPrinter {
    files_scanned: usize,
    threshold: OffenceThreshold,
    applied: Vec<String>,
    skipped: Vec<String>,
    unconfigured: Vec<(String, String)>,
    exclusions: Vec<(String, usize)>,
    config_file: Option<String>,
    baseline: Option<String>,
    suppressed: usize,
    stale: usize,
    fixed: usize,
}

impl ReportPrinter {
    pub fn new(files_scanned: usize) -> Self {
        Self {
            files_scanned,
            threshold: OffenceThreshold::default(),
            applied: Vec::new(),
            skipped: Vec::new(),
            unconfigured: Vec::new(),
            exclusions: Vec::new(),
            config_file: None,
            baseline: None,
            suppressed: 0,
            stale: 0,
            fixed: 0,
        }
    }

    // How many files --fix repaired. Stated alongside what is left, because a
    // fixer reporting only its successes would be the same silence this tool
    // refuses everywhere else.
    pub fn with_fixed(self, fixed: usize) -> Self {
        Self { fixed, ..self }
    }

    // A run that reported nothing while a baseline hid four hundred findings
    // would be the most comfortable lie this tool could tell, so the count is
    // in the summary of every run that used one -- including when it is the
    // whole story and the report itself is empty.
    pub fn with_baseline(self, baseline: Option<String>, suppressed: usize, stale: usize) -> Self {
        Self {
            baseline,
            suppressed,
            stale,
            ..self
        }
    }

    // A run configured by a file the reader never typed on the command line
    // must say so, or the switches in force are invisible.
    pub fn with_config_file(self, config_file: Option<String>) -> Self {
        Self {
            config_file,
            ..self
        }
    }

    pub fn with_exclusions(self, exclusions: Vec<(String, usize)>) -> Self {
        Self { exclusions, ..self }
    }

    pub fn with_rules(
        self,
        applied: Vec<String>,
        skipped: Vec<String>,
        unconfigured: Vec<(String, String)>,
    ) -> Self {
        Self {
            applied,
            skipped,
            unconfigured,
            ..self
        }
    }

    pub fn with_threshold(self, threshold: OffenceThreshold) -> Self {
        Self { threshold, ..self }
    }

    pub fn print(&self, offences: &[Offence]) {
        println!("{}", self.render(offences));
    }

    // Returns the report rather than writing it, so the shape is assertable in a
    // test instead of being checked for not panicking.
    pub fn render(&self, offences: &[Offence]) -> String {
        let mut report = String::from("stern4rust report\n\n");
        if offences.is_empty() {
            report.push_str(self.clean_verdict());
            report.push_str("\n\n");
            report.push_str(&self.roster());
            report.push_str(&self.exclusion_roster());
            report.push_str(&self.config_line());
            report.push_str(&self.baseline_line());
            report.push_str(&self.fixed_line());
            report.push_str(&self.summary(offences));
            return report;
        }

        // Sized to what is shown, so one withheld offence with a very long path
        // cannot widen a column nothing in the report occupies.
        let shown = self.threshold.kept(offences);
        let widths = ColumnWidths::of(shown);
        report.push_str(&Self::heading(&widths));
        for offence in shown {
            report.push_str(&Self::row(offence, &widths));
            report.push_str(&Self::correction_row(offence, &widths));
        }
        report.push('\n');
        report.push_str(&self.omission(offences));
        report.push_str(&self.roster());
        report.push_str(&self.exclusion_roster());
        report.push_str(&self.config_line());
        report.push_str(&self.baseline_line());
        report.push_str(&self.fixed_line());
        report.push_str(&self.summary(offences));
        report
    }

    // "All rules are satisfied" is only true when all of them ran. Saying it
    // after --skip turned two off, or after the header rule was dropped for
    // want of a header file, would be the tool telling the comfortable lie it
    // exists to catch.
    fn clean_verdict(&self) -> &'static str {
        if self.everything_ran() {
            "All rules are satisfied."
        } else {
            "All applied rules are satisfied."
        }
    }

    fn everything_ran(&self) -> bool {
        self.skipped.is_empty() && self.unconfigured.is_empty()
    }

    // Named, not counted. A count answers "how many", which is only useful to a
    // reader who already knows how many there are.
    fn roster(&self) -> String {
        if self.applied.is_empty() {
            return String::new();
        }
        let mut roster = format!("  applied: {}\n", self.applied.join(", "));
        if !self.everything_ran() {
            roster.push_str(&format!("  not applied: {}\n", self.absences().join(", ")));
        }
        roster.push('\n');
        roster
    }

    // What --fix repaired, stated beside what it could not. A fixer reporting
    // only its successes would leave the reader believing the file is done.
    fn fixed_line(&self) -> String {
        if self.fixed == 0 {
            return String::new();
        }
        format!("  fixed: {} file(s) rewritten\n\n", self.fixed)
    }

    // Named with its count, because a run that reported nothing while a
    // baseline hid four hundred findings would be the most comfortable lie this
    // tool could tell. A stale entry is called out for the same reason a dead
    // --exclude pattern is: it describes an offence somebody has since fixed,
    // and until the file is rewritten it makes the baseline look like it is
    // still holding something back.
    fn baseline_line(&self) -> String {
        let Some(path) = &self.baseline else {
            return String::new();
        };
        let mut line = format!("  baseline: {path} ({} suppressed)\n", self.suppressed);
        if self.stale > 0 {
            line.push_str(&format!(
                "  {} baseline entries matched nothing -- rerun with --write-baseline to \
                 refresh it\n",
                self.stale
            ));
        }
        line.push('\n');
        line
    }

    // A run configured by a file the reader never typed must say which file.
    // Every switch in force would otherwise be invisible, and a report that
    // applied one rule because of a line in a .toml would look exactly like one
    // that applied one rule because somebody asked for it.
    fn config_line(&self) -> String {
        match &self.config_file {
            Some(path) => format!("  config: {path}\n\n"),
            None => String::new(),
        }
    }

    // Every pattern with the number of files it removed, including zero. A
    // pattern that matched nothing is the one the reader most needs to see:
    // it names a tree that has moved or been deleted, and until somebody is
    // told, it goes on looking like it is doing work.
    fn exclusion_roster(&self) -> String {
        if self.exclusions.is_empty() {
            return String::new();
        }
        let listed: Vec<String> = self
            .exclusions
            .iter()
            .map(|(pattern, count)| format!("{pattern} ({count} files)"))
            .collect();
        let mut roster = format!("  excluded: {}\n", listed.join(", "));
        let dead = self.unmatched();
        if !dead.is_empty() {
            roster.push_str(&format!(
                "  matched nothing: {} -- delete the pattern or correct it\n",
                dead.join(", ")
            ));
        }
        roster.push('\n');
        roster
    }

    fn unmatched(&self) -> Vec<&str> {
        self.exclusions
            .iter()
            .filter(|(_, count)| *count == 0)
            .map(|(pattern, _)| pattern.as_str())
            .collect()
    }

    // Skipped and unconfigured are both "did not run" and are not the same
    // thing. One is a choice the reader made; the other is a flag they did not
    // pass, and saying which is the difference between a note and an
    // instruction.
    fn absences(&self) -> Vec<String> {
        self.skipped
            .iter()
            .map(|name| format!("{name} (skipped)"))
            .chain(
                self.unconfigured
                    .iter()
                    .map(|(name, requirement)| format!("{name} ({requirement})")),
            )
            .collect()
    }

    // Named alongside the flag that raises it. A cap nobody was told about reads
    // as "that was all of them", which is the one thing this report must never
    // say when it is not true.
    fn omission(&self, offences: &[Offence]) -> String {
        let omitted = self.threshold.omitted(offences);
        if omitted == 0 {
            return String::new();
        }
        format!(
            "... and {omitted} more offences not shown. Raise --offence-threshold \
             (currently {}, use 0 for all) to see them.\n\n",
            self.threshold.limit()
        )
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

    fn excluded_total(&self) -> usize {
        self.exclusions.iter().map(|(_, count)| count).sum()
    }

    fn summary(&self, offences: &[Offence]) -> String {
        let broken: BTreeSet<&str> = offences.iter().map(|offence| offence.rule).collect();
        format!(
            "summary: files_scanned={} files_excluded={} offences={} baselined={} fixed={} \
             rules_broken={} rules_applied={} rules_skipped={} rules_unconfigured={}",
            self.files_scanned,
            self.excluded_total(),
            offences.len(),
            self.suppressed,
            self.fixed,
            broken.len(),
            self.applied.len(),
            self.skipped.len(),
            self.unconfigured.len()
        )
    }
}
