// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A source file's name, and whether it names a type.
//
// A file is named after a type when its stem is the type's name, ignoring
// underscores and case, or the last words of it. The words in front are left to
// where the file sits: `loopback/src/driver.rs` holding `LoopbackDriver`. The
// match is on whole words, so `dex.rs` is not named after `Index`, and only the
// front may be left out: `protocol_observer.rs` claims more than `Observer`
// says.

use stern4rust::finding::model::file_stem::FileStem;

#[test]
fn names_a_stem_of_underscores_alone_is_false() {
    // Arrange & Act
    let named = FileStem::of("src/_.rs").names("Widget");

    // Assert
    assert!(!named);
}

#[test]
fn names_a_stem_that_is_part_of_a_word_is_false() {
    // Arrange & Act
    let named = FileStem::of("src/dex.rs").names("Index");

    // Assert
    assert!(!named);
}

#[test]
fn names_a_stem_that_is_the_last_words_of_the_type_is_true() {
    // Arrange & Act
    let named = FileStem::of("src/transport/listener.rs").names("UdpEiListener");

    // Assert
    assert!(named);
}

#[test]
fn names_a_stem_that_is_the_type_name_is_true() {
    // Arrange & Act
    let named = FileStem::of("src/udp_listener.rs").names("UdpListener");

    // Assert
    assert!(named);
}

#[test]
fn names_a_stem_that_leaves_out_the_last_word_is_false() {
    // Arrange & Act
    let named = FileStem::of("src/listener.rs").names("UdpListenerTask");

    // Assert
    assert!(!named);
}

// The match the proposal started from: the same letters, underscores aside.
#[test]
fn names_a_stem_that_spells_the_type_without_underscores_is_true() {
    // Arrange & Act
    let named = FileStem::of("src/walrecord.rs").names("WalRecord");

    // Assert
    assert!(named);
}

#[test]
fn names_a_stem_with_a_word_the_type_lacks_is_false() {
    // Arrange & Act
    let named = FileStem::of("src/protocol_observer.rs").names("Observer");

    // Assert
    assert!(!named);
}

#[test]
fn names_an_acronym_in_capitals_by_its_snake_case_word_is_true() {
    // Arrange & Act
    let named = FileStem::of("src/http_server.rs").names("HTTPServer");

    // Assert
    assert!(named);
}
