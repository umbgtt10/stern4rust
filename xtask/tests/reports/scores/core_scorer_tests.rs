// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::process::command_runner_tests::FakeCommandRunner;
use xtask::process::launch_failure::LaunchFailure;
use xtask::reports::scores::core_scorer::CoreScorer;
use xtask::reports::scores::score_failure::ScoreFailure;
use xtask::reports::scores::score_sheet::ScoreSheet;
use xtask::reports::scores::tool_score::ToolScore;

const LISTING: &str = "core/Cargo.toml\0core/src/lib.rs\0core/tests/all_tests.rs\0";

fn answering() -> FakeCommandRunner {
    FakeCommandRunner::new().with_replies(&[
        LISTING,
        "",
        r#"{"version":"0.10.0","overall":{"grip_absolute_total":40.1}}"#,
        r#"{"version":"0.14.0","overall":{"total_braintax":163.99}}"#,
    ])
}

fn arguments(args: &[&str]) -> Vec<String> {
    args.iter().map(|arg| String::from(*arg)).collect()
}

#[test]
fn score_exports_the_staged_core_files_then_scores_core_there() {
    // Arrange
    let runner = answering();
    let scorer = CoreScorer::new(&runner, String::from("/scratch"));

    // Act
    scorer.score().expect("scored");

    // Assert
    assert_eq!(
        runner.calls(),
        [
            arguments(&["ls-files", "-z", "--", "core"]),
            arguments(&[
                "checkout-index",
                "--force",
                "--prefix=/scratch/",
                "--",
                "core/Cargo.toml",
                "core/src/lib.rs",
                "core/tests/all_tests.rs",
            ]),
            arguments(&["grip4rust", "--json", "/scratch/core"]),
            arguments(&["braintax4rust", "--json", "/scratch/core"]),
        ]
    );
    assert_eq!(runner.programs(), ["git", "git", "cargo", "cargo"]);
}

#[test]
fn score_of_a_failing_tool_names_its_command_with_what_it_said() {
    // Arrange
    let runner = answering().with_capturing_codes(&[Some(0), Some(0), Some(1)]);
    let scorer = CoreScorer::new(&runner, String::from("/scratch"));

    // Act
    let scored = scorer.score();

    // Assert
    assert_eq!(
        scored.err(),
        Some(ScoreFailure::CommandFailed {
            command: String::from("cargo grip4rust --json /scratch/core"),
            stderr: String::new(),
        })
    );
}

#[test]
fn score_of_a_runner_that_cannot_launch_fails_with_the_launch() {
    // Arrange
    let runner = FakeCommandRunner::new().with_launch_failure();
    let scorer = CoreScorer::new(&runner, String::from("/scratch"));

    // Act
    let scored = scorer.score();

    // Assert
    assert_eq!(
        scored.err(),
        Some(ScoreFailure::Launch(LaunchFailure {
            program: String::from("git"),
            reason: String::from("refused by the fake"),
        }))
    );
}

#[test]
fn score_scores_core_as_staged() {
    // Arrange
    let runner = answering();
    let scorer = CoreScorer::new(&runner, String::from("/scratch"));

    // Act
    let scored = scorer.score().map(|sheet| sheet.render());

    // Assert
    let expected = ScoreSheet::new(
        ToolScore {
            version: String::from("0.10.0"),
            total: 40.1,
        },
        ToolScore {
            version: String::from("0.14.0"),
            total: 163.99,
        },
    );
    assert_eq!(scored, Ok(expected.render()));
}

// A manifest, a test or a note is no source to score: core's own code is what
// counts.
#[test]
fn score_with_no_source_under_core_staged_fails_saying_so() {
    // Arrange
    let runner = FakeCommandRunner::new()
        .with_replies(&["core/Cargo.toml\0core/src/NOTES.md\0core/tests/all_tests.rs\0"]);
    let scorer = CoreScorer::new(&runner, String::from("/scratch"));

    // Act
    let scored = scorer.score();

    // Assert
    assert_eq!(scored.err(), Some(ScoreFailure::NoSource));
}
