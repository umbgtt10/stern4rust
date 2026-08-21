// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Turning the requested packages into directories to walk.
//
// The case that matters most is a package name that does not exist. Returning an
// empty result there would let a typo in a gate script scan nothing and report
// success, which is the failure mode that makes a green gate meaningless -- so
// it is an error instead.

use std::path::Path;
use stern4rust::settings::config::Config;
use stern4rust::settings::manifest_resolver::ManifestResolver;

fn config_for(packages: &[&str]) -> Config {
    Config {
        manifest_path: Some(Path::new("Cargo.toml").to_path_buf()),
        packages: packages.iter().map(|name| (*name).to_string()).collect(),
        ..Config::default()
    }
}

#[test]
fn packages_names_the_package_it_could_not_find() {
    // Arrange
    let config = config_for(&["no-such-package"]);

    // Act
    let error = ManifestResolver::packages(&config).expect_err("an error");

    // Assert
    assert!(error.to_string().contains("no-such-package"));
}

// A typo must fail loudly rather than scan nothing and pass.
#[test]
fn packages_of_an_unknown_package_is_an_error() {
    // Arrange
    let config = config_for(&["no-such-package"]);

    // Act
    let result = ManifestResolver::packages(&config);

    // Assert
    assert!(result.is_err());
}

// A name that is not in the workspace is an error rather than an empty result,
// so a typo in a gate script cannot scan nothing and report success.
#[test]
fn packages_of_an_unknown_package_returns_an_error() {
    // Arrange
    let config = config_for(&["no-such-package"]);

    // Act
    let result = ManifestResolver::packages(&config);

    // Assert
    assert!(result.is_err());
}

#[test]
fn packages_of_this_crate_by_name_returns_its_directory() {
    // Arrange
    let config = config_for(&["cargo-stern4rust"]);

    // Act
    let roots = ManifestResolver::packages(&config).expect("resolve");

    // Assert
    assert_eq!(roots.len(), 1);
    assert!(roots[0].root.join("Cargo.toml").exists());
}

// The expectation `spdx-matches-manifest` holds every header to, taken from the
// package being judged rather than from a flag -- and carried on the package
// rather than aggregated across the run.
#[test]
fn packages_of_this_crate_carries_the_licence_its_manifest_declares() {
    // Arrange
    let config = config_for(&["cargo-stern4rust"]);

    // Act
    let packages = ManifestResolver::packages(&config).expect("resolve packages");

    // Assert
    assert_eq!(packages.len(), 1);
    assert_eq!(packages[0].name, "cargo-stern4rust");
    assert_eq!(packages[0].license.as_deref(), Some("MIT"));
}

#[test]
fn packages_without_a_named_package_returns_this_crate() {
    // Arrange
    let config = config_for(&[]);

    // Act
    let roots = ManifestResolver::packages(&config).expect("resolve");

    // Assert
    assert_eq!(roots.len(), 1);
}

#[test]
fn relative_to_a_path_outside_the_root_returns_the_whole_path() {
    // Arrange
    let root = Path::new("/workspace/crate");
    let path = Path::new("/elsewhere/a.rs");

    // Act
    let relative = ManifestResolver::relative_to(root, path);

    // Assert
    assert!(relative.ends_with("a.rs"));
}

// Reports are read by people and pasted into scripts, so a path looks the same
// whichever platform produced it.
#[test]
fn relative_to_reports_forward_slashes() {
    // Arrange
    let root = Path::new("/workspace");
    let path = Path::new("/workspace/src/rules/header_rule.rs");

    // Act
    let relative = ManifestResolver::relative_to(root, path);

    // Assert
    assert!(!relative.contains('\\'));
}

#[test]
fn relative_to_strips_the_root_prefix() {
    // Arrange
    let root = Path::new("/workspace/crate");
    let path = Path::new("/workspace/crate/src/a.rs");

    // Act
    let relative = ManifestResolver::relative_to(root, path);

    // Assert
    assert_eq!(relative, "src/a.rs");
}

// This crate is a single package, not a workspace, so there is no root to
// centralise into and the rule stays silent rather than reporting.
#[test]
fn workspace_dependencies_of_a_package_that_is_not_a_workspace_is_none() {
    // Arrange
    let config = config_for(&["cargo-stern4rust"]);

    // Act
    let declared = ManifestResolver::workspace_dependencies(&config);

    // Assert
    assert!(declared.is_none());
}
