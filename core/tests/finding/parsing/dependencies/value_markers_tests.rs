// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Which traits say only how a value is copied, compared or printed: the value
// traits, and every marker trait the files declare -- no items, and every
// supertrait a value trait or another such marker.

use stern4rust::finding::parsing::dependencies::value_markers::ValueMarkers;
use stern4rust::source_file::SourceFile;

fn markers(files: &[(&str, &str)]) -> ValueMarkers {
    let sources: Vec<SourceFile> = files
        .iter()
        .map(|(path, contents)| SourceFile::new(path, contents))
        .collect();
    ValueMarkers::of(&sources)
}

#[test]
fn is_value_of_a_marker_over_a_behaviour_trait_is_false() {
    // Arrange
    let declared = markers(&[(
        "src/tagged.rs",
        "pub trait Store {\n    fn get(&self) -> u8;\n}\n\npub trait Tagged: Store + Clone {}\n",
    )]);

    // Act
    let value = declared.is_value("Tagged");

    // Assert
    assert!(!value);
}

#[test]
fn is_value_of_a_marker_over_another_marker_declared_elsewhere_is_true() {
    // Arrange
    let declared = markers(&[
        ("src/node_id.rs", "pub trait NodeId: PeerId {}\n"),
        ("src/peer_id.rs", "pub trait PeerId: Copy + Eq {}\n"),
    ]);

    // Act
    let value = declared.is_value("NodeId");

    // Assert
    assert!(value);
}

#[test]
fn is_value_of_a_marker_over_value_traits_is_true() {
    // Arrange
    let declared = markers(&[(
        "src/peer_id.rs",
        "use core::fmt::Debug;\n\npub trait PeerId: Copy + Eq + Debug + Send {}\n\nimpl PeerId for u64 {}\n",
    )]);

    // Act
    let value = declared.is_value("PeerId");

    // Assert
    assert!(value);
}

#[test]
fn is_value_of_a_trait_declaring_a_method_is_false() {
    // Arrange
    let declared = markers(&[(
        "src/reporter.rs",
        "pub trait Reporter: Send {\n    fn report(&self, line: &str);\n}\n",
    )]);

    // Act
    let value = declared.is_value("Reporter");

    // Assert
    assert!(!value);
}

#[test]
fn is_value_of_a_value_trait_is_true_and_of_a_trait_no_file_declares_false() {
    // Arrange
    let declared = markers(&[]);

    // Act
    let values = (declared.is_value("Clone"), declared.is_value("Reporter"));

    // Assert
    assert_eq!(values, (true, false));
}
