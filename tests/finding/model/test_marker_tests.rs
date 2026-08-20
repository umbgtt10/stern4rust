// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// One AAA marker comment, read as the phases it names.
//
// The boundary is what stops `// Actually` from being an Act, and the trailing
// prose is what lets a marker explain itself -- both observed in the family
// before this was written.

use stern4rust::finding::model::test_marker::MarkerPhase;
use stern4rust::finding::model::test_marker::TestMarker;

#[test]
fn parse_of_a_fully_merged_marker_names_all_three_phases() {
    // Arrange & Act
    let marker = TestMarker::parse("    // Arrange & Act & Assert", 3);

    // Assert
    assert_eq!(
        marker.map(|found| found.phases),
        Some(vec![
            MarkerPhase::Arrange,
            MarkerPhase::Act,
            MarkerPhase::Assert
        ])
    );
}

#[test]
fn parse_of_a_marker_keeps_the_line_it_was_found_on() {
    // Arrange & Act
    let marker = TestMarker::parse("    // Assert", 42);

    // Assert
    assert_eq!(marker.map(|found| found.line), Some(42));
}

#[test]
fn parse_of_a_marker_labels_it_as_written() {
    // Arrange & Act
    let marker = TestMarker::parse("    // Arrange & Act", 1);

    // Assert
    assert_eq!(
        marker.map(|found| found.label),
        Some("// Arrange & Act".to_string())
    );
}

#[test]
fn parse_of_a_marker_with_trailing_prose_names_its_phase() {
    // Arrange & Act
    let marker = TestMarker::parse("    // Act: heal the partition", 1);

    // Assert
    assert_eq!(
        marker.map(|found| found.phases),
        Some(vec![MarkerPhase::Act])
    );
}

#[test]
fn parse_of_a_merged_act_and_assert_names_both_phases() {
    // Arrange & Act
    let marker = TestMarker::parse("    // Act & Assert", 1);

    // Assert
    assert_eq!(
        marker.map(|found| found.phases),
        Some(vec![MarkerPhase::Act, MarkerPhase::Assert])
    );
}

#[test]
fn parse_of_a_plain_arrange_names_one_phase() {
    // Arrange & Act
    let marker = TestMarker::parse("    // Arrange", 1);

    // Assert
    assert_eq!(
        marker.map(|found| found.phases),
        Some(vec![MarkerPhase::Arrange])
    );
}

// The word boundary. Without it every explanatory comment starting `Act` is a
// marker, and a test's sequence reads as nonsense.
#[test]
fn parse_of_a_word_beginning_with_a_phase_name_is_not_a_marker() {
    // Arrange & Act
    let marker = TestMarker::parse("    // Actually this needs explaining", 1);

    // Assert
    assert!(marker.is_none());
}

#[test]
fn parse_of_an_arrange_with_dashed_prose_names_one_phase() {
    // Arrange & Act
    let marker = TestMarker::parse("    // Arrange -- four nodes, one split", 1);

    // Assert
    assert_eq!(
        marker.map(|found| found.phases),
        Some(vec![MarkerPhase::Arrange])
    );
}

#[test]
fn parse_of_an_ordinary_comment_is_not_a_marker() {
    // Arrange & Act
    let marker = TestMarker::parse("    // the queue starts empty", 1);

    // Assert
    assert!(marker.is_none());
}

#[test]
fn parse_of_code_that_is_not_a_comment_is_not_a_marker() {
    // Arrange & Act
    let marker = TestMarker::parse("    let arrange = 1;", 1);

    // Assert
    assert!(marker.is_none());
}
