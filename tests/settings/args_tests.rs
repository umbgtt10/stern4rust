// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The argv fixup every cargo subcommand needs. Cargo runs `cargo stern4rust ...`
// as `cargo-stern4rust stern4rust ...`, so the subcommand name arrives twice;
// running the binary directly does not repeat it. The strip therefore has to be
// conditional, and it has to look at position 1 only -- a package that happens
// to be called stern4rust must survive.

use clap::Parser;
use stern4rust::settings::args::Args;

fn argv(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|part| (*part).to_string()).collect()
}

#[test]
fn parse_from_collects_every_package_flag() {
    // Arrange & Act
    let args = Args::parse_from(argv(&[
        "cargo-stern4rust",
        "--package",
        "alpha",
        "--package",
        "beta",
    ]));

    // Assert
    assert_eq!(args.packages, vec!["alpha".to_string(), "beta".to_string()]);
}

#[test]
fn parse_from_defaults_to_no_packages_and_no_manifest_path() {
    // Arrange & Act
    let args = Args::parse_from(argv(&["cargo-stern4rust"]));

    // Assert
    assert!(args.packages.is_empty());
    assert!(args.manifest_path.is_none());
}

#[test]
fn parse_from_reads_the_manifest_path() {
    // Arrange & Act
    let args = Args::parse_from(argv(&[
        "cargo-stern4rust",
        "--manifest-path",
        "/tmp/Cargo.toml",
    ]));

    // Assert
    assert_eq!(
        args.manifest_path.as_deref(),
        Some(std::path::Path::new("/tmp/Cargo.toml"))
    );
}

#[test]
fn without_cargo_subcommand_drops_the_name_cargo_inserts() {
    // Arrange
    let args = argv(&["cargo-stern4rust", "stern4rust", "--package", "alpha"]);

    // Act
    let forwarded = Args::without_cargo_subcommand(args);

    // Assert
    assert_eq!(forwarded, argv(&["cargo-stern4rust", "--package", "alpha"]));
}

#[test]
fn without_cargo_subcommand_handles_a_bare_binary_name() {
    // Arrange
    let args = argv(&["cargo-stern4rust"]);

    // Act
    let forwarded = Args::without_cargo_subcommand(args.clone());

    // Assert
    assert_eq!(forwarded, args);
}

#[test]
fn without_cargo_subcommand_handles_being_given_nothing() {
    // Arrange & Act
    let forwarded = Args::without_cargo_subcommand(Vec::new());

    // Assert
    assert!(forwarded.is_empty());
}

// The strip is positional. Dropping every occurrence would silently discard a
// package argument, and the run would then cover the wrong set of packages
// while still exiting successfully.
#[test]
fn without_cargo_subcommand_keeps_a_package_that_happens_to_be_named_stern4rust() {
    // Arrange
    let args = argv(&["cargo-stern4rust", "--package", "stern4rust"]);

    // Act
    let forwarded = Args::without_cargo_subcommand(args.clone());

    // Assert
    assert_eq!(forwarded, args);
}

#[test]
fn without_cargo_subcommand_leaves_a_direct_invocation_untouched() {
    // Arrange
    let args = argv(&["cargo-stern4rust", "--package", "alpha"]);

    // Act
    let forwarded = Args::without_cargo_subcommand(args.clone());

    // Assert
    assert_eq!(forwarded, args);
}

#[test]
fn without_cargo_subcommand_strips_only_the_first_occurrence() {
    // Arrange
    let args = argv(&["cargo-stern4rust", "stern4rust", "--package", "stern4rust"]);

    // Act
    let forwarded = Args::without_cargo_subcommand(args);

    // Assert
    assert_eq!(
        forwarded,
        argv(&["cargo-stern4rust", "--package", "stern4rust"])
    );
}
