// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use crate::reporting::report_printer::ReportPrinter;
use crate::reporting::rule_explanation::RuleExplanation;
use crate::reporting::rule_listing::RuleListing;
use crate::reporting::run_report::RunReport;
use crate::traits::report_renderer::ReportRenderer;

// The table a person reads, and the rule listing beside it.
#[derive(Debug, Clone, Copy, Default)]
pub struct TextReportRenderer;

impl TextReportRenderer {
    pub fn new() -> Self {
        Self
    }
}

impl ReportRenderer for TextReportRenderer {
    fn list(&self, explanations: &[RuleExplanation]) -> String {
        RuleListing::new(explanations).render()
    }

    fn render(&self, report: &RunReport) -> String {
        let unconfigured: Vec<(String, String)> = report
            .unconfigured
            .iter()
            .map(|rule| (rule.name.to_string(), rule.requirement.to_string()))
            .collect();
        ReportPrinter::new(report.files_scanned)
            .with_threshold(report.threshold)
            .with_rules(report.applied.clone(), report.skipped.clone(), unconfigured)
            .with_package_rosters(report.rosters.clone())
            .with_exclusions(report.exclusions.clone())
            .with_config_file(report.config_file.clone())
            .with_baseline(report.baseline.clone(), report.suppressed, report.stale)
            .with_fixed(report.fixed)
            .render(&report.offences)
    }
}
