// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// What one rule wants, in its own words: a sentence, a scrap of source that breaks it, and the scrap put right.

use stern4rust::reporting::rule_explanation::RuleExplanation;

fn explanation() -> RuleExplanation {
    RuleExplanation::new(
        "header",
        "every source file opens with the house header",
        "fn main() {}",
        "// Copyright\nfn main() {}",
    )
}

#[test]
fn new_keeps_every_field_it_was_given() {
    // Arrange & Act
    let found = explanation();

    // Assert
    assert_eq!(found.name(), "header");
    assert_eq!(
        found.summary(),
        "every source file opens with the house header"
    );
    assert_eq!(found.breaks(), "fn main() {}");
    assert_eq!(found.instead(), "// Copyright\nfn main() {}");
}
