// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The shape of the tests tree itself, rather than of any one file in it.
//
// A tests folder is reached through exactly one door: `tests/all_tests.rs`, and
// a `mod.rs` in every subfolder below it. Miss one and the files beneath it are
// not compiled at all -- they still exist, still look like tests, and still get
// counted by anyone reading the directory, but nothing runs them. That is the
// failure this rule exists for, and it is silent by construction: a test that is
// never compiled cannot fail.
//
// Both registry files hold nothing but the header and `pub mod` declarations.
// Anything else in them is logic living where nobody looks for it.

use stern4rust::rule::Rule;
use stern4rust::rules::layout::tests_layout_rule::TestsLayoutRule;
use stern4rust::source_file::SourceFile;

const HEADER: &str = "// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>\n\
                      // Licensed under the MIT License\n\
                      // SPDX-License-Identifier: MIT\n";

const REGISTRY: &str = "pub mod alpha_tests;\npub mod beta_tests;\n";

const RULE: &str = "tests-layout";

const TEST_BODY: &str = "#[test]\nfn alpha_does_something() {}\n";

fn check(files: &[(&str, &str)]) -> Vec<stern4rust::reporting::offence::Offence> {
    let files: Vec<SourceFile> = files
        .iter()
        .map(|(path, body)| SourceFile::new(path, &format!("{HEADER}\n{body}")))
        .collect();
    TestsLayoutRule::new().check_workspace(&files)
}

fn descriptions(files: &[(&str, &str)]) -> String {
    check(files)
        .into_iter()
        .map(|offence| format!("{} {}", offence.file, offence.description))
        .collect::<Vec<String>>()
        .join(" | ")
}

// A rule that judges the tree has nothing to say about one file on its own, and
// must not report the same thing twice through both doors.
#[test]
fn check_of_a_single_file_reports_nothing() {
    // Arrange
    let file = SourceFile::new(
        "tests/all_tests.rs",
        &format!("{HEADER}\nfn helper() {{}}\n"),
    );

    // Act
    let offences = TestsLayoutRule::new().check(&file);

    // Assert
    assert!(offences.is_empty());
}

#[test]
fn check_workspace_a_crate_without_a_tests_folder_reports_nothing() {
    // Arrange & Act
    let offences = check(&[("src/lib.rs", "pub mod alpha;\n")]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// Recursive. A folder two levels down needs its own door as much as the first.
#[test]
fn check_workspace_a_deeply_nested_subfolder_without_a_mod_file_reports_it_missing() {
    // Arrange & Act
    let found = descriptions(&[
        ("tests/all_tests.rs", REGISTRY),
        ("tests/rules/mod.rs", REGISTRY),
        ("tests/rules/deep/alpha_tests.rs", TEST_BODY),
    ]);

    // Assert
    assert!(found.contains("tests/rules/deep/mod.rs"), "got {found}");
}

#[test]
fn check_workspace_a_flat_tests_folder_with_its_registry_reports_nothing() {
    // Arrange & Act
    let offences = check(&[
        ("tests/all_tests.rs", REGISTRY),
        ("tests/alpha_tests.rs", TEST_BODY),
    ]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_a_nested_tests_folder_with_every_registry_reports_nothing() {
    // Arrange & Act
    let offences = check(&[
        ("tests/all_tests.rs", REGISTRY),
        ("tests/rules/mod.rs", REGISTRY),
        ("tests/rules/alpha_tests.rs", TEST_BODY),
    ]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_a_registry_holding_a_constant_reports_it() {
    // Arrange & Act
    let offences = check(&[
        ("tests/all_tests.rs", REGISTRY),
        (
            "tests/rules/mod.rs",
            "const LIMIT: usize = 1;\n\npub mod alpha_tests;\n",
        ),
        ("tests/rules/alpha_tests.rs", TEST_BODY),
    ]);

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].file, "tests/rules/mod.rs");
    assert_eq!(offences[0].line, 5);
    assert!(
        offences[0].description.contains("the constant `LIMIT`"),
        "got {}",
        offences[0].description
    );
}

#[test]
fn check_workspace_a_registry_holding_a_function_reports_it() {
    // Arrange & Act
    let offences = check(&[(
        "tests/all_tests.rs",
        "pub mod alpha_tests;\n\nfn helper() {}\n",
    )]);

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].file, "tests/all_tests.rs");
    assert_eq!(offences[0].line, 7);
    assert!(
        offences[0].description.contains("the function `helper`"),
        "got {}",
        offences[0].description
    );
}

#[test]
fn check_workspace_a_registry_holding_an_import_reports_it() {
    // Arrange & Act
    let offences = check(&[(
        "tests/all_tests.rs",
        "use alpha::One;\n\npub mod alpha_tests;\n",
    )]);

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].line, 5);
    assert!(
        offences[0].description.contains("use alpha::One;"),
        "got {}",
        offences[0].description
    );
}

// A declaration points at a file. A module with a body is code hiding in the one
// file a reader scans expecting a list.
#[test]
fn check_workspace_a_registry_holding_an_inline_module_reports_it() {
    // Arrange & Act
    let offences = check(&[("tests/all_tests.rs", "pub mod alpha_tests { }\n")]);

    // Assert
    assert_eq!(offences.len(), 1);
    assert!(
        offences[0]
            .description
            .contains("the inline module `alpha_tests`"),
        "got {}",
        offences[0].description
    );
}

// The defect this rule shipped with. Four strays produced four rows that were
// byte-identical, every one of them pointing at line 1 -- a report claiming a
// file has four problems while naming none of them.
#[test]
fn check_workspace_a_registry_holding_several_strays_reports_each_at_its_own_line() {
    // Arrange
    let body =
        "use std::fmt;\n\nconst LIMIT: usize = 1;\n\nfn helper() {}\n\npub mod alpha_tests;\n";

    // Act
    let offences = check(&[("tests/all_tests.rs", body)]);

    // Assert
    assert_eq!(offences.len(), 3);
    assert_eq!(
        offences
            .iter()
            .map(|offence| offence.line)
            .collect::<Vec<usize>>(),
        [5, 7, 9]
    );
    assert_eq!(
        offences
            .iter()
            .filter(|offence| offence.description == offences[0].description)
            .count(),
        1
    );
}

#[test]
fn check_workspace_a_registry_of_nothing_but_declarations_reports_nothing() {
    // Arrange & Act
    let offences = check(&[
        ("tests/all_tests.rs", REGISTRY),
        ("tests/alpha_tests.rs", TEST_BODY),
        ("tests/beta_tests.rs", TEST_BODY),
    ]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_a_second_all_tests_below_the_top_reports_it() {
    // Arrange & Act
    let offences = check(&[
        ("tests/all_tests.rs", REGISTRY),
        ("tests/rules/mod.rs", REGISTRY),
        ("tests/rules/all_tests.rs", REGISTRY),
    ]);

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].file, "tests/rules/all_tests.rs");
}

// The source tree has its own shape and is not this rule's business.
#[test]
fn check_workspace_a_source_subfolder_without_a_mod_file_reports_nothing() {
    // Arrange & Act
    let offences = check(&[
        ("tests/all_tests.rs", REGISTRY),
        ("src/rules/header_rule.rs", "pub struct A;\n"),
    ]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_a_subfolder_without_a_mod_file_reports_it_missing() {
    // Arrange & Act
    let offences = check(&[
        ("tests/all_tests.rs", REGISTRY),
        ("tests/rules/alpha_tests.rs", TEST_BODY),
    ]);

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].file, "tests/rules/mod.rs");
}

// Without the door nothing below it is compiled. The files are still there and
// still look like tests, which is what makes this worth failing a build over.
#[test]
fn check_workspace_a_tests_folder_without_all_tests_reports_it_missing() {
    // Arrange & Act
    let offences = check(&[("tests/alpha_tests.rs", TEST_BODY)]);

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].rule, RULE);
    assert_eq!(offences[0].file, "tests/all_tests.rs");
}

// An all_tests.rs that is not at the top is not the door, so the folder still
// has none.
#[test]
fn check_workspace_an_all_tests_only_in_a_subfolder_reports_the_top_one_missing() {
    // Arrange & Act
    let found = descriptions(&[
        ("tests/rules/mod.rs", REGISTRY),
        ("tests/rules/all_tests.rs", REGISTRY),
    ]);

    // Assert
    assert!(found.contains("tests/all_tests.rs"), "got {found}");
}

// An empty registry is a door onto nothing, but it is still a door and the rule
// has no opinion on how many rooms are behind it.
#[test]
fn check_workspace_an_empty_registry_reports_nothing() {
    // Arrange & Act
    let offences = check(&[("tests/all_tests.rs", "")]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// An intermediate folder is a folder too, even when it holds nothing but the
// folder below it.
#[test]
fn check_workspace_an_intermediate_folder_without_a_mod_file_reports_it_missing() {
    // Arrange & Act
    let found = descriptions(&[
        ("tests/all_tests.rs", REGISTRY),
        ("tests/rules/deep/mod.rs", REGISTRY),
        ("tests/rules/deep/alpha_tests.rs", TEST_BODY),
    ]);

    // Assert
    assert!(found.contains("tests/rules/mod.rs"), "got {found}");
}

// Guiding principle 2: silence is never success. readable-source reports the
// file, but this rule's own answer was simply absent -- indistinguishable from
// a registry it had checked and found clean.
#[test]
fn check_workspace_of_an_unparseable_registry_says_it_was_not_checked() {
    // Arrange & Act
    let offences = check(&[
        (
            "tests/all_tests.rs",
            "pub mod deep;
",
        ),
        (
            "tests/deep/mod.rs",
            "pub mod broken
",
        ),
        ("tests/deep/alpha_tests.rs", TEST_BODY),
    ]);

    // Assert
    let skipped: Vec<&stern4rust::reporting::offence::Offence> = offences
        .iter()
        .filter(|offence| offence.description.contains("could not be parsed"))
        .collect();
    assert_eq!(skipped.len(), 1);
    assert_eq!(skipped[0].file, "tests/deep/mod.rs");
    assert_eq!(
        skipped[0].correction,
        "correct the syntax error readable-source reports, so this registry can be checked"
    );
}

#[test]
fn check_workspace_reports_every_missing_registry_rather_than_only_the_first() {
    // Arrange & Act
    let offences = check(&[
        ("tests/alpha/alpha_tests.rs", TEST_BODY),
        ("tests/beta/beta_tests.rs", TEST_BODY),
    ]);

    // Assert
    assert_eq!(offences.len(), 3);
}

#[test]
fn name_is_the_kebab_case_rule_name_used_in_the_report() {
    // Arrange & Act
    let name = TestsLayoutRule::new().name();

    // Assert
    assert_eq!(name, RULE);
}
