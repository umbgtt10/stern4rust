// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Finding the files a rule will judge. "Every .rs file" is the rule's own
// wording, so tests/ counts as much as src/ -- a test file without the header is
// exactly as wrong as a source file without it.
//
// target/ is the one exclusion that matters: it holds generated code nobody
// wrote, and judging it would bury every real finding under thousands of rows.

use std::fs;
use std::path::Path;
use std::path::PathBuf;
use stern4rust::source_walker::SourceWalker;

fn names(root: &Path) -> Vec<String> {
    SourceWalker::walk(root)
        .iter()
        .map(|path| {
            path.strip_prefix(root)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect()
}

fn temp_root(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("stern4rust_walker_{name}"));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("create the temp root");
    root
}

fn write(root: &Path, relative: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().expect("a parent")).expect("create parents");
    fs::write(path, "// file").expect("write the file");
}

#[test]
fn walk_descends_into_nested_directories() {
    // Arrange
    let root = temp_root("nested");
    write(&root, "src/rules/deep/subject.rs");

    // Act
    let found = names(&root);

    // Assert
    assert_eq!(found, ["src/rules/deep/subject.rs"]);
}

#[test]
fn walk_finds_a_rust_file_at_the_root() {
    // Arrange
    let root = temp_root("root_file");
    write(&root, "build.rs");

    // Act
    let found = names(&root);

    // Assert
    assert_eq!(found, ["build.rs"]);
}

#[test]
fn walk_finds_rust_files_under_src_and_tests_alike() {
    // Arrange
    let root = temp_root("src_and_tests");
    write(&root, "src/a.rs");
    write(&root, "tests/a_tests.rs");

    // Act
    let found = names(&root);

    // Assert
    assert_eq!(found, ["src/a.rs", "tests/a_tests.rs"]);
}

#[test]
fn walk_ignores_files_that_are_not_rust() {
    // Arrange
    let root = temp_root("non_rust");
    write(&root, "src/a.rs");
    write(&root, "README.md");
    write(&root, "Cargo.toml");

    // Act
    let found = names(&root);

    // Assert
    assert_eq!(found, ["src/a.rs"]);
}

// The package being walked holds a manifest by definition, so the rule that
// skips nested packages has to exempt the one it started from -- otherwise the
// walk skips everything and every run reports a clean tree.
#[test]
fn walk_keeps_the_root_package_even_though_it_holds_a_manifest() {
    // Arrange
    let root = temp_root("root_manifest");
    write(&root, "Cargo.toml");
    write(&root, "src/a.rs");

    // Act
    let found = names(&root);

    // Assert
    assert_eq!(found, ["src/a.rs"]);
}

#[test]
fn walk_of_a_directory_that_does_not_exist_returns_nothing() {
    // Arrange
    let root = std::env::temp_dir().join("stern4rust_walker_absent");
    let _ = fs::remove_dir_all(&root);

    // Act
    let found = SourceWalker::walk(&root);

    // Assert
    assert!(found.is_empty());
}

#[test]
fn walk_of_a_directory_without_rust_files_returns_nothing() {
    // Arrange
    let root = temp_root("empty");
    write(&root, "docs/notes.md");

    // Act
    let found = names(&root);

    // Assert
    assert!(found.is_empty());
}

#[test]
fn walk_returns_paths_in_a_stable_order() {
    // Arrange
    let root = temp_root("order");
    write(&root, "src/z.rs");
    write(&root, "src/a.rs");
    write(&root, "src/m.rs");

    // Act
    let found = names(&root);

    // Assert
    assert_eq!(found, ["src/a.rs", "src/m.rs", "src/z.rs"]);
}

// The shape this was written for: crap4rust and grip4rust both keep sample
// crates under tests/fixtures/, and judging them produced 154 offences against
// code that is input data rather than the repository's own.
#[test]
fn walk_skips_a_fixture_package_under_tests() {
    // Arrange
    let root = temp_root("fixture_package");
    write(&root, "Cargo.toml");
    write(&root, "tests/a_tests.rs");
    write(&root, "tests/fixtures/sample/Cargo.toml");
    write(&root, "tests/fixtures/sample/src/lib.rs");
    write(&root, "tests/fixtures/sample/tests/sample_tests.rs");

    // Act
    let found = names(&root);

    // Assert
    assert_eq!(found, ["tests/a_tests.rs"]);
}

// A directory with its own manifest is a different package. Its files are that
// package's to answer for, under whatever rules it has chosen, and cargo would
// not compile them as part of this one either.
#[test]
fn walk_skips_a_nested_package_holding_its_own_manifest() {
    // Arrange
    let root = temp_root("nested_package");
    write(&root, "Cargo.toml");
    write(&root, "src/a.rs");
    write(&root, "vendored/other/Cargo.toml");
    write(&root, "vendored/other/src/b.rs");

    // Act
    let found = names(&root);

    // Assert
    assert_eq!(found, ["src/a.rs"]);
}

// Generated code nobody wrote. Judging it would drown every real finding.
#[test]
fn walk_skips_the_target_directory() {
    // Arrange
    let root = temp_root("target");
    write(&root, "src/a.rs");
    write(&root, "target/debug/build/generated.rs");

    // Act
    let found = names(&root);

    // Assert
    assert_eq!(found, ["src/a.rs"]);
}
