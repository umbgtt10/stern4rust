// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::reporting::rule_explanation::RuleExplanation;
use crate::reporting::run_report::RunReport;

// What a run says, in the form the reader asked for: the report of a scan, or
// the rule listing a run without one prints.
pub trait ReportRenderer {
    fn list(&self, explanations: &[RuleExplanation]) -> String;

    fn render(&self, report: &RunReport) -> String;
}
