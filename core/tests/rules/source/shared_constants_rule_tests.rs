// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A module-level constant is private, or public and shared from a constants
// file: read by two or more other files of the workspace, from a file holding
// only imports, constants, statics and type aliases. Each public constant read
// by fewer, or shared from beside other items, and each constant of restricted
// visibility is reported, with the correction its case calls for. Statics,
// associated constants, inline modules, registries and tests/ are left alone.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use stern4rust::finding::model::name_readers::NameReaders;
use stern4rust::reporting::offence::Offence;
use stern4rust::rule::Rule;
use stern4rust::rules::source::shared_constants_rule::SharedConstantsRule;
use stern4rust::source_file::SourceFile;

const HEADER: &str = "// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>\n\
                      // Licensed under the MIT License\n\
                      // SPDX-License-Identifier: MIT\n";
const RULE: &str = "shared-constants";

fn check(path: &str, body: &str, read_by: &[&str]) -> Vec<Offence> {
    let readers = NameReaders::new(BTreeMap::from([(
        String::from("LIMIT"),
        read_by
            .iter()
            .map(|file| (*file).to_string())
            .collect::<BTreeSet<_>>(),
    )]))
    .in_package("node");
    SharedConstantsRule::new(readers).check(&SourceFile::new(path, &format!("{HEADER}\n{body}")))
}

#[test]
fn check_a_constant_in_an_inline_module_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/pool.rs",
        "pub struct Pool;\n\nmod limits {\n    pub const LIMIT: u32 = 3;\n}\n",
        &[],
    );

    // Assert
    assert!(offences.is_empty(), "{offences:?}");
}

#[test]
fn check_a_constant_its_own_file_and_one_other_read_says_to_share_it() {
    // Arrange & Act
    let offences = check(
        "src/baseline.rs",
        "pub const LIMIT: u32 = 3;\n\npub fn limit() -> u32 {\n    LIMIT\n}\n",
        &["node/src/baseline.rs", "node/tests/baseline_tests.rs"],
    );

    // Assert
    assert_eq!(offences.len(), 1, "{offences:?}");
    assert_eq!(
        offences[0].description,
        "`LIMIT` is read here and by `node/tests/baseline_tests.rs`, from a file that is not a \
         constants file"
    );
    assert_eq!(
        offences[0].correction,
        "move it into a constants file -- imports, constants, statics and type aliases only --, \
         such as `<topic>_constants.rs`, or make it private if `node/tests/baseline_tests.rs` \
         can do without it"
    );
}

#[test]
fn check_a_constant_one_other_file_reads_says_to_move_it_there() {
    // Arrange & Act
    let offences = check(
        "src/limits.rs",
        "pub const LIMIT: u32 = 3;\n",
        &["node/src/limits.rs", "node/src/pool.rs"],
    );

    // Assert
    assert_eq!(offences.len(), 1, "{offences:?}");
    assert_eq!(offences[0].rule, RULE);
    assert_eq!(offences[0].line, 5);
    assert_eq!(offences[0].subject.as_deref(), Some("LIMIT"));
    assert_eq!(
        offences[0].description,
        "`LIMIT` is public, and only `node/src/pool.rs` reads it"
    );
    assert_eq!(
        offences[0].correction,
        "move it into `node/src/pool.rs` and make it private"
    );
}

#[test]
fn check_a_constant_read_from_tests_and_a_sibling_crate_is_shared() {
    // Arrange & Act
    let offences = check(
        "src/limits.rs",
        "pub const LIMIT: u32 = 3;\n",
        &["node/tests/limits_tests.rs", "types/src/wire.rs"],
    );

    // Assert
    assert!(offences.is_empty(), "{offences:?}");
}

#[test]
fn check_a_private_constant_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/pool.rs",
        "const LIMIT: u32 = 3;\n\npub struct Pool;\n",
        &["node/src/pool.rs"],
    );

    // Assert
    assert!(offences.is_empty(), "{offences:?}");
}

#[test]
fn check_a_public_constant_no_other_file_reads_says_to_make_it_private() {
    // Arrange & Act
    let offences = check(
        "src/pool.rs",
        "pub const LIMIT: u32 = 3;\n\npub struct Pool;\n",
        &["node/src/pool.rs"],
    );

    // Assert
    assert_eq!(offences.len(), 1, "{offences:?}");
    assert_eq!(
        offences[0].description,
        "`LIMIT` is public, and no other file reads it"
    );
    assert_eq!(
        offences[0].correction,
        "make it private, or delete it if nothing reads it"
    );
}

#[test]
fn check_a_registry_reports_nothing() {
    // Arrange & Act
    let offences = check("src/lib.rs", "pub const LIMIT: u32 = 3;\n", &[]);

    // Assert
    assert!(offences.is_empty(), "{offences:?}");
}

#[test]
fn check_a_restricted_constant_says_private_or_public() {
    // Arrange & Act
    let offences = check(
        "src/limits.rs",
        "pub(crate) const LIMIT: u32 = 3;\npub(in crate::node) const PAGE: u32 = 8;\n",
        &[],
    );

    // Assert
    let described: Vec<&str> = offences
        .iter()
        .map(|offence| offence.description.as_str())
        .collect();
    assert_eq!(
        described,
        vec![
            "`LIMIT` is `pub(crate)`; a constant is private, or public and shared",
            "`PAGE` is `pub(in crate::node)`; a constant is private, or public and shared",
        ]
    );
    assert_eq!(
        offences[0].correction,
        "make it private, or `pub` and shared from a constants file"
    );
}

#[test]
fn check_a_shared_constant_beside_other_items_says_to_move_it_into_a_constants_file() {
    // Arrange & Act
    let offences = check(
        "src/pool.rs",
        "pub const LIMIT: u32 = 3;\n\npub struct Pool;\n",
        &[
            "node/src/executor.rs",
            "node/src/pool.rs",
            "types/src/wire.rs",
        ],
    );

    // Assert
    assert_eq!(offences.len(), 1, "{offences:?}");
    assert_eq!(
        offences[0].description,
        "`LIMIT` is read by 2 other files, from a file that is not a constants file"
    );
    assert_eq!(
        offences[0].correction,
        "move it into a constants file -- imports, constants, statics and type aliases only --, \
         such as `<topic>_constants.rs`"
    );
}

#[test]
fn check_a_shared_constant_in_a_constants_file_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/limits.rs",
        "use std::time::Duration;\n\npub const LIMIT: u32 = 3;\npub static NAME: &str = \"pool\";\n\
         pub type Count = u32;\n",
        &["node/src/executor.rs", "node/src/pool.rs"],
    );

    // Assert
    assert!(offences.is_empty(), "{offences:?}");
}

#[test]
fn check_a_static_and_an_associated_constant_report_nothing() {
    // Arrange & Act
    let offences = check(
        "src/pool.rs",
        "pub static HUB: u8 = 0;\n\npub struct Pool;\n\nimpl Pool {\n    pub const LIMIT: u32 = 3;\n}\n",
        &[],
    );

    // Assert
    assert!(offences.is_empty(), "{offences:?}");
}

#[test]
fn check_a_test_file_reports_nothing() {
    // Arrange & Act
    let offences = check("tests/pool_tests.rs", "pub const LIMIT: u32 = 3;\n", &[]);

    // Assert
    assert!(offences.is_empty(), "{offences:?}");
}

#[test]
fn check_an_unparseable_file_reports_nothing() {
    // Arrange & Act
    let offences = check("src/pool.rs", "pub const LIMIT: u32 = ;\nfn broken( {", &[]);

    // Assert
    assert!(offences.is_empty(), "{offences:?}");
}

#[test]
fn check_two_public_constants_nobody_reads_reports_each() {
    // Arrange & Act
    let offences = check(
        "src/pool.rs",
        "pub const LIMIT: u32 = 3;\npub const PAGE: u32 = 8;\n\npub struct Pool;\n",
        &[],
    );

    // Assert
    let subjects: Vec<Option<&str>> = offences
        .iter()
        .map(|offence| offence.subject.as_deref())
        .collect();
    assert_eq!(subjects, vec![Some("LIMIT"), Some("PAGE")]);
}

#[test]
fn check_workspace_reports_nothing() {
    // Arrange
    let files = [SourceFile::new(
        "src/pool.rs",
        "pub const LIMIT: u32 = 3;\n",
    )];

    // Act
    let offences = SharedConstantsRule::new(NameReaders::default()).check_workspace(&files);

    // Assert
    assert!(offences.is_empty(), "{offences:?}");
}

#[test]
fn explanation_breaks_the_rule_it_explains() {
    // Arrange
    let rule = SharedConstantsRule::new(NameReaders::default());
    let explanation = rule.explanation();

    // Act
    let broken = rule.check(&SourceFile::new("src/example.rs", explanation.breaks));
    let mended = rule.check(&SourceFile::new("src/example.rs", explanation.instead));

    // Assert
    assert_eq!(broken.len(), 1, "{broken:?}");
    assert!(mended.is_empty(), "{mended:?}");
}

#[test]
fn is_configured_needs_nothing() {
    // Arrange
    let rule = SharedConstantsRule::new(NameReaders::default());

    // Act & Assert
    assert!(rule.is_configured());
    assert_eq!(rule.requirement(), None);
}

#[test]
fn name_is_shared_constants() {
    // Arrange & Act & Assert
    assert_eq!(
        SharedConstantsRule::new(NameReaders::default()).name(),
        RULE
    );
}
