// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::cmp::Ordering;
use stern4rust::source_path::SourcePath;

fn path(text: &str) -> SourcePath {
    SourcePath::new(String::from(text))
}

#[test]
fn as_str_returns_the_text_it_was_given() {
    // Arrange
    let source = path("/p/src/lib.rs");

    // Act
    let text = source.as_str();

    // Assert
    assert_eq!(text, "/p/src/lib.rs");
}

#[test]
fn cmp_orders_paths_by_component_rather_than_by_byte() {
    // Arrange
    let nested = path("/a/b");
    let sibling = path("/a-b");

    // Act
    let order = nested.cmp(&sibling);

    // Assert
    assert_eq!(order, Ordering::Less);
}

#[test]
fn eq_with_a_trailing_separator_is_the_same_path() {
    // Arrange
    let plain = path("/p/src");
    let trailing = path("/p/src/");

    // Act
    let same = plain == trailing;

    // Assert
    assert!(same);
}

#[test]
fn extension_of_a_dotfile_returns_none() {
    // Arrange
    let source = path("/p/.rs");

    // Act
    let extension = source.extension();

    // Assert
    assert_eq!(extension, None);
}

#[test]
fn extension_of_a_file_without_one_returns_none() {
    // Arrange
    let source = path("/p/src/Makefile");

    // Act
    let extension = source.extension();

    // Assert
    assert_eq!(extension, None);
}

#[test]
fn extension_of_a_rust_file_returns_rs() {
    // Arrange
    let source = path("/p/src/lib.rs");

    // Act
    let extension = source.extension();

    // Assert
    assert_eq!(extension, Some("rs"));
}

#[test]
fn file_name_of_a_root_returns_none() {
    // Arrange
    let source = path("/");

    // Act
    let name = source.file_name();

    // Assert
    assert_eq!(name, None);
}

#[test]
fn file_stem_of_a_dotfile_returns_the_whole_name() {
    // Arrange
    let source = path("/p/src/.hidden");

    // Act
    let stem = source.file_stem();

    // Assert
    assert_eq!(stem, Some(".hidden"));
}

#[test]
fn file_stem_of_a_name_with_two_extensions_drops_only_the_last() {
    // Arrange
    let source = path("/p/src/data.rs.bak");

    // Act
    let stem = source.file_stem();

    // Assert
    assert_eq!(stem, Some("data.rs"));
}

#[test]
fn file_stem_of_a_root_returns_none() {
    // Arrange
    let source = path("/");

    // Act
    let stem = source.file_stem();

    // Assert
    assert_eq!(stem, None);
}

#[test]
fn join_of_an_absolute_name_replaces_the_path() {
    // Arrange & Act
    let joined = path("/p").join("/q");

    // Assert
    assert_eq!(joined.as_str(), "/q");
}

#[test]
fn join_onto_a_directory_adds_the_name_below_it() {
    // Arrange & Act
    let joined = path("/p").join("src");

    // Assert
    assert_eq!(joined, path("/p/src"));
}

#[test]
fn join_onto_a_path_ending_in_a_separator_adds_none() {
    // Arrange & Act
    let joined = path("/p/").join("src");

    // Assert
    assert_eq!(joined.as_str(), "/p/src");
}

#[test]
fn parent_of_a_file_returns_its_directory() {
    // Arrange
    let source = path("/p/src/lib.rs");

    // Act
    let parent = source.parent();

    // Assert
    assert_eq!(parent, Some(path("/p/src")));
}

#[test]
fn parent_of_a_name_alone_returns_the_empty_path() {
    // Arrange
    let source = path("Cargo.toml");

    // Act
    let parent = source.parent();

    // Assert
    assert_eq!(
        parent.map(|parent| String::from(parent.as_str())),
        Some(String::new())
    );
}

#[test]
fn parent_of_a_top_level_entry_returns_the_root() {
    // Arrange
    let source = path("/p");

    // Act
    let parent = source.parent();

    // Assert
    assert_eq!(
        parent.map(|parent| String::from(parent.as_str())),
        Some(String::from("/"))
    );
}

#[test]
fn parent_of_the_root_returns_none() {
    // Arrange
    let source = path("/");

    // Act
    let parent = source.parent();

    // Assert
    assert_eq!(parent, None);
}

#[test]
fn strip_prefix_of_a_sibling_sharing_the_prefix_text_returns_none() {
    // Arrange
    let base = path("/home/user/project");
    let source = path("/home/user/projectx/src/lib.rs");

    // Act
    let rest = source.strip_prefix(&base);

    // Assert
    assert_eq!(rest, None);
}

#[test]
fn strip_prefix_under_the_base_returns_the_rest_as_written() {
    // Arrange
    let base = path("/home/user/project");
    let source = path("/home/user/project//src/lib.rs");

    // Act
    let rest = source.strip_prefix(&base);

    // Assert
    assert_eq!(rest, Some("src/lib.rs"));
}

#[test]
fn strip_prefix_with_a_relative_base_under_an_absolute_path_returns_none() {
    // Arrange
    let base = path("home/user");
    let source = path("/home/user/lib.rs");

    // Act
    let rest = source.strip_prefix(&base);

    // Assert
    assert_eq!(rest, None);
}

#[test]
fn to_string_displays_the_text_it_was_given() {
    // Arrange
    let source = path("/p/src/lib.rs");

    // Act
    let text = source.to_string();

    // Assert
    assert_eq!(text, "/p/src/lib.rs");
}
