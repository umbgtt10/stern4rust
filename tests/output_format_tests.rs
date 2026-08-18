// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Which of the two reports a run emits.
//
// The table stays the default. A tool that changed its human output the moment
// it grew a machine one would break every terminal and every gate script that
// already reads it, to serve a consumer that has to opt in anyway.

use clap::ValueEnum;
use stern4rust::output_format::OutputFormat;

#[test]
fn default_is_the_table_a_human_reads() {
    // Arrange & Act
    let format = OutputFormat::default();

    // Assert
    assert_eq!(format, OutputFormat::Text);
}

#[test]
fn from_str_of_an_unknown_name_is_rejected() {
    // Arrange & Act
    let format = OutputFormat::from_str("yaml", true);

    // Assert
    assert!(format.is_err());
}

#[test]
fn from_str_of_json_is_the_machine_readable_document() {
    // Arrange & Act
    let format = OutputFormat::from_str("json", true);

    // Assert
    assert_eq!(format, Ok(OutputFormat::Json));
}

#[test]
fn from_str_of_text_is_the_table() {
    // Arrange & Act
    let format = OutputFormat::from_str("text", true);

    // Assert
    assert_eq!(format, Ok(OutputFormat::Text));
}
