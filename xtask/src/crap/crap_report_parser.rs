// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::crap::crap_report::CrapReport;
use serde_json::from_str;

const OBJECT_OPEN: &str = "{";

pub struct CrapReportParser;

impl CrapReportParser {
    pub fn new() -> Self {
        Self
    }

    // crap4rust runs the crate's own test suite to gather coverage before it
    // reports, so its stdout carries that run's output ahead of the JSON. The
    // report is the last object opening at column zero; nested braces in the
    // payload are indented and cannot be mistaken for it.
    pub fn parse(&self, stdout: &str) -> Result<CrapReport, String> {
        let start = stdout
            .lines()
            .enumerate()
            .filter(|(_, line)| *line == OBJECT_OPEN)
            .map(|(index, _)| index)
            .last()
            .ok_or_else(|| String::from("could not find a JSON report in crap4rust's output"))?;

        let json_text = stdout.lines().skip(start).collect::<Vec<_>>().join("\n");

        from_str(&json_text).map_err(|error| format!("could not parse crap4rust JSON: {error}"))
    }
}

impl Default for CrapReportParser {
    fn default() -> Self {
        Self::new()
    }
}
