// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Reading a file the walker found, and turning a failure into a finding.
//
// A file that cannot be read used to abort the whole run, which meant one
// unreadable file hid every offence already found in every other file. That is
// the wrong trade: a bad manifest is a could-not-run condition because nothing
// can be enumerated without it, but a single unreadable file is a fact about the
// tree and the rest of the tree is still worth reporting on.

use std::path::Path;
use stern4rust::source_reader::SourceReader;

const RULE: &str = "readable-source";

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn read_of_a_file_that_does_not_exist_names_the_readable_source_rule() {
    // Arrange & Act
    let found = SourceReader::read(root(), &root().join("src/there_is_no_such_file.rs"));

    // Assert
    let offence = found.expect_err("expected an offence");
    assert_eq!(offence.rule, RULE);
    assert!(
        offence.description.contains("could not be read"),
        "got {}",
        offence.description
    );
}

#[test]
fn read_of_a_file_that_does_not_exist_returns_an_offence() {
    // Arrange & Act
    let found = SourceReader::read(root(), &root().join("src/there_is_no_such_file.rs"));

    // Assert
    assert!(found.is_err());
}

#[test]
fn read_of_a_readable_file_reports_the_path_relative_to_the_root() {
    // Arrange & Act
    let found = SourceReader::read(root(), &root().join("src/lib.rs"));

    // Assert
    assert_eq!(found.expect("readable").relative_path(), "src/lib.rs");
}

#[test]
fn read_of_a_readable_file_returns_its_contents() {
    // Arrange & Act
    let found = SourceReader::read(root(), &root().join("src/lib.rs"));

    // Assert
    assert!(
        found
            .expect("readable")
            .contents()
            .contains("pub mod rule;")
    );
}
