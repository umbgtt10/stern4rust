// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use stern4rust::reporting::offence::Offence;
use stern4rust::reporting::offence_threshold::OffenceThreshold;
use stern4rust::reporting::rule_explanation::RuleExplanation;
use stern4rust::reporting::run_report::RunReport;
use stern4rust::reporting::unconfigured_rule::UnconfiguredRule;
use stern4rust::traits::report_renderer::ReportRenderer;
use stern4rust_cli::reporting::json_printer::JsonPrinter;
use stern4rust_cli::reporting::json_report_renderer::JsonReportRenderer;
use stern4rust_cli::reporting::json_rule_listing::JsonRuleListing;

fn report() -> RunReport {
    RunReport {
        files_scanned: 3,
        threshold: OffenceThreshold::new(1),
        applied: vec![String::from("header")],
        skipped: vec![String::from("tests-layout")],
        unconfigured: vec![UnconfiguredRule {
            name: "spdx-matches-manifest",
            requirement: "a licence in the manifest",
        }],
        rosters: Vec::new(),
        exclusions: vec![(String::from("gone/**"), 0)],
        config_file: Some(String::from("stern4rust.toml")),
        baseline: Some(String::from("stern4rust-baseline.json")),
        suppressed: 2,
        stale: 1,
        fixed: 1,
        offences: vec![Offence::new(
            "src/a.rs",
            1,
            "header",
            String::from("the header is missing"),
            String::from("add it"),
        )],
    }
}

#[test]
fn list_is_the_json_rule_listing() {
    // Arrange
    let explanations = [RuleExplanation {
        name: "pure-traits",
        summary: "A trait declares; it does not implement.",
        breaks: "trait Store {}",
        instead: "trait Store;",
    }];

    // Act
    let listing = JsonReportRenderer::new().list(&explanations);

    // Assert
    assert_eq!(listing, JsonRuleListing::new(&explanations).render());
}

// The document names an unconfigured rule by name alone; the table says what
// it waits for.
#[test]
fn render_draws_the_document_the_printer_draws_from_the_same_run() {
    // Arrange
    let report = report();

    // Act
    let rendered = JsonReportRenderer::new().render(&report);

    // Assert
    assert_eq!(
        rendered,
        JsonPrinter::new(3)
            .with_threshold(OffenceThreshold::new(1))
            .with_rules(
                vec![String::from("header")],
                vec![String::from("tests-layout")],
                vec![String::from("spdx-matches-manifest")],
            )
            .with_package_rosters(Vec::new())
            .with_exclusions(vec![(String::from("gone/**"), 0)])
            .with_config_file(Some(String::from("stern4rust.toml")))
            .with_baseline(Some(String::from("stern4rust-baseline.json")), 2, 1)
            .with_fixed(1)
            .render(&report.offences)
    );
}
