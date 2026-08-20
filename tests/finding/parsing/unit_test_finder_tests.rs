// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Finding tests, and the machinery of tests, in the production source tree.
//
// Three shapes are looked for. A function carrying a test attribute is the
// obvious one. `#[cfg(test)]` is the usual one, and it has to be recognised
// through a predicate rather than by matching the literal text, since
// `any(test, ...)` and `not(test)` gate on test just as effectively.
// `#[cfg_attr(test, ...)]` is the third, and only in that spelling: a type
// carrying a derive only under test means one thing to the tests and another to
// the shipped build. `#[cfg_attr(feature = "serde", ...)]` is ordinary library
// work and is left alone.
//
// A predicate mentioning `test` is found by scanning for an identifier, not a
// substring, so `#[cfg(feature = "test")]` is a feature named test and not a
// test gate.

use stern4rust::finding::model::unit_test_site::UnitTestSite;
use stern4rust::finding::parsing::unit_test_finder::UnitTestFinder;
use stern4rust::source_file::SourceFile;

fn corrections(body: &str) -> Vec<String> {
    sites(body)
        .into_iter()
        .map(|site| site.correction)
        .collect()
}

fn labels(body: &str) -> Vec<String> {
    sites(body).into_iter().map(|site| site.label).collect()
}

fn sites(body: &str) -> Vec<UnitTestSite> {
    UnitTestFinder::sites(&SourceFile::new("src/subject.rs", body)).expect("parses")
}

// The path a reader is being sent to, mirroring src/<path>.rs onto
// tests/<path>_tests.rs, so the correction names a file rather than a policy.
#[test]
fn sites_correction_names_the_mirrored_test_file() {
    // Arrange & Act
    let found = corrections("#[cfg(test)]\nmod tests {}\n");

    // Assert
    assert!(
        found[0].contains("tests/subject_tests.rs"),
        "got {}",
        found[0]
    );
}

#[test]
fn sites_names_a_cfg_test_module_with_its_identifier() {
    // Arrange & Act
    let found = labels("#[cfg(test)]\nmod tests {}\n");

    // Assert
    assert_eq!(found, ["the `#[cfg(test)]` module `tests`"]);
}

#[test]
fn sites_names_a_test_function_with_its_identifier() {
    // Arrange & Act
    let found = labels("#[test]\nfn checks_something() {}\n");

    // Assert
    assert_eq!(found, ["the test function `checks_something`"]);
}

// any(test, ...) gates on test just as effectively as test alone.
#[test]
fn sites_of_a_cfg_any_test_predicate_reports_it() {
    // Arrange & Act
    let found = sites("#[cfg(any(test, feature = \"extra\"))]\nmod tests {}\n");

    // Assert
    assert_eq!(found.len(), 1);
}

// Applying a derive behind a feature is ordinary library work -- serde is the
// obvious case -- and gates on something the shipped build can also select.
#[test]
fn sites_of_a_cfg_attr_on_a_feature_reports_nothing() {
    // Arrange & Act
    let found = sites("#[cfg_attr(feature = \"serde\", derive(Serialize))]\npub struct A;\n");

    // Assert
    assert!(found.is_empty(), "expected none, got {found:?}");
}

// A type carrying a derive only under test is a type that means one thing to
// the tests and another to the shipped build.
#[test]
fn sites_of_a_cfg_attr_on_test_reports_it() {
    // Arrange & Act
    let found = sites("#[cfg_attr(test, derive(Debug))]\npub struct A;\n");

    // Assert
    assert_eq!(found.len(), 1);
    assert!(
        found[0].label.contains("`#[cfg_attr(test, ...)]`"),
        "got {}",
        found[0].label
    );
}

#[test]
fn sites_of_a_cfg_feature_module_reports_nothing() {
    // Arrange & Act
    let found = sites("#[cfg(feature = \"extra\")]\nmod extras {}\n");

    // Assert
    assert!(found.is_empty(), "expected none, got {found:?}");
}

// A feature happening to be named test is a feature, not a test gate. The
// predicate is scanned for an identifier rather than for a substring.
#[test]
fn sites_of_a_cfg_feature_named_test_reports_nothing() {
    // Arrange & Act
    let found = sites("#[cfg(feature = \"test\")]\nmod extras {}\n");

    // Assert
    assert!(found.is_empty(), "expected none, got {found:?}");
}

// A file-based declaration hides the test code in a sibling file, where the
// gate is not repeated. The declaration is the only thing there is to catch.
#[test]
fn sites_of_a_cfg_test_module_declaration_reports_it() {
    // Arrange & Act
    let found = sites("#[cfg(test)]\nmod tests;\n");

    // Assert
    assert_eq!(found.len(), 1);
}

#[test]
fn sites_of_a_cfg_test_module_reports_it() {
    // Arrange & Act
    let found = sites("#[cfg(test)]\nmod tests {\n    #[test]\n    fn a() {}\n}\n");

    // Assert
    assert_eq!(found.len(), 1);
}

// Nesting is where a gate is easiest to miss by eye, so the walk descends into
// inline modules rather than looking only at the top level.
#[test]
fn sites_of_a_cfg_test_nested_in_an_inline_module_reports_it() {
    // Arrange & Act
    let found = sites("pub mod inner {\n    #[cfg(test)]\n    mod tests {}\n}\n");

    // Assert
    assert_eq!(found.len(), 1);
}

// rustc will say so far more clearly, and readable-source reports the file.
#[test]
fn sites_of_a_file_that_does_not_parse_returns_nothing() {
    // Arrange & Act
    let found = UnitTestFinder::sites(&SourceFile::new("src/a.rs", "mod broken {\n"));

    // Assert
    assert!(found.is_none());
}

#[test]
fn sites_of_a_plain_source_file_reports_nothing() {
    // Arrange & Act
    let found =
        sites("pub struct A;\n\nimpl A {\n    pub fn new() -> Self {\n        Self\n    }\n}\n");

    // Assert
    assert!(found.is_empty(), "expected none, got {found:?}");
}

#[test]
fn sites_of_a_qualified_test_function_reports_it() {
    // Arrange & Act
    let found = sites("#[tokio::test]\nasync fn checks_something() {}\n");

    // Assert
    assert_eq!(found.len(), 1);
}

#[test]
fn sites_of_an_empty_file_returns_nothing() {
    // Arrange & Act
    let found = sites("");

    // Assert
    assert!(found.is_empty(), "expected none, got {found:?}");
}

#[test]
fn sites_reports_every_offending_item_rather_than_only_the_first() {
    // Arrange & Act
    let found = sites("#[cfg(test)]\nmod tests {}\n\n#[test]\nfn alpha() {}\n");

    // Assert
    assert_eq!(found.len(), 2);
}

#[test]
fn sites_reports_the_line_the_item_starts_on() {
    // Arrange & Act
    let found = sites("pub struct A;\n\n#[cfg(test)]\nmod tests {}\n");

    // Assert
    assert_eq!(found[0].line, 3);
}
