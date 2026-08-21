// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The `[package.<name>]` sections, and the two questions asked of them.
//
// The one that matters is the second. A section naming a package the run does
// not scan is not a harmless leftover: it reads as a rule set being applied, and
// a package quietly running every rule when its section said to skip one is the
// silence this tool exists to refuse. `deny_unknown_fields` cannot catch it,
// because the section name is data rather than a key.

use std::collections::BTreeMap;
use stern4rust::settings::package_config::PackageConfig;
use stern4rust::settings::package_sections::PackageSections;

fn sections(names: &[&str]) -> PackageSections {
    PackageSections::new(
        names
            .iter()
            .map(|name| ((*name).to_string(), PackageConfig::default()))
            .collect::<BTreeMap<_, _>>(),
    )
}

fn skipping(entries: &[(&str, &[&str])]) -> PackageSections {
    PackageSections::new(
        entries
            .iter()
            .map(|(name, skip)| {
                (
                    (*name).to_string(),
                    PackageConfig {
                        skip: skip.iter().map(|rule| (*rule).to_string()).collect(),
                        ..PackageConfig::default()
                    },
                )
            })
            .collect::<BTreeMap<_, _>>(),
    )
}

#[test]
fn is_empty_without_any_section_is_true() {
    // Arrange & Act
    let sections = sections(&[]);

    // Assert
    assert!(sections.is_empty());
}

#[test]
fn of_a_package_with_a_section_returns_it() {
    // Arrange
    let sections = sections(&["validation"]);

    // Act
    let section = sections.of("validation");

    // Assert
    assert!(section.is_some());
}

// Most packages have no section, which is the honest rendering of "this one
// applies everything".
#[test]
fn of_a_package_without_a_section_returns_nothing() {
    // Arrange
    let sections = sections(&["validation"]);

    // Act
    let section = sections.of("node");

    // Assert
    assert!(section.is_none());
}

// The report answers for the run as a whole, so a rule one package stood down on
// did not apply to the run. Naming it as skipped understates -- most packages
// applied it -- and understating is the only direction this tool may err in.
#[test]
fn skipped_anywhere_collects_what_any_section_stands_down_on() {
    // Arrange
    let sections = skipping(&[
        ("validation", &["paired-test-file"]),
        ("system-tests", &["paired-test-file", "test-naming"]),
    ]);

    // Act
    let skipped = sections.skipped_anywhere();

    // Assert
    assert_eq!(skipped, vec!["paired-test-file", "test-naming"]);
}

// One rule stood down on by two packages is one rule, not two.
#[test]
fn skipped_anywhere_names_a_rule_two_sections_share_once() {
    // Arrange
    let sections = skipping(&[
        ("validation", &["paired-test-file"]),
        ("system-tests", &["paired-test-file"]),
    ]);

    // Act
    let skipped = sections.skipped_anywhere();

    // Assert
    assert_eq!(skipped, vec!["paired-test-file"]);
}

#[test]
fn skipped_anywhere_without_any_section_is_empty() {
    // Arrange & Act
    let skipped = sections(&[]).skipped_anywhere();

    // Assert
    assert!(skipped.is_empty());
}

// The report has to name what it scans as well as what it could not find, or a
// reader with a typo has nothing to compare against.
#[test]
fn validate_names_the_packages_the_run_does_scan() {
    // Arrange
    let sections = sections(&["nope"]);

    // Act
    let result = sections.validate(&["node", "validation"][..]);

    // Assert
    let error = format!("{}", result.expect_err("an error"));
    assert!(error.contains("node"));
    assert!(error.contains("validation"));
}

// Scoping a run to one package is an ordinary thing to do -- a developer
// checking one crate, a gate that wants one -- and the sections for the others
// are not typos. Validating against the scan rather than the workspace made
// `--package node` an error in any repository whose root config had sections,
// which is every repository this feature was built for.
#[test]
fn validate_of_a_section_for_a_workspace_package_outside_this_run_is_ok() {
    // Arrange -- system-tests is in the workspace; this run just is not looking
    // at it.
    let sections = sections(&["system-tests"]);

    // Act
    let result = sections.validate(&["node", "system-tests"][..]);

    // Assert
    assert!(result.is_ok());
}

// The failure this type exists for.
#[test]
fn validate_where_a_section_names_no_scanned_package_is_an_error() {
    // Arrange -- a plausible typo rather than nonsense.
    let sections = sections(&["validaton"]);

    // Act
    let result = sections.validate(&["node", "validation"][..]);

    // Assert
    let error = result.expect_err("a section naming nothing must not pass");
    assert!(format!("{error}").contains("validaton"));
}

#[test]
fn validate_where_every_section_names_a_scanned_package_is_ok() {
    // Arrange
    let sections = sections(&["node", "validation"]);

    // Act
    let result = sections.validate(&["node", "node-infra", "validation"][..]);

    // Assert
    assert!(result.is_ok());
}
