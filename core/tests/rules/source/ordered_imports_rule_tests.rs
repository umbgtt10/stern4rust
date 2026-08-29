// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Imports in `src/` run in alphabetic order, on the pairs where the alphabet is
// the authority.
//
// The stand-downs are the whole design, and they are not concessions: `cargo
// fmt` runs first in the gate and orders `self`, `super`, `crate` and
// uppercase-initial paths by rules of its own. A rule demanding the alphabet
// there would write a file no edit could make green -- each run undoing the
// last. `ImportPath` already decides which pairs those are.

use stern4rust::reporting::offence::Offence;
use stern4rust::rule::Rule;
use stern4rust::rules::source::ordered_imports_rule::OrderedImportsRule;
use stern4rust::source_file::SourceFile;

const HEADER: &str = "// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>\n\
                      // Licensed under the MIT License\n\
                      // SPDX-License-Identifier: MIT\n";
const RULE: &str = "ordered-imports";

fn check(path: &str, body: &str) -> Vec<Offence> {
    OrderedImportsRule::new().check(&SourceFile::new(path, &format!("{HEADER}\n{body}")))
}

// `serde_json::Value` sorts before `serde_json::from_str` under rustfmt and
// after it under the alphabet. Only the pair is remarkable, not either path.
#[test]
fn check_a_case_divergence_inside_a_path_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/widget.rs",
        "use serde_json::Value;\nuse serde_json::from_str;\n\npub struct W;\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// rustfmt sorts `crate` ahead of every other path, so the alphabet has no say.
#[test]
fn check_a_crate_import_before_an_external_one_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/widget.rs",
        "use crate::alpha::Alpha;\nuse anyhow::Result;\n\npub struct W;\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_a_file_that_does_not_parse_reports_nothing() {
    // Arrange & Act
    let offences = check("src/widget.rs", "use zzz::Z\nuse aaa::A;\n");

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// The same pair the wrong way round is still a real disorder.
#[test]
fn check_a_path_prefixed_by_the_next_one_reports_it() {
    // Arrange & Act
    let offences = check(
        "src/widget.rs",
        "use aaa_crate::select::select4;\nuse aaa_crate::select::select;\n\npub struct W;\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
}

// Two imports where one path is a prefix of the other at the last segment.
// Compared as written, the shorter line ends in `;` (0x3B) where the longer
// carries on with `4` (0x34), so a raw text sort demands the longer first --
// while rustfmt compares parsed segments and demands the shorter. Found in
// `etheram-raft-embassy` on `select` beside `select4` and `Either` beside
// `Either4`, where `cargo fmt --check` was clean and this rule wanted the
// opposite order, so the corrections could not be applied at all.
#[test]
fn check_a_path_that_prefixes_the_next_one_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/widget.rs",
        "use aaa_crate::select::select;\nuse aaa_crate::select::select4;\n\npub struct W;\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// A `pub use` sorts among plain ones by its path. Compared as written, `pub`
// (0x70) precedes `use` (0x75), so every re-export read as belonging above
// every import regardless of what it named.
#[test]
fn check_a_public_re_export_is_ordered_by_its_path_not_its_visibility() {
    // Arrange & Act
    let offences = check(
        "src/widget.rs",
        "use aaa_crate::Alpha;\npub use zzz_crate::Zed;\n\npub struct W;\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_a_public_re_export_out_of_order_by_path_reports_it() {
    // Arrange & Act
    let offences = check(
        "src/widget.rs",
        "use zzz_crate::Zed;\npub use aaa_crate::Alpha;\n\npub struct W;\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
}

#[test]
fn check_a_sorted_import_block_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/widget.rs",
        "use aaa_crate::Alpha;\nuse std::fmt;\nuse zzz_crate::Zed;\n\npub struct W;\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// tests/ belongs to test-file-structure, which asks the same question of a
// stricter shape.
#[test]
fn check_a_test_file_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "tests/widget_tests.rs",
        "use zzz_crate::Zed;\nuse aaa_crate::Alpha;\n\npub struct W;\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// The offence quotes what the reader will search for, and that is the import,
// not the gate above it.
#[test]
fn check_an_out_of_order_attributed_import_names_the_import_not_the_attribute() {
    // Arrange & Act
    let offences = check(
        "src/widget.rs",
        "#[cfg(feature = \"alpha\")]\nuse zeta_crate::Zeta;\n#[cfg(feature = \"zeta\")]\nuse alpha_crate::Alpha;\n\npub struct W;\n",
    );

    // Assert
    assert_eq!(
        offences[0].subject.as_deref(),
        Some("use alpha_crate::Alpha;")
    );
    assert!(
        offences[0].description.contains("use zeta_crate::Zeta;"),
        "{:?}",
        offences[0].description
    );
}

#[test]
fn check_an_unsorted_import_block_reports_it() {
    // Arrange & Act
    let offences = check(
        "src/widget.rs",
        "use zzz_crate::Zed;\nuse aaa_crate::Alpha;\n\npub struct W;\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].rule, RULE);
    assert_eq!(offences[0].line, 6);
    assert_eq!(
        offences[0].subject.as_deref(),
        Some("use aaa_crate::Alpha;")
    );
    assert!(
        offences[0].description.contains("out of alphabetic order"),
        "got {}",
        offences[0].description
    );
    assert_eq!(
        offences[0].correction,
        "move `use aaa_crate::Alpha;` above `use zzz_crate::Zed;`"
    );
}

// An uppercase-initial crate sorts behind every lowercase one under rustfmt.
#[test]
fn check_an_uppercase_first_segment_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/widget.rs",
        "use Bbb::gamma;\nuse aaa_crate::Alpha;\n\npub struct W;\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// An item's span starts at its first attribute, so reading the line the span
// begins on gave a cfg-gated import its `#[cfg(...)]` line as the thing to
// sort. `embassy-logging` has four such imports and every one was reported,
// with corrections that ordered them by feature name: following all of them
// would have put `rtt_sink` above `qemu_sink` above `host_sink` -- the paths
// in reverse, by the rule that exists to alphabetise them.
#[test]
fn check_attributed_imports_in_order_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/widget.rs",
        "#[cfg(feature = \"zeta\")]\nuse alpha_crate::Alpha;\n#[cfg(feature = \"alpha\")]\nuse zeta_crate::Zeta;\n\npub struct W;\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// The attribute must not hide a real disorder either. Same two attributes, the
// paths the wrong way round.
#[test]
fn check_attributed_imports_out_of_order_reports_it() {
    // Arrange & Act
    let offences = check(
        "src/widget.rs",
        "#[cfg(feature = \"alpha\")]\nuse zeta_crate::Zeta;\n#[cfg(feature = \"zeta\")]\nuse alpha_crate::Alpha;\n\npub struct W;\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
}

// A blank line ends a block, so the first import of the next one is compared
// with nothing.
#[test]
fn check_blocks_separated_by_a_blank_line_are_ordered_apart() {
    // Arrange & Act
    let offences = check(
        "src/widget.rs",
        "use zzz_crate::Zed;\n\nuse aaa_crate::Alpha;\n\npub struct W;\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_reports_every_unordered_pair() {
    // Arrange & Act
    let offences = check(
        "src/widget.rs",
        "use ccc_crate::C;\nuse bbb_crate::B;\nuse aaa_crate::A;\n\npub struct W;\n",
    );

    // Assert
    assert_eq!(offences.len(), 2);
    assert_eq!(offences[0].subject.as_deref(), Some("use bbb_crate::B;"));
    assert_eq!(offences[1].subject.as_deref(), Some("use aaa_crate::A;"));
}

#[test]
fn check_workspace_of_a_tree_reports_nothing() {
    // Arrange
    let file = SourceFile::new("src/widget.rs", "use zzz::Z;\nuse aaa::A;\n");

    // Act
    let offences = OrderedImportsRule::new().check_workspace(&[file]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn is_configured_for_a_rule_that_needs_nothing_returns_true() {
    // Arrange & Act
    let configured = OrderedImportsRule::new().is_configured();

    // Assert
    assert!(configured);
}

#[test]
fn name_is_the_kebab_case_rule_name_used_in_the_report() {
    // Arrange & Act
    let name = OrderedImportsRule::new().name();

    // Assert
    assert_eq!(name, RULE);
}
