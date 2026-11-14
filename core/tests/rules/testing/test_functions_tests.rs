// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use stern4rust::rules::testing::test_functions::TestFunctions;
use stern4rust::source_file::SourceFile;

fn names(path: &str, contents: &str) -> Vec<String> {
    TestFunctions::of(&SourceFile::new(path, contents))
        .iter()
        .map(|function| function.sig.ident.to_string())
        .collect()
}

// Each test judged in turn, every finding kept in order; a helper is judged
// by no one.
#[test]
fn judged_of_a_test_file_gathers_what_each_test_was_found_to_break() {
    // Arrange
    let file = SourceFile::new(
        "tests/window_tests.rs",
        "fn window() {}\n\n#[test]\nfn opens() {}\n\n#[test]\nfn closes() {}\n",
    );

    // Act
    let found = TestFunctions::judged(&file, |function| {
        let name = function.sig.ident.to_string();
        [name.clone(), name]
    });

    // Assert
    assert_eq!(found, ["opens", "opens", "closes", "closes"]);
}

#[test]
fn of_a_file_that_does_not_parse_returns_none() {
    // Arrange & Act
    let found = names("tests/window_tests.rs", "#[test]\nfn open(");

    // Assert
    assert!(found.is_empty(), "{found:?}");
}

// A registry lists test files and a source file is no test file: neither is
// read for tests, whatever it holds.
#[test]
fn of_a_registry_or_a_source_file_returns_none() {
    // Arrange
    let body = "#[test]\nfn opens() {}\n";

    // Act
    let found = (
        names("tests/all_tests.rs", body),
        names("tests/rules/mod.rs", body),
        names("src/window.rs", body),
    );

    // Assert
    assert_eq!(found, (Vec::new(), Vec::new(), Vec::new()));
}

#[test]
fn of_a_test_file_returns_its_test_functions_and_no_helper() {
    // Arrange & Act
    let found = names(
        "tests/window_tests.rs",
        "fn window() {}\n\n#[test]\nfn opens() {}\n\n#[test]\nfn closes() {}\n",
    );

    // Assert
    assert_eq!(found, ["opens", "closes"]);
}
