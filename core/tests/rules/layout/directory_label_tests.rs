// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use stern4rust::rules::layout::directory_label::DirectoryLabel;
use stern4rust::source_path::SourcePath;

#[test]
fn of_a_nested_directory_writes_it_with_forward_slashes() {
    // Arrange & Act
    let label = DirectoryLabel::of(&SourcePath::new(String::from("src\\rules\\layout")));

    // Assert
    assert_eq!(label, "src/rules/layout");
}

// The root is no name at all, or a dot; either way a reader needs it said.
#[test]
fn of_the_package_root_names_it() {
    // Arrange & Act
    let labels = (
        DirectoryLabel::of(&SourcePath::new(String::from(""))),
        DirectoryLabel::of(&SourcePath::new(String::from("."))),
    );

    // Assert
    assert_eq!(
        labels,
        (
            String::from("the package root"),
            String::from("the package root")
        )
    );
}
