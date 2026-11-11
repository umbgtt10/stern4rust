// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::reports::scores::tool_score::ToolScore;

const CORE: &str = "core";

// core scored: its grip over its braintax and their ratio, under a title
// naming both tools' releases -- scores of different releases do not compare.
pub struct ScoreSheet {
    grip: ToolScore,
    braintax: ToolScore,
}

impl ScoreSheet {
    pub fn new(grip: ToolScore, braintax: ToolScore) -> Self {
        Self { grip, braintax }
    }

    pub fn render(&self) -> String {
        let grip = self.grip.total;
        let braintax = self.braintax.total;
        format!(
            "Grip / braintax, {CORE}/ (cargo-grip4rust {}, cargo-braintax4rust {}):\n  {CORE:<16}{grip:>8.2} / {braintax:>8.2}  {}",
            self.grip.version,
            self.braintax.version,
            Self::ratio(grip, braintax)
        )
    }

    // A core whose functions cost nothing has no ratio.
    fn ratio(grip: f64, braintax: f64) -> String {
        if braintax > 0.0 {
            format!("{:.4}", grip / braintax)
        } else {
            String::from("-")
        }
    }
}
