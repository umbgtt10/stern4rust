// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Turning the workspace cargo described into the packages a run walks.
//
// The case that matters most is a package name that does not exist. Returning an
// empty result there would let a typo in a gate script scan nothing and report
// success, which is the failure mode that makes a green gate meaningless -- so
// it is an error instead. What cargo itself says about a real workspace is the
// command line's `CargoWorkspaceReader`, tested there.

use stern4rust::finding::model::manifest_dependency::ManifestDependency;
use stern4rust::settings::config::Config;
use stern4rust::settings::manifest_resolver::ManifestResolver;
use stern4rust::settings::package_description::PackageDescription;
use stern4rust::settings::workspace_description::WorkspaceDescription;
use stern4rust::source_path::SourcePath;

fn config_for(packages: &[&str]) -> Config {
    Config {
        packages: packages.iter().map(|name| (*name).to_string()).collect(),
        ..Config::default()
    }
}

fn member(name: &str, declared: Vec<ManifestDependency>) -> PackageDescription {
    PackageDescription {
        name: name.to_string(),
        manifest_path: path(&format!("/repo/{name}/Cargo.toml")),
        license: Some(String::from("MIT")),
        declared,
    }
}

fn path(text: &str) -> SourcePath {
    SourcePath::new(text.to_string())
}

fn workspace(declares_workspace: bool) -> WorkspaceDescription {
    WorkspaceDescription {
        root: path("/repo"),
        declares_workspace,
        packages: vec![
            member(
                "alpha",
                vec![
                    ManifestDependency::new("alpha/Cargo.toml", "anyhow", "dependencies", true),
                    ManifestDependency::new("alpha/Cargo.toml", "nix", "dependencies", false)
                        .with_target(Some("cfg(unix)")),
                ],
            ),
            member(
                "beta",
                vec![ManifestDependency::new(
                    "beta/Cargo.toml",
                    "serde",
                    "dev-dependencies",
                    false,
                )],
            ),
        ],
    }
}

#[test]
fn declared_dependencies_names_what_every_member_declares_in_member_order() {
    // Arrange
    let workspace = workspace(true);

    // Act
    let declared = ManifestResolver::declared_dependencies(&workspace);

    // Assert
    let names: Vec<&str> = declared
        .iter()
        .map(|dependency| dependency.name())
        .collect();
    assert_eq!(names, ["anyhow", "nix", "serde"]);
}

#[test]
fn packages_of_a_named_member_returns_it_rooted_at_its_manifest_directory() {
    // Arrange
    let workspace = workspace(true);

    // Act
    let packages = ManifestResolver::packages(&workspace, &config_for(&["beta"])).expect("resolve");

    // Assert
    assert_eq!(packages.len(), 1);
    assert_eq!(packages[0].name(), "beta");
    assert_eq!(packages[0].root(), &path("/repo/beta"));
    assert_eq!(packages[0].license(), Some("MIT"));
}

#[test]
fn packages_of_an_unknown_package_names_it_in_the_error() {
    // Arrange
    let workspace = workspace(true);

    // Act
    let error = ManifestResolver::packages(&workspace, &config_for(&["gamma"]))
        .expect_err("an unknown package");

    // Assert
    assert_eq!(
        error.to_string(),
        "package `gamma` is not in this workspace"
    );
}

#[test]
fn packages_without_a_named_package_returns_every_member_in_order() {
    // Arrange
    let workspace = workspace(true);

    // Act
    let packages = ManifestResolver::packages(&workspace, &config_for(&[])).expect("resolve");

    // Assert
    let names: Vec<&str> = packages.iter().map(|package| package.name()).collect();
    assert_eq!(names, ["alpha", "beta"]);
}

#[test]
fn relative_to_a_path_outside_the_root_returns_the_whole_path() {
    // Arrange
    let root = path("/workspace/crate");
    let path = path("/elsewhere/a.rs");

    // Act
    let relative = ManifestResolver::relative_to(&root, &path);

    // Assert
    assert!(relative.ends_with("a.rs"));
}

// Reports are read by people and pasted into scripts, so a path looks the same
// whichever platform produced it.
#[test]
fn relative_to_reports_forward_slashes() {
    // Arrange
    let root = path("/workspace");
    let path = path("/workspace/src/rules/header_rule.rs");

    // Act
    let relative = ManifestResolver::relative_to(&root, &path);

    // Assert
    assert!(!relative.contains('\\'));
}

#[test]
fn relative_to_strips_the_root_prefix() {
    // Arrange
    let root = path("/workspace/crate");
    let path = path("/workspace/crate/src/a.rs");

    // Act
    let relative = ManifestResolver::relative_to(&root, &path);

    // Assert
    assert_eq!(relative, "src/a.rs");
}

#[test]
fn workspace_dependencies_leaves_target_tables_out() {
    // Arrange
    let workspace = workspace(true);

    // Act
    let declared = ManifestResolver::workspace_dependencies(&workspace);

    // Assert
    let declared = declared.expect("a workspace");
    let names: Vec<&str> = declared
        .iter()
        .map(|dependency| dependency.name())
        .collect();
    assert_eq!(names, ["anyhow", "serde"]);
}

// A single package is no workspace, which is not the same as a workspace with
// nothing to report.
#[test]
fn workspace_dependencies_of_a_root_declaring_no_workspace_is_none() {
    // Arrange
    let workspace = workspace(false);

    // Act
    let declared = ManifestResolver::workspace_dependencies(&workspace);

    // Assert
    assert!(declared.is_none());
}

#[test]
fn workspace_package_names_names_every_member() {
    // Arrange
    let workspace = workspace(true);

    // Act
    let names = ManifestResolver::workspace_package_names(&workspace);

    // Assert
    assert_eq!(names, ["alpha", "beta"]);
}

#[test]
fn workspace_root_is_the_root_cargo_described() {
    // Arrange
    let workspace = workspace(true);

    // Act
    let root = ManifestResolver::workspace_root(&workspace);

    // Assert
    assert_eq!(root, path("/repo"));
}
