// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Keeps the report it was handed whole, so a test can read every part of it
// -- the rosters, the exclusions, the threshold -- rather than what a summary
// prints.

use std::cell::RefCell;
use std::string::String;
use stern4rust::reporting::rule_explanation::RuleExplanation;
use stern4rust::reporting::run_report::RunReport;
use stern4rust::traits::report_renderer::ReportRenderer;

#[derive(Default)]
pub struct CapturingReportRenderer {
    captured: RefCell<Option<RunReport>>,
}

impl CapturingReportRenderer {
    pub fn captured(&self) -> RunReport {
        self.captured
            .borrow()
            .clone()
            .expect("a report was rendered")
    }
}

impl ReportRenderer for CapturingReportRenderer {
    fn list(&self, _explanations: &[RuleExplanation]) -> String {
        String::new()
    }

    fn render(&self, report: &RunReport) -> String {
        self.captured.replace(Some(report.clone()));
        String::new()
    }
}
