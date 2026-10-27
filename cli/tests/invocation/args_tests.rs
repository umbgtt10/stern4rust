// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The argv fixup every cargo subcommand needs. Cargo runs `cargo stern4rust ...`
// as `cargo-stern4rust stern4rust ...`, so the subcommand name arrives twice;
// running the binary directly does not repeat it. The strip therefore has to be
// conditional, and it has to look at position 1 only -- a package that happens
// to be called stern4rust must survive.

use clap::Parser;
use stern4rust_cli::invocation::args::Args;
use stern4rust_cli::invocation::output_format::OutputFormat;

fn argv(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|part| (*part).to_string()).collect()
}

#[test]
fn invocation_carries_every_option_it_was_given() {
    // Arrange
    let args = Args::parse_from(argv(&[
        "cargo-stern4rust",
        "--manifest-path",
        "work/Cargo.toml",
        "--package",
        "node",
        "--header-file",
        "docs/header.txt",
        "--offence-threshold",
        "7",
        "--rule",
        "header",
        "--skip",
        "tests-layout",
        "--exclude",
        "vendor/**",
        "--baseline",
        "base.json",
        "--write-baseline",
        "--fix",
    ]));

    // Act
    let invocation = args.invocation().expect("UTF-8 paths");

    // Assert
    assert_eq!(invocation.manifest_directory.as_str(), "work");
    assert_eq!(invocation.packages, ["node"]);
    assert_eq!(
        invocation.header_file.as_ref().map(|path| path.as_str()),
        Some("docs/header.txt")
    );
    assert_eq!(invocation.offence_threshold, Some(7));
    assert_eq!(invocation.rules, ["header"]);
    assert_eq!(invocation.skipped_rules, ["tests-layout"]);
    assert_eq!(invocation.excludes, ["vendor/**"]);
    assert_eq!(
        invocation.baseline.as_ref().map(|path| path.as_str()),
        Some("base.json")
    );
    assert!(invocation.write_baseline);
    assert!(invocation.fix);
    assert!(!invocation.list_rules);
}

// The config and a default baseline are looked for beside the manifest, and a
// manifest named without a directory sits in the current one.
#[test]
fn invocation_of_a_bare_manifest_name_looks_in_the_current_directory() {
    // Arrange
    let args = Args::parse_from(argv(&["cargo-stern4rust", "--manifest-path", "Cargo.toml"]));

    // Act
    let invocation = args.invocation().expect("UTF-8 paths");

    // Assert
    assert_eq!(invocation.manifest_directory.as_str(), "");
}

#[test]
fn invocation_without_a_manifest_path_looks_in_the_current_directory() {
    // Arrange
    let args = Args::parse_from(argv(&["cargo-stern4rust", "--rules"]));

    // Act
    let invocation = args.invocation().expect("UTF-8 paths");

    // Assert
    assert_eq!(invocation.manifest_directory.as_str(), ".");
    assert!(invocation.list_rules);
}

#[test]
fn parse_from_collects_every_exclude_flag() {
    // Arrange & Act
    let args = Args::parse_from(argv(&[
        "cargo-stern4rust",
        "--exclude",
        "fixture/**",
        "--exclude",
        "vendor/**",
    ]));

    // Assert
    assert_eq!(
        args.excludes(),
        ["fixture/**".to_string(), "vendor/**".to_string()]
    );
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
    assert_eq!(
        args.packages(),
        vec!["alpha".to_string(), "beta".to_string()]
    );
}

#[test]
fn parse_from_collects_every_skip_flag() {
    // Arrange & Act
    let args = Args::parse_from(argv(&[
        "cargo-stern4rust",
        "--skip",
        "header",
        "--skip",
        "test-naming",
    ]));

    // Assert
    assert_eq!(
        args.skipped_rules(),
        ["header".to_string(), "test-naming".to_string()]
    );
}

#[test]
fn parse_from_defaults_to_no_packages_and_no_manifest_path() {
    // Arrange & Act
    let args = Args::parse_from(argv(&["cargo-stern4rust"]));

    // Assert
    assert!(args.packages().is_empty());
    assert!(args.manifest_path().is_none());
}

#[test]
fn parse_from_of_no_switches_leaves_every_one_off() {
    // Arrange & Act
    let args = Args::parse_from(argv(&["cargo-stern4rust"]));

    // Assert
    assert!(!args.fix());
    assert!(!args.list_rules());
    assert!(!args.write_baseline());
    assert_eq!(args.format(), OutputFormat::Text);
}

#[test]
fn parse_from_reads_each_switch_and_the_format() {
    // Arrange & Act
    let args = Args::parse_from(argv(&[
        "cargo-stern4rust",
        "--fix",
        "--rules",
        "--write-baseline",
        "--format",
        "json",
    ]));

    // Assert
    assert!(args.fix());
    assert!(args.list_rules());
    assert!(args.write_baseline());
    assert_eq!(args.format(), OutputFormat::Json);
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
        args.manifest_path(),
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
