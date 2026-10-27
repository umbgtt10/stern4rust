// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use stern4rust::reporting::rule_explanation::RuleExplanation;
use stern4rust::reporting::run_report::RunReport;
use stern4rust::traits::report_renderer::ReportRenderer;

// A renderer that writes nothing a reader would want: the counts a run handed
// over on one line, then one `<file>:<line> <rule>` line per offence, in the
// order it was handed them -- or, for a listing, one rule name per line.
pub struct SummaryReportRenderer;

impl ReportRenderer for SummaryReportRenderer {
    fn list(&self, explanations: &[RuleExplanation]) -> String {
        explanations
            .iter()
            .map(|explanation| format!("{}\n", explanation.name))
            .collect()
    }

    fn render(&self, report: &RunReport) -> String {
        let mut out = format!(
            "scanned={} fixed={} baselined={} stale={} applied={} config={}\n",
            report.files_scanned,
            report.fixed,
            report.suppressed,
            report.stale,
            report.applied.len(),
            report.config_file.as_deref().unwrap_or("none"),
        );
        for offence in &report.offences {
            out.push_str(&format!(
                "{}:{} {}\n",
                offence.file(),
                offence.line(),
                offence.rule()
            ));
        }
        out
    }
}
