// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use serde::Deserialize;

const CLEAN_VERDICT: &str = "Clean";

#[derive(Debug, Clone, Deserialize)]
pub struct CrapFunction {
    pub name: String,
    pub relative_file: String,
    pub line: u32,
    pub complexity: u32,
    pub coverage: f64,
    pub crap_score: f64,
    pub verdict: String,
}

impl CrapFunction {
    pub fn is_clean(&self) -> bool {
        self.verdict == CLEAN_VERDICT
    }

    pub fn describe(&self) -> String {
        format!(
            "{}:{} {} (complexity {}, coverage {:.0}%, crap {:.1}) [{}]",
            self.relative_file,
            self.line,
            self.name,
            self.complexity,
            self.coverage * 100.0,
            self.crap_score,
            self.verdict,
        )
    }
}
