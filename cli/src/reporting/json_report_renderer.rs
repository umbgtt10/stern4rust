// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use stern4rust::reporting::rule_explanation::RuleExplanation;
use stern4rust::reporting::run_report::RunReport;
use stern4rust::traits::report_renderer::ReportRenderer;

use crate::reporting::json_printer::JsonPrinter;
use crate::reporting::json_rule_listing::JsonRuleListing;

// `--format json`: the run and the rule listing as documents, for a gate script
// or an agent that would otherwise have to guess where one column of the table
// ends and the next begins.
#[derive(Debug, Clone, Copy, Default)]
pub struct JsonReportRenderer;

impl JsonReportRenderer {
    pub fn new() -> Self {
        Self
    }
}

impl ReportRenderer for JsonReportRenderer {
    fn list(&self, explanations: &[RuleExplanation]) -> String {
        JsonRuleListing::new(explanations).render()
    }

    fn render(&self, report: &RunReport) -> String {
        JsonPrinter::new(report.files_scanned)
            .with_threshold(report.threshold)
            .with_rules(
                report.applied.clone(),
                report.skipped.clone(),
                report
                    .unconfigured
                    .iter()
                    .map(|rule| rule.name.to_string())
                    .collect(),
            )
            .with_package_rosters(report.rosters.clone())
            .with_exclusions(report.exclusions.clone())
            .with_config_file(report.config_file.clone())
            .with_baseline(report.baseline.clone(), report.suppressed, report.stale)
            .with_fixed(report.fixed)
            .render(&report.offences)
    }
}
