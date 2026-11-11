// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::process::command_runner::CommandRunner;
use crate::reports::scores::score_failure::ScoreFailure;
use crate::reports::scores::score_sheet::ScoreSheet;
use crate::reports::scores::tool_score::ToolScore;
use crate::reports::scores::tool_score_parser::ToolScoreParser;

const BRAINTAX: &str = "braintax4rust";
const BRAINTAX_TOOL: &str = "cargo-braintax4rust";
const BRAINTAX_TOTAL: &str = "total_braintax";
const CARGO: &str = "cargo";
const CORE: &str = "core";
const GIT: &str = "git";
const GRIP: &str = "grip4rust";
const GRIP_TOOL: &str = "cargo-grip4rust";
const GRIP_TOTAL: &str = "grip_absolute_total";
const RUST: &str = ".rs";
const SOURCE: &str = "core/src/";

// core scored as it is staged: the files under core/ that the index holds --
// the one git hands a commit hook -- exported into a scratch directory, so
// nothing unstaged is scored, and scored there by cargo-grip4rust and
// cargo-braintax4rust. A manifest or a test alone is nothing to score.
pub struct CoreScorer<'a> {
    runner: &'a dyn CommandRunner,
    scratch: String,
}

impl<'a> CoreScorer<'a> {
    pub fn new(runner: &'a dyn CommandRunner, scratch: String) -> Self {
        Self { runner, scratch }
    }

    pub fn score(&self) -> Result<ScoreSheet, ScoreFailure> {
        let listing = self.capture(GIT, &["ls-files", "-z", "--", CORE])?;
        let files: Vec<&str> = listing
            .split('\0')
            .filter(|file| !file.is_empty())
            .collect();
        if !files
            .iter()
            .any(|file| file.starts_with(SOURCE) && file.ends_with(RUST))
        {
            return Err(ScoreFailure::NoSource);
        }
        let prefix = format!("--prefix={}/", self.scratch);
        let mut export = vec!["checkout-index", "--force", prefix.as_str(), "--"];
        export.extend(files);
        self.capture(GIT, &export)?;
        let directory = format!("{}/{CORE}", self.scratch);
        let grip = self.measured(GRIP, GRIP_TOOL, GRIP_TOTAL, &directory)?;
        let braintax = self.measured(BRAINTAX, BRAINTAX_TOOL, BRAINTAX_TOTAL, &directory)?;
        Ok(ScoreSheet::new(grip, braintax))
    }

    fn capture(&self, program: &str, args: &[&str]) -> Result<String, ScoreFailure> {
        let args: Vec<String> = args.iter().map(|arg| String::from(*arg)).collect();
        let outcome = self.runner.run_capturing(program, &args)?;
        if !outcome.is_success() {
            return Err(ScoreFailure::CommandFailed {
                command: format!("{program} {}", args.join(" ")),
                stderr: String::from(outcome.stderr()),
            });
        }
        Ok(String::from(outcome.stdout()))
    }

    fn measured(
        &self,
        subcommand: &str,
        tool: &str,
        total: &str,
        directory: &str,
    ) -> Result<ToolScore, ScoreFailure> {
        let report = self.capture(CARGO, &[subcommand, "--json", directory])?;
        ToolScoreParser::parse(tool, &report, total)
    }
}
