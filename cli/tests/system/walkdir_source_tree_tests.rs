// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Finding the files a rule will judge. "Every .rs file" is the rule's own
// wording, so tests/ counts as much as src/ -- a test file without the header is
// exactly as wrong as a source file without it.
//
// target/ is the one exclusion that matters: it holds generated code nobody
// wrote, and judging it would bury every real finding under thousands of rows.

use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use stern4rust::source_path::SourcePath;
use stern4rust::traits::source_tree::SourceTree;
use stern4rust_cli::system::portable_path::PortablePath;
use stern4rust_cli::system::walkdir_source_tree::WalkdirSourceTree;

fn names(root: &Path) -> Vec<String> {
    let root = PortablePath::of(root).expect("a UTF-8 temporary directory");
    WalkdirSourceTree::new()
        .rust_files(&root)
        .expect("listable")
        .iter()
        .map(|path| {
            path.strip_prefix(&root)
                .unwrap_or(path.as_str())
                .replace('\\', "/")
        })
        .collect()
}

// A file name that is not UTF-8 can only be made on Linux: Windows names are
// UTF-16, and macOS's file system refuses a name that is not valid UTF-8.
#[cfg(target_os = "linux")]
mod not_utf8 {
    use super::portable;
    use super::temp_root;
    use std::ffi::OsStr;
    use std::fs;
    use std::os::unix::ffi::OsStrExt;
    use stern4rust::traits::source_tree::SourceTree;
    use stern4rust_cli::system::walkdir_source_tree::WalkdirSourceTree;

    #[test]
    fn rust_files_with_a_file_name_that_is_not_utf8_fails_naming_it() {
        // Arrange
        let root = temp_root("not_utf8");
        fs::write(root.join(OsStr::from_bytes(b"bad\xff.rs")), "pub fn f() {}").expect("write");

        // Act
        let files = WalkdirSourceTree::new().rust_files(&portable(&root));

        // Assert
        let failure = files.expect_err("not UTF-8");
        assert!(failure.to_string().ends_with("bad\u{fffd}.rs"), "{failure}");
        assert_eq!(failure.cause.to_string(), "the path is not valid UTF-8");
    }
}

fn portable(path: &Path) -> SourcePath {
    PortablePath::of(path).expect("a UTF-8 temporary directory")
}

fn temp_root(name: &str) -> PathBuf {
    let root = env::temp_dir().join(format!("stern4rust_walker_{name}"));
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
fn contains_a_file_on_disk_is_true() {
    // Arrange
    let root = temp_root("contains_file");
    write(&root, "stern4rust-baseline.json");

    // Act
    let found =
        WalkdirSourceTree::new().contains(&portable(&root.join("stern4rust-baseline.json")));

    // Assert
    assert!(found);
}

#[test]
fn contains_a_path_that_does_not_exist_is_false() {
    // Arrange
    let root = temp_root("contains_nothing");

    // Act
    let found =
        WalkdirSourceTree::new().contains(&portable(&root.join("stern4rust-baseline.json")));

    // Assert
    assert!(!found);
}

#[test]
fn read_source_of_a_file_on_disk_returns_its_contents() {
    // Arrange
    let root = temp_root("read");
    write(&root, "src/lib.rs");

    // Act
    let source = WalkdirSourceTree::new().read_source(&portable(&root.join("src").join("lib.rs")));

    // Assert
    assert_eq!(source.expect("readable"), "// file");
}

#[test]
fn read_source_of_a_file_that_does_not_exist_names_it() {
    // Arrange
    let root = temp_root("read_absent");
    let path = portable(&root.join("src").join("lib.rs"));

    // Act
    let source = WalkdirSourceTree::new().read_source(&path);

    // Assert
    assert_eq!(source.expect_err("absent").path, path);
}

#[test]
fn rust_files_descends_into_nested_directories() {
    // Arrange
    let root = temp_root("nested");
    write(&root, "src/rules/deep/subject.rs");

    // Act
    let found = names(&root);

    // Assert
    assert_eq!(found, ["src/rules/deep/subject.rs"]);
}

// A fixture crate nested under tests/ is judged like anything else. The walker
// once skipped it, which was the wrong place to solve the problem: sample code a
// tool analyses does not belong inside the package that ships, and a linter
// quietly declining to look at part of a tree is the silence this tool refuses.
#[test]
fn rust_files_finds_a_fixture_package_nested_under_tests() {
    // Arrange
    let root = temp_root("fixture_package");
    write(&root, "Cargo.toml");
    write(&root, "tests/a_tests.rs");
    write(&root, "tests/fixtures/sample/Cargo.toml");
    write(&root, "tests/fixtures/sample/src/lib.rs");

    // Act
    let found = names(&root);

    // Assert
    assert_eq!(
        found,
        ["tests/a_tests.rs", "tests/fixtures/sample/src/lib.rs"]
    );
}

// A manifest is a fact about cargo, not about whose conventions apply. Skipping
// on sight let a whole tree go unreported with nothing in the report saying so.
#[test]
fn rust_files_finds_a_nested_package_holding_its_own_manifest() {
    // Arrange
    let root = temp_root("nested_package");
    write(&root, "Cargo.toml");
    write(&root, "src/a.rs");
    write(&root, "vendored/other/Cargo.toml");
    write(&root, "vendored/other/src/b.rs");

    // Act
    let found = names(&root);

    // Assert
    assert_eq!(found, ["src/a.rs", "vendored/other/src/b.rs"]);
}

#[test]
fn rust_files_finds_a_rust_file_at_the_root() {
    // Arrange
    let root = temp_root("root_file");
    write(&root, "build.rs");

    // Act
    let found = names(&root);

    // Assert
    assert_eq!(found, ["build.rs"]);
}

#[test]
fn rust_files_finds_rust_files_under_src_and_tests_alike() {
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
fn rust_files_ignores_files_that_are_not_rust() {
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

// A manifest is not a signal to the walker, at the root or anywhere else. This
// once guarded an exemption from a nested-package skip; it now pins that no such
// skip exists to need exempting.
#[test]
fn rust_files_keeps_the_root_package_even_though_it_holds_a_manifest() {
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
fn rust_files_of_a_directory_that_does_not_exist_returns_nothing() {
    // Arrange
    let root = env::temp_dir().join("stern4rust_walker_absent");
    let _ = fs::remove_dir_all(&root);

    // Act
    let found = WalkdirSourceTree::new()
        .rust_files(&portable(&root))
        .expect("listable");

    // Assert
    assert!(found.is_empty());
}

#[test]
fn rust_files_of_a_directory_without_rust_files_returns_nothing() {
    // Arrange
    let root = temp_root("empty");
    write(&root, "docs/notes.md");

    // Act
    let found = names(&root);

    // Assert
    assert!(found.is_empty());
}

#[test]
fn rust_files_returns_paths_in_a_stable_order() {
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

// Generated code nobody wrote. Judging it would drown every real finding.
#[test]
fn rust_files_skips_the_target_directory() {
    // Arrange
    let root = temp_root("target");
    write(&root, "src/a.rs");
    write(&root, "target/debug/build/generated.rs");

    // Act
    let found = names(&root);

    // Assert
    assert_eq!(found, ["src/a.rs"]);
}

#[test]
fn write_source_into_a_directory_that_does_not_exist_names_the_file() {
    // Arrange
    let root = temp_root("write_nowhere");
    let path = portable(&root.join("gone").join("a_tests.rs"));

    // Act
    let written = WalkdirSourceTree::new().write_source(&path, "// fixed");

    // Assert
    assert_eq!(written.expect_err("no directory").path, path);
}

#[test]
fn write_source_replaces_what_the_file_held() {
    // Arrange
    let root = temp_root("write");
    write(&root, "tests/a_tests.rs");
    let path = root.join("tests").join("a_tests.rs");

    // Act
    WalkdirSourceTree::new()
        .write_source(&portable(&path), "// fixed")
        .expect("writable");

    // Assert
    assert_eq!(fs::read_to_string(&path).expect("read back"), "// fixed");
}
