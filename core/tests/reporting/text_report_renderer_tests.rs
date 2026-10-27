// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use stern4rust::reporting::offence::Offence;
use stern4rust::reporting::offence_threshold::OffenceThreshold;
use stern4rust::reporting::report_printer::ReportPrinter;
use stern4rust::reporting::rule_explanation::RuleExplanation;
use stern4rust::reporting::rule_listing::RuleListing;
use stern4rust::reporting::run_report::RunReport;
use stern4rust::reporting::text_report_renderer::TextReportRenderer;
use stern4rust::reporting::unconfigured_rule::UnconfiguredRule;
use stern4rust::traits::report_renderer::ReportRenderer;

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
        offences: vec![
            Offence::new(
                "src/a.rs",
                1,
                "header",
                String::from("the header is missing"),
                String::from("add it"),
            ),
            Offence::new(
                "src/b.rs",
                1,
                "header",
                String::from("the header is missing"),
                String::from("add it"),
            ),
        ],
    }
}

#[test]
fn list_is_the_text_rule_listing() {
    // Arrange
    let explanations = [RuleExplanation {
        name: "pure-traits",
        summary: "A trait declares; it does not implement.",
        breaks: "trait Store {}",
        instead: "trait Store;",
    }];

    // Act
    let listing = TextReportRenderer::new().list(&explanations);

    // Assert
    assert_eq!(listing, RuleListing::new(&explanations).render());
}

// Everything the run handed over reaches the table, through the printer that
// has always drawn it.
#[test]
fn render_draws_the_report_the_printer_draws_from_the_same_run() {
    // Arrange
    let report = report();

    // Act
    let rendered = TextReportRenderer::new().render(&report);

    // Assert
    assert_eq!(
        rendered,
        ReportPrinter::new(3)
            .with_threshold(OffenceThreshold::new(1))
            .with_rules(
                vec![String::from("header")],
                vec![String::from("tests-layout")],
                vec![(
                    String::from("spdx-matches-manifest"),
                    String::from("a licence in the manifest")
                )],
            )
            .with_package_rosters(Vec::new())
            .with_exclusions(vec![(String::from("gone/**"), 0)])
            .with_config_file(Some(String::from("stern4rust.toml")))
            .with_baseline(Some(String::from("stern4rust-baseline.json")), 2, 1)
            .with_fixed(1)
            .render(&report.offences)
    );
    assert!(rendered.contains("src/a.rs"), "{rendered}");
    assert!(!rendered.contains("src/b.rs"), "{rendered}");
}
