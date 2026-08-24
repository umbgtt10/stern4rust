// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Every file's SPDX identifier says what the manifest says.
//
// The `header` rule compares a header against a text file and nothing else, so
// an SPDX line disagreeing with `Cargo.toml` passes it. This rule takes its
// expected value from the manifest instead, which is why it needs no
// `--header-file` to hold.

use stern4rust::reporting::offence::Offence;
use stern4rust::rule::Rule;
use stern4rust::rules::manifest::spdx_matches_manifest_rule::SpdxMatchesManifestRule;
use stern4rust::source_file::SourceFile;

const APACHE: &str = "// Copyright 2025 Umberto Gotti\n\
                      // Licensed under the Apache License, Version 2.0\n";

const MIT: &str = "// Copyright 2025 Umberto Gotti\n\
                   // SPDX-License-Identifier: MIT\n";

fn check(license: Option<&str>, path: &str, body: &str) -> Vec<Offence> {
    SpdxMatchesManifestRule::new(license.map(str::to_string)).check(&SourceFile::new(path, body))
}

fn check_workspace(license: Option<&str>, paths: &[&str]) -> Vec<Offence> {
    let files: Vec<SourceFile> = paths
        .iter()
        .map(|path| SourceFile::new(path, MIT))
        .collect();
    SpdxMatchesManifestRule::new(license.map(str::to_string)).check_workspace(&files)
}

// The etheram-core shape: prose claiming a licence, no machine-readable line.
#[test]
fn check_a_file_claiming_a_licence_in_prose_only_reports_it() {
    // Arrange & Act
    let offences = check(Some("MIT"), "src/widget.rs", APACHE);

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].rule, "spdx-matches-manifest");
    assert_eq!(offences[0].expected.as_deref(), Some("MIT"));
    assert_eq!(
        offences[0].correction,
        "add `// SPDX-License-Identifier: MIT` to the header, or correct the manifest"
    );
}

// The braintax4rust shape: a registry with no header at all.
#[test]
fn check_a_file_with_no_header_reports_it() {
    // Arrange & Act
    let offences = check(Some("MIT"), "src/traits/mod.rs", "pub mod reporter;\n");

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].subject.as_deref(), Some("src/traits/mod.rs"));
}

#[test]
fn check_a_matching_spdx_reports_nothing() {
    // Arrange & Act
    let offences = check(Some("MIT"), "src/widget.rs", MIT);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_a_mismatched_spdx_reports_it() {
    // Arrange & Act
    let offences = check(
        Some("MIT"),
        "src/widget.rs",
        "// SPDX-License-Identifier: Apache-2.0\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
    assert!(
        offences[0].description.contains("Apache-2.0"),
        "the identifier found is named, got {}",
        offences[0].description
    );
    assert!(
        offences[0].description.contains("MIT"),
        "the manifest's licence is named, got {}",
        offences[0].description
    );
}

// An SPDX line below the header is code or prose, not the file's declaration.
#[test]
fn check_a_spdx_line_below_the_header_reports_it() {
    // Arrange & Act
    let offences = check(
        Some("MIT"),
        "src/widget.rs",
        "// Copyright 2025\n\npub struct W;\n// SPDX-License-Identifier: MIT\n",
    );

    // Assert
    assert_eq!(offences.len(), 1);
}

// Without a licence in the manifest there is nothing to compare against, so the
// per-file question is not asked at all.
#[test]
fn check_with_no_manifest_licence_reports_nothing() {
    // Arrange & Act
    let offences = check(None, "src/widget.rs", APACHE);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_with_a_manifest_licence_reports_nothing() {
    // Arrange & Act
    let offences = check_workspace(Some("MIT"), &["src/a.rs"]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// The manifest's silence is not an offence but an absence of configuration.
// Reporting it per package root gave twenty identical lines on a twenty-package
// workspace; the registry drops the rule and the report names it instead.
#[test]
fn check_workspace_with_no_manifest_licence_reports_nothing() {
    // Arrange & Act
    let offences = check_workspace(None, &["src/a.rs", "src/b.rs", "src/c.rs"]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn is_configured_with_a_manifest_licence_returns_true() {
    // Arrange & Act
    let configured = SpdxMatchesManifestRule::new(Some("MIT".to_string())).is_configured();

    // Assert
    assert!(configured);
}

// The manifest configures this rule, so a manifest naming no licence leaves it
// nothing to work from.
#[test]
fn is_configured_without_a_manifest_licence_returns_false() {
    // Arrange & Act
    let configured = SpdxMatchesManifestRule::new(None).is_configured();

    // Assert
    assert!(!configured);
}

#[test]
fn name_is_the_kebab_case_rule_name_used_in_the_report() {
    // Arrange & Act
    let name = SpdxMatchesManifestRule::new(None).name();

    // Assert
    assert_eq!(name, "spdx-matches-manifest");
}

#[test]
fn requirement_names_the_manifest_field_the_rule_needs() {
    // Arrange & Act
    let requirement = SpdxMatchesManifestRule::new(None).requirement();

    // Assert
    assert_eq!(requirement, Some("needs a `license` field in Cargo.toml"));
}
