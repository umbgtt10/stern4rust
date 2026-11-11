// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use xtask::reports::scores::score_sheet::ScoreSheet;
use xtask::reports::scores::tool_score::ToolScore;

fn tool_score(version: &str, total: f64) -> ToolScore {
    ToolScore {
        version: String::from(version),
        total,
    }
}

// A core whose functions cost nothing has no ratio to show.
#[test]
fn render_of_a_core_with_no_braintax_shows_no_ratio() {
    // Arrange
    let sheet = ScoreSheet::new(tool_score("0.10.0", 3.80), tool_score("0.14.0", 0.0));

    // Act
    let rendered = sheet.render();

    // Assert
    assert_eq!(
        rendered,
        "Grip / braintax, core/ (cargo-grip4rust 0.10.0, cargo-braintax4rust 0.14.0):\n  core                3.80 /     0.00  -"
    );
}

// core's grip over its braintax, and their ratio -- the tools' versions named,
// since scores of different releases do not compare.
#[test]
fn render_shows_core_s_grip_and_braintax_with_their_ratio() {
    // Arrange
    let sheet = ScoreSheet::new(tool_score("0.10.0", 40.1), tool_score("0.14.0", 163.99));

    // Act
    let rendered = sheet.render();

    // Assert
    assert_eq!(
        rendered,
        "Grip / braintax, core/ (cargo-grip4rust 0.10.0, cargo-braintax4rust 0.14.0):\n  core               40.10 /   163.99  0.2445"
    );
}
