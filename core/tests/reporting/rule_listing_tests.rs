// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The rule set as a document, in both of the forms it is asked for.
//
// These moved here from report_printer_tests and json_printer_tests when the
// rendering left the printers. The assertions are the ones that were written
// against the printers, re-pointed rather than rewritten: the contract did not
// change, only the type that answers for it.

use stern4rust::reporting::rule_explanation::RuleExplanation;
use stern4rust::reporting::rule_listing::RuleListing;

fn explanations() -> [RuleExplanation; 1] {
    [RuleExplanation {
        name: "pure-traits",
        summary: "A trait declares; it does not implement.",
        breaks: "trait Store {\n    fn commit(&self) -> bool {\n        true\n    }\n}",
        instead: "trait Store {\n    fn commit(&self) -> bool;\n}",
    }]
}

// The listing is the one document with no run behind it, so it renders from the
// explanations alone. Both scraps are indented under a label rather than run
// together, because the question it answers is what the rule looks like.
#[test]
fn render_as_text_shows_each_rule_with_what_breaks_it_and_what_puts_it_right() {
    // Arrange
    let listing = explanations();

    // Act
    let report = RuleListing::new(&listing).render();

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
    let report = RuleListing::new(&[]).render();

    // Assert
    assert_eq!(report, "stern4rust rules\n");
}
