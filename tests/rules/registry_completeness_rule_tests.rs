// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A registry declares every file beside it, so nothing goes uncompiled.
//
// Only one direction is checked, and the reason is measurable: `pub mod x;`
// with no `x.rs` is a compile error, so rustc reports it. An orphan `x.rs` that
// nothing declares produces no error and no warning. Silence is the failure, so
// silence is what this looks for.

use stern4rust::reporting::offence::Offence;
use stern4rust::rule::Rule;
use stern4rust::rules::registry_completeness_rule::RegistryCompletenessRule;
use stern4rust::source_file::SourceFile;

fn check(files: &[(&str, &str)]) -> Vec<Offence> {
    let sources: Vec<SourceFile> = files
        .iter()
        .map(|(path, contents)| SourceFile::new(path, contents))
        .collect();
    RegistryCompletenessRule::new().check_workspace(&sources)
}

#[test]
fn check_workspace_of_a_complete_registry_reports_nothing() {
    // Arrange & Act
    let found = check(&[
        ("src/lib.rs", "pub mod widget;\n"),
        ("src/widget.rs", "pub struct Widget;\n"),
    ]);

    // Assert
    assert!(found.is_empty(), "expected none, got {found:?}");
}

// A directory with no registry at all is nobody's index -- src/bin/, or the
// package root holding build.rs.
#[test]
fn check_workspace_of_a_directory_with_no_registry_reports_nothing() {
    // Arrange & Act
    let found = check(&[("build.rs", "fn main() {}\n")]);

    // Assert
    assert!(found.is_empty(), "expected none, got {found:?}");
}

// A file declared only from main.rs is reached. Reporting it would send the
// reader to add a line that is already there, in a different file.
#[test]
fn check_workspace_of_a_file_declared_only_by_main_reports_nothing() {
    // Arrange & Act
    let found = check(&[
        ("src/lib.rs", "pub mod widget;\n"),
        ("src/main.rs", "mod helper;\nfn main() {}\n"),
        ("src/widget.rs", "pub struct Widget;\n"),
        ("src/helper.rs", "pub struct Helper;\n"),
    ]);

    // Assert
    assert!(found.is_empty(), "expected none, got {found:?}");
}

// A private `mod name;` compiles the file just as well, and being compiled is
// the whole concern. module-registry is the rule that wants `pub`.
#[test]
fn check_workspace_of_a_privately_declared_file_reports_nothing() {
    // Arrange & Act
    let found = check(&[
        ("src/lib.rs", "mod widget;\n"),
        ("src/widget.rs", "pub struct Widget;\n"),
    ]);

    // Assert
    assert!(found.is_empty(), "expected none, got {found:?}");
}

// A subfolder without a registry cannot be declared yet, and tests-layout is
// the rule that says so. Reporting it here would instruct the reader to declare
// something that would not compile.
#[test]
fn check_workspace_of_a_subfolder_without_a_registry_reports_nothing() {
    // Arrange & Act
    let found = check(&[
        ("src/lib.rs", "pub mod widget;\n"),
        ("src/widget.rs", "pub struct Widget;\n"),
        ("src/loose/alpha.rs", "pub struct Alpha;\n"),
    ]);

    // Assert
    assert!(found.is_empty(), "expected none, got {found:?}");
}

// An inline module declares no file, so it cannot be what reaches one.
#[test]
fn check_workspace_of_an_inline_module_does_not_count_as_a_declaration() {
    // Arrange & Act
    let found = check(&[
        ("src/lib.rs", "pub mod widget { pub struct W; }\n"),
        ("src/widget.rs", "pub struct Widget;\n"),
    ]);

    // Assert
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].subject.as_deref(), Some("widget"));
}

// The failure this rule exists for: the file is fine, the registry is valid,
// and nothing compiles the file.
#[test]
fn check_workspace_of_an_undeclared_file_reports_it_against_the_registry() {
    // Arrange & Act
    let found = check(&[
        ("src/lib.rs", "pub mod widget;\n"),
        ("src/widget.rs", "pub struct Widget;\n"),
        ("src/orphan.rs", "pub struct Orphan;\n"),
    ]);

    // Assert
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].file, "src/lib.rs");
    assert_eq!(found[0].rule, "registry-completeness");
    assert_eq!(found[0].subject.as_deref(), Some("orphan"));
    assert_eq!(found[0].correction, "add `pub mod orphan;` to src/lib.rs");
}

// A subfolder with a registry of its own is a module and has to be declared.
#[test]
fn check_workspace_of_an_undeclared_subfolder_reports_it() {
    // Arrange & Act
    let found = check(&[
        ("src/lib.rs", "pub mod widget;\n"),
        ("src/widget.rs", "pub struct Widget;\n"),
        ("src/rules/mod.rs", "pub mod alpha;\n"),
        ("src/rules/alpha.rs", "pub struct Alpha;\n"),
    ]);

    // Assert
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].file, "src/lib.rs");
    assert_eq!(found[0].subject.as_deref(), Some("rules"));
}

// The tests tree is the shape the gap was first noticed in.
#[test]
fn check_workspace_of_an_undeclared_test_file_reports_it() {
    // Arrange & Act
    let found = check(&[
        ("tests/all_tests.rs", "pub mod alpha_tests;\n"),
        ("tests/alpha_tests.rs", "#[test]\nfn t() {}\n"),
        ("tests/beta_tests.rs", "#[test]\nfn t() {}\n"),
    ]);

    // Assert
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].file, "tests/all_tests.rs");
    assert_eq!(found[0].subject.as_deref(), Some("beta_tests"));
}

// One real offence must not become a page of wrong ones. readable-source
// reports the unparseable registry.
#[test]
fn check_workspace_of_an_unparseable_registry_reports_nothing() {
    // Arrange & Act
    let found = check(&[
        ("src/lib.rs", "pub mod ( broken\n"),
        ("src/widget.rs", "pub struct Widget;\n"),
        ("src/orphan.rs", "pub struct Orphan;\n"),
    ]);

    // Assert
    assert!(found.is_empty(), "expected none, got {found:?}");
}

#[test]
fn name_is_registry_completeness() {
    // Arrange & Act
    let name = RegistryCompletenessRule::new().name();

    // Assert
    assert_eq!(name, "registry-completeness");
}
