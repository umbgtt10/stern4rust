// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Why a run could not judge anything, in the words a run has always used. A
// failure wrapping another says that one's words and passes its reason on.

use crate::support::in_memory_source_tree::NotHeld;
use std::error::Error;
use stern4rust::run_failure::RunFailure;
use stern4rust::settings::config_failure::ConfigFailure;
use stern4rust::source_path::SourcePath;
use stern4rust::source_read_failure::SourceReadFailure;

fn path(text: &str) -> SourcePath {
    SourcePath::new(text.to_string())
}

#[test]
fn source_of_a_failure_with_no_reason_beneath_it_is_none() {
    // Arrange
    let failure = RunFailure::NoRules;

    // Act
    let source = failure.source();

    // Assert
    assert!(source.is_none());
}

#[test]
fn source_of_a_wrapped_failure_is_its_reason() {
    // Arrange
    let failure = RunFailure::Config(ConfigFailure::Invalid {
        path: path("stern4rust.toml"),
        cause: Box::new(NotHeld),
    });

    // Act
    let source = failure.source().map(ToString::to_string);

    // Assert
    assert_eq!(source, Some(String::from("not held")));
}

#[test]
fn source_of_an_unreadable_header_is_why_it_could_not_be_read() {
    // Arrange
    let failure = RunFailure::HeaderUnreadable(SourceReadFailure {
        path: path("docs/header.txt"),
        cause: Box::new(NotHeld),
    });

    // Act
    let source = failure.source().map(ToString::to_string);

    // Assert
    assert_eq!(source, Some(String::from("not held")));
}

#[test]
fn to_string_of_a_wrapped_failure_says_what_it_says() {
    // Arrange & Act
    let text = RunFailure::Config(ConfigFailure::Unreadable {
        path: path("stern4rust.toml"),
        cause: Box::new(NotHeld),
    })
    .to_string();

    // Assert
    assert_eq!(text, "stern4rust.toml could not be read");
}

#[test]
fn to_string_of_each_failure_of_its_own_says_what_to_do() {
    // Arrange
    let failures = [
        RunFailure::HeaderUnreadable(SourceReadFailure {
            path: path("docs/header.txt"),
            cause: Box::new(NotHeld),
        }),
        RunFailure::HeaderWithoutFile,
        RunFailure::NoBaselinePath,
        RunFailure::NoRules,
        RunFailure::PackageNotFound(String::from("gamma")),
        RunFailure::UnknownRules {
            unknown: vec![String::from("hedaer")],
            known: vec!["header", "tests-layout"],
        },
        RunFailure::UnknownSections {
            unknown: vec![String::from("gamma"), String::from("delta")],
            workspace: vec![String::from("alpha")],
        },
    ];

    // Act
    let texts: Vec<String> = failures.iter().map(ToString::to_string).collect();

    // Assert
    assert_eq!(
        texts,
        [
            "failed to read header file docs/header.txt",
            "--rule header needs --header-file, otherwise the run would apply no rules at all",
            "--write-baseline needs a path to write to",
            "no rules are configured, so nothing would be checked -- pass --header-file to \
             enable the header rule",
            "package `gamma` is not in this workspace",
            "unknown rule name(s): hedaer -- the rules are: header, tests-layout",
            "stern4rust.toml configures package(s) that are not in this workspace: gamma, delta \
             -- it holds: alpha",
        ]
    );
}
