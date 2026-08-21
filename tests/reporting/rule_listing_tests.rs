// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The rule set as a document, in both of the forms it is asked for.
//
// These moved here from report_printer_tests and json_printer_tests when the
// rendering left the printers. The assertions are the ones that were written
// against the printers, re-pointed rather than rewritten: the contract did not
// change, only the type that answers for it.

use serde_json::Value;
use serde_json::from_str;
use stern4rust::reporting::output_format::OutputFormat;
use stern4rust::reporting::rule_explanation::RuleExplanation;
use stern4rust::reporting::rule_listing::RuleListing;

fn explanations() -> [RuleExplanation; 1] {
    [RuleExplanation::new(
        "pure-traits",
        "A trait declares; it does not implement.",
        "trait Store {\n    fn commit(&self) -> bool {\n        true\n    }\n}",
        "trait Store {\n    fn commit(&self) -> bool;\n}",
    )]
}

// Same four fields the table shows, named rather than laid out, so an agent
// asking what a rule wants gets it without parsing indentation.
#[test]
fn render_as_json_carries_every_explanation_as_a_named_object() {
    // Arrange
    let listing = explanations();

    // Act
    let document: Value =
        from_str(&RuleListing::new(&listing).render(OutputFormat::Json)).expect("valid json");

    // Assert
    let entries = document["rules"].as_array().expect("a rules array");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["name"], "pure-traits");
    assert_eq!(
        entries[0]["summary"],
        "A trait declares; it does not implement."
    );
    assert_eq!(
        entries[0]["instead"],
        "trait Store {\n    fn commit(&self) -> bool;\n}"
    );
}

// The listing is the one document with no run behind it, so it renders from the
// explanations alone. Both scraps are indented under a label rather than run
// together, because the question it answers is what the rule looks like.
#[test]
fn render_as_text_shows_each_rule_with_what_breaks_it_and_what_puts_it_right() {
    // Arrange
    let listing = explanations();

    // Act
    let report = RuleListing::new(&listing).render(OutputFormat::Text);

    // Assert
    assert!(report.contains("pure-traits"), "{report}");
    assert!(
        report.contains("A trait declares; it does not implement."),
        "{report}"
    );
    assert!(report.contains("breaks:"), "{report}");
    assert!(report.contains("instead:"), "{report}");
    assert!(report.contains("fn commit(&self) -> bool;"), "{report}");
}

#[test]
fn render_of_no_rules_still_names_the_tool() {
    // Arrange & Act
    let report = RuleListing::new(&[]).render(OutputFormat::Text);

    // Assert
    assert_eq!(report, "stern4rust rules\n");
}

// The two forms must not give different pictures, which is the whole of
// ADR-MachineReadableReport. One type renders both from one list, so the way
// they could drift is the format argument being ignored.
#[test]
fn render_of_the_two_formats_names_the_same_rules() {
    // Arrange
    let listing = explanations();

    // Act
    let text = RuleListing::new(&listing).render(OutputFormat::Text);
    let json = RuleListing::new(&listing).render(OutputFormat::Json);

    // Assert
    assert_ne!(text, json);
    let document: Value = from_str(&json).expect("valid json");
    for entry in document["rules"].as_array().expect("a rules array") {
        let name = entry["name"].as_str().expect("a name");
        assert!(text.contains(name), "{name} missing from the text listing");
    }
}
