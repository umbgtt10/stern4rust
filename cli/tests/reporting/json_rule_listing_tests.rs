// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The rule listing as a document, beside the text a person reads. The two must
// not give different pictures, which is the whole of ADR-MachineReadableReport.

use serde_json::Value;
use serde_json::from_str;
use stern4rust::reporting::rule_explanation::RuleExplanation;
use stern4rust::reporting::rule_listing::RuleListing;
use stern4rust_cli::reporting::json_rule_listing::JsonRuleListing;

fn explanations() -> [RuleExplanation; 1] {
    [RuleExplanation {
        name: "pure-traits",
        summary: "A trait declares; it does not implement.",
        breaks: "trait Store {\n    fn commit(&self) -> bool {\n        true\n    }\n}",
        instead: "trait Store {\n    fn commit(&self) -> bool;\n}",
    }]
}

// Same four fields the table shows, named rather than laid out, so an agent
// asking what a rule wants gets it without parsing indentation.
#[test]
fn render_as_json_carries_every_explanation_as_a_named_object() {
    // Arrange
    let listing = explanations();

    // Act
    let document: Value = from_str(&JsonRuleListing::new(&listing).render()).expect("valid json");

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

// The two forms must not give different pictures, which is the whole of
// ADR-MachineReadableReport. One type renders both from one list, so the way
// they could drift is the format argument being ignored.
#[test]
fn render_of_the_two_formats_names_the_same_rules() {
    // Arrange
    let listing = explanations();

    // Act
    let text = RuleListing::new(&listing).render();
    let json = JsonRuleListing::new(&listing).render();

    // Assert
    assert_ne!(text, json);
    let document: Value = from_str(&json).expect("valid json");
    for entry in document["rules"].as_array().expect("a rules array") {
        let name = entry["name"].as_str().expect("a name");
        assert!(text.contains(name), "{name} missing from the text listing");
    }
}
