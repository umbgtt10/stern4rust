// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The wiring from arguments to a verdict, run against this crate's own tree.
//
// The verdict is a return value rather than a call to exit, which is the only
// reason any of this is reachable from a test at all. Two outcomes have to stay
// tellable apart: a broken rule is a successful run with a finding, while a tool
// that could not run is an Err. Collapsing them would let "I could not read your
// code" reach a gate script as a pass.

use clap::Parser;
use serde_json::Value;
use serde_json::from_str;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use stern4rust::reporting::run_outcome::RunOutcome;
use stern4rust::runner::Runner;
use stern4rust::settings::args::Args;
use stern4rust::settings::config_file::ConfigFile;

const THIS_CRATE: &str = "cargo-stern4rust";

fn args_from(parts: &[&str]) -> Args {
    Args::parse_from(parts.iter().map(|part| (*part).to_string()))
}

fn config_directory(name: &str, contents: &str) -> PathBuf {
    let path = probe_package(name);
    fs::write(path.join("stern4rust.toml"), contents).expect("write the config");
    path
}

fn header_file(name: &str, contents: &str) -> PathBuf {
    let path = env::temp_dir().join(format!("stern4rust_header_{name}.txt"));
    fs::write(&path, contents).expect("write the header file");
    path
}

// A package with no stern4rust.toml beside it. Tests that need the absence of a
// config cannot point at this repository, because this repository has one.
fn probe_package(name: &str) -> PathBuf {
    let path = env::temp_dir().join(format!("stern4rust_run_{name}"));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(path.join("src")).expect("create the package");
    fs::write(
        path.join("Cargo.toml"),
        "[package]
name = \"probe\"
version = \"0.1.0\"
edition = \"2021\"
",
    )
    .expect("write the manifest");
    fs::write(
        path.join("src/lib.rs"),
        "pub mod widget;
",
    )
    .expect("write the registry");
    fs::write(
        path.join("src/widget.rs"),
        "pub struct Widget;
",
    )
    .expect("write the module");
    path
}

// A real workspace, judged whole and judged one member at a time.
//
// Both bugs in the per-package configuration reached a release past a full unit
// suite and nine green gates, because nothing here built a workspace and ran the
// tool the way a person does. 0.9.3 made `--package <member>` an error in any
// repository whose root config had sections; 0.9.4 fixed that and left a scoped
// run reporting a rule as applied and skipped three lines apart. Unit tests on
// the pieces guard the shapes now, but only this reaches the wiring.
fn probe_workspace(name: &str) -> PathBuf {
    let root = env::temp_dir().join(format!("stern4rust_ws_{name}"));
    let _ = fs::remove_dir_all(&root);
    for member in ["alpha", "beta"] {
        fs::create_dir_all(root.join(member).join("src")).expect("create the member");
        fs::write(
            root.join(member).join("Cargo.toml"),
            format!(
                "[package]
name = \"{member}\"
version = \"0.1.0\"
edition = \"2021\"
"
            ),
        )
        .expect("write the member manifest");
        fs::write(
            root.join(member).join("src/lib.rs"),
            "pub mod widget;
",
        )
        .expect("write the registry");
        fs::write(
            root.join(member).join("src/widget.rs"),
            "pub struct Widget;
",
        )
        .expect("write the module");
    }
    fs::write(
        root.join("Cargo.toml"),
        "[workspace]
resolver = \"2\"
members = [\"alpha\", \"beta\"]
",
    )
    .expect("write the workspace manifest");
    // A section for one member only, which is the arrangement both bugs needed.
    fs::write(
        root.join(ConfigFile::NAME),
        "[package.beta]
skip = [\"test-free-source\"]
",
    )
    .expect("write the config");
    root
}

// The invariant both per-package bugs broke, in the shape they broke it: a run
// must never name a rule as skipped that its own roster lists as applied.
//
// 0.9.3 broke it by failing outright; 0.9.5 by folding stand-downs from packages
// the run was not walking into the summary, so `--package alpha` reported a rule
// applied and skipped three lines apart. Nothing caught the second -- the run
// stays perfectly Ok while contradicting itself, and every test asserted on the
// outcome rather than on what was said.
fn report_for(root: &Path, package: Option<&str>) -> String {
    let manifest = root.join("Cargo.toml");
    let mut parts = vec![
        "cargo-stern4rust".to_string(),
        "--manifest-path".to_string(),
        manifest.to_string_lossy().into_owned(),
    ];
    if let Some(name) = package {
        parts.push("--package".to_string());
        parts.push(name.to_string());
    }
    Runner::run_reporting(args_from(
        &parts.iter().map(String::as_str).collect::<Vec<_>>(),
    ))
    .expect("the run itself should succeed")
    .1
}

fn run_with_header(name: &str, contents: &str) -> RunOutcome {
    let path = header_file(name, contents);
    Runner::run(args_from(&[
        "cargo-stern4rust",
        "--manifest-path",
        "Cargo.toml",
        "--package",
        THIS_CRATE,
        "--header-file",
        &path.to_string_lossy(),
    ]))
    .expect("the run itself should succeed")
}

fn run_workspace(root: &Path, package: Option<&str>) -> Result<RunOutcome, anyhow::Error> {
    let manifest = root.join("Cargo.toml");
    let mut parts = vec![
        "cargo-stern4rust".to_string(),
        "--manifest-path".to_string(),
        manifest.to_string_lossy().into_owned(),
    ];
    if let Some(name) = package {
        parts.push("--package".to_string());
        parts.push(name.to_string());
    }
    Runner::run(args_from(
        &parts.iter().map(String::as_str).collect::<Vec<_>>(),
    ))
}

// A typo in a gate script must fail loudly rather than scan nothing and pass.
#[test]
fn run_against_an_unknown_package_is_an_error() {
    // Arrange
    let path = header_file("unknown_package", "// Copyright 2025\n");
    let args = args_from(&[
        "cargo-stern4rust",
        "--manifest-path",
        "Cargo.toml",
        "--package",
        "no-such-package",
        "--header-file",
        &path.to_string_lossy(),
    ]);

    // Act
    let result = Runner::run(args);

    // Assert
    assert!(result.is_err());
}

#[test]
fn run_against_this_crate_with_a_header_it_does_not_carry_reports_rules_broken() {
    // Arrange & Act
    let outcome = run_with_header("foreign", "// Copyright 1999 Someone Else\n");

    // Assert
    assert_eq!(outcome, RunOutcome::RulesBroken);
}

// This crate keeps its own rule, so pointing stern4rust at itself with its own
// header is the end-to-end case: it walks, reads and judges every file here.
#[test]
fn run_against_this_crate_with_its_own_header_is_clean() {
    // Arrange & Act
    let outcome = run_with_header(
        "own",
        "// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>\n\
         // Licensed under the MIT License\n\
         // SPDX-License-Identifier: MIT\n",
    );

    // Assert
    assert_eq!(outcome, RunOutcome::Clean);
}

#[test]
fn run_over_a_whole_workspace_with_a_section_succeeds() {
    // Arrange
    let root = probe_workspace("whole");

    // Act
    let result = run_workspace(&root, None);

    // Assert
    assert!(result.is_ok(), "{:?}", result.err());
}

// A section naming no member of the workspace is still an error, which is the
// case the 0.9.4 fix had to keep while letting the two above through.
#[test]
fn run_over_a_workspace_whose_section_names_no_member_is_an_error() {
    // Arrange
    let root = probe_workspace("typo");
    fs::write(
        root.join(ConfigFile::NAME),
        "[package.gamma]
skip = [\"test-free-source\"]
",
    )
    .expect("write the config");

    // Act
    let result = run_workspace(&root, None);

    // Assert
    let error = result.expect_err("a section naming no member must not pass");
    assert!(format!("{error}").contains("gamma"));
}

// And the whole workspace still names it, because there beta really does stand
// it down -- so the tests above pin scoping rather than the rule vanishing.
#[test]
fn run_over_the_whole_workspace_reports_the_stand_down_its_section_asks_for() {
    // Arrange
    let root = probe_workspace("report_whole");

    // Act
    let report = report_for(&root, None);

    // Assert
    assert!(report.contains("test-free-source (skipped)"), "{report}");
    assert!(report.contains("rules_skipped=1"), "{report}");
}

// The JSON document is what a gate script reads, and until this test the only
// thing joining the runner to the JSON printer was a line nothing exercised.
// Removing that line passed all 696 tests: the printer had five tests of its
// own and every one of them called the printer directly.
#[test]
fn run_reporting_in_json_carries_a_roster_for_each_package() {
    // Arrange
    let root = probe_workspace("json_rosters");
    let manifest = root.join("Cargo.toml");

    // Act
    let (_, report) = Runner::run_reporting(args_from(&[
        "cargo-stern4rust",
        "--manifest-path",
        &manifest.to_string_lossy(),
        "--format",
        "json",
    ]))
    .expect("the run itself should succeed");

    // Assert
    let document: Value = from_str(&report).expect("valid json");
    let packages = document["packages"].as_array().expect("a packages array");
    let named: Vec<&str> = packages
        .iter()
        .filter_map(|package| package["package"].as_str())
        .collect();
    assert_eq!(named, vec!["alpha", "beta"]);
}

// And the detail is the per-package one rather than the run-level aggregate
// repeated, which is the difference a machine reading this needs.
#[test]
fn run_reporting_in_json_names_the_member_that_stood_a_rule_down() {
    // Arrange
    let root = probe_workspace("json_skip");
    let manifest = root.join("Cargo.toml");

    // Act
    let (_, report) = Runner::run_reporting(args_from(&[
        "cargo-stern4rust",
        "--manifest-path",
        &manifest.to_string_lossy(),
        "--format",
        "json",
    ]))
    .expect("the run itself should succeed");

    // Assert
    let document: Value = from_str(&report).expect("valid json");
    let beta = document["packages"]
        .as_array()
        .expect("array")
        .iter()
        .find(|package| package["package"] == "beta")
        .expect("beta");
    assert_eq!(beta["rules_skipped"][0], "test-free-source");
    let alpha = document["packages"]
        .as_array()
        .expect("array")
        .iter()
        .find(|package| package["package"] == "alpha")
        .expect("alpha");
    assert_eq!(alpha["rules_skipped"].as_array().expect("array").len(), 0);
}

// The roster and the summary describe the same run and must agree about it.
#[test]
fn run_scoped_to_a_member_never_names_an_applied_rule_as_skipped() {
    // Arrange
    let root = probe_workspace("report_agree");

    // Act
    let report = report_for(&root, Some("alpha"));

    // Assert
    let applied = report
        .lines()
        .find(|line| line.trim_start().starts_with("applied:"))
        .expect("a roster");
    assert!(applied.contains("test-free-source"), "{report}");
    assert!(!report.contains("test-free-source (skipped)"), "{report}");
}

#[test]
fn run_scoped_to_a_member_without_a_section_reports_nothing_skipped() {
    // Arrange
    let root = probe_workspace("report_alpha");

    // Act
    let report = report_for(&root, Some("alpha"));

    // Assert
    assert!(
        !report.contains("(skipped)"),
        "alpha has no section, so nothing was stood down:\n{report}"
    );
    assert!(report.contains("rules_skipped=0"), "{report}");
}

// The 0.9.3 failure, in the invocation that hit it: scoping to the member with
// no section left the section for the other one looking like a typo.
#[test]
fn run_scoped_to_a_member_without_a_section_succeeds() {
    // Arrange
    let root = probe_workspace("scoped_alpha");

    // Act
    let result = run_workspace(&root, Some("alpha"));

    // Assert
    assert!(
        result.is_ok(),
        "a section for another member must not fail this run: {:?}",
        result.err()
    );
}

#[test]
fn run_scoped_to_the_member_that_has_the_section_succeeds() {
    // Arrange
    let root = probe_workspace("scoped_beta");

    // Act
    let result = run_workspace(&root, Some("beta"));

    // Assert
    assert!(result.is_ok(), "{:?}", result.err());
}

#[test]
fn run_scoped_to_the_member_with_a_section_reports_its_own_stand_down() {
    // Arrange
    let root = probe_workspace("report_beta");

    // Act
    let report = report_for(&root, Some("beta"));

    // Assert
    assert!(report.contains("test-free-source (skipped)"), "{report}");
    assert!(report.contains("rules_skipped=1"), "{report}");
}

// A switch that quietly matched nothing would look exactly like a switch that
// worked, which is the whole reason this is an error rather than a no-op.
// The end-to-end proof that an exclusion removes files from judgement rather
// than merely from the report: the same run that reports offences against a
// header this crate does not carry finds nothing once every file is excluded.
// A repository states its settings once instead of at every invocation, and
// the report names the file so the switches in force are not invisible.
#[test]
fn run_with_a_config_file_applies_its_settings() {
    // Arrange
    let path = config_directory(
        "applies",
        "rules = [\"tests-layout\"]
",
    );

    // Act
    let outcome = Runner::run(args_from(&[
        "cargo-stern4rust",
        "--manifest-path",
        &path.join("Cargo.toml").to_string_lossy(),
    ]))
    .expect("the run itself should succeed");

    // Assert
    assert_eq!(outcome, RunOutcome::Clean);
}

// The whole point of a baseline, end to end: a run that failed passes once its
// offences are recorded, and fails again the moment a new one appears.
#[test]
fn run_with_a_written_baseline_forgives_the_old_and_still_fails_on_the_new() {
    // Arrange
    let path = config_directory(
        "baseline",
        "rules = [\"imported-paths\"]
",
    );
    let widget = path.join("src/widget.rs");
    fs::write(
        &widget,
        "pub struct W;
impl W { pub fn go() { let _ = std::env::args(); } }
",
    )
    .expect("write the offence");
    let manifest = path.join("Cargo.toml").to_string_lossy().to_string();
    let judge = || {
        Runner::run(args_from(&[
            "cargo-stern4rust",
            "--manifest-path",
            &manifest,
        ]))
        .expect("the run itself should succeed")
    };
    assert_eq!(
        judge(),
        RunOutcome::RulesBroken,
        "the offence should be seen"
    );

    // Act
    Runner::run(args_from(&[
        "cargo-stern4rust",
        "--manifest-path",
        &manifest,
        "--write-baseline",
    ]))
    .expect("writing the baseline should succeed");

    // Assert
    assert_eq!(
        judge(),
        RunOutcome::Clean,
        "the recorded offence is forgiven"
    );
    fs::write(
        &widget,
        "pub struct W;
impl W { pub fn go() { let _ = std::env::args(); let _ = std::env::vars(); } }
",
    )
    .expect("introduce a new offence");
    assert_eq!(
        judge(),
        RunOutcome::RulesBroken,
        "a new offence still fails"
    );
}

#[test]
fn run_with_an_exclusion_covering_every_file_finds_nothing_to_judge() {
    // Arrange
    let path = header_file(
        "excluded",
        "// nobody carries this header
",
    );

    // Act
    let outcome = Runner::run(args_from(&[
        "cargo-stern4rust",
        "--manifest-path",
        "Cargo.toml",
        "--package",
        THIS_CRATE,
        "--header-file",
        &path.to_string_lossy(),
        "--exclude",
        "**/*.rs",
    ]))
    .expect("the run itself should succeed");

    // Assert
    assert_eq!(outcome, RunOutcome::Clean);
}

// A file that exists and cannot be understood must not be treated as absent:
// running as though it were would apply a configuration nobody chose.
#[test]
fn run_with_an_invalid_config_file_is_an_error() {
    // Arrange
    let path = config_directory(
        "invalid",
        "rules = 7
",
    );

    // Act
    let outcome = Runner::run(args_from(&[
        "cargo-stern4rust",
        "--manifest-path",
        &path.join("Cargo.toml").to_string_lossy(),
    ]));

    // Assert
    assert!(outcome.is_err());
}

#[test]
fn run_with_an_unknown_rule_name_is_an_error() {
    // Arrange
    let args = args_from(&[
        "cargo-stern4rust",
        "--manifest-path",
        "Cargo.toml",
        "--skip",
        "test-file-strucutre",
    ]);

    // Act
    let result = Runner::run(args);

    // Assert
    assert!(result.is_err());
}

#[test]
fn run_with_an_unreadable_header_file_is_an_error() {
    // Arrange
    let absent = env::temp_dir().join("stern4rust_header_absent.txt");
    let _ = fs::remove_file(&absent);
    let args = args_from(&[
        "cargo-stern4rust",
        "--manifest-path",
        "Cargo.toml",
        "--header-file",
        &absent.to_string_lossy(),
    ]);

    // Act
    let result = Runner::run(args);

    // Assert
    assert!(result.is_err());
}

// Asking for a rule by name and getting an empty run is worse than not asking:
// the registry's usual habit of leaving an unconfigurable rule out silently is
// right for an omission and wrong for a request.
// An unusable pattern is a 1, not a silent run that excludes nothing: a gate
// whose exclude glob has a typo would otherwise judge the tree it was told to
// leave alone and report offences nobody asked about.
#[test]
fn run_with_an_unusable_exclude_pattern_is_an_error() {
    // Arrange & Act
    let outcome = Runner::run(args_from(&[
        "cargo-stern4rust",
        "--manifest-path",
        "Cargo.toml",
        "--package",
        THIS_CRATE,
        "--exclude",
        "fixture/[",
    ]));

    // Assert
    assert!(outcome.is_err());
}

#[test]
fn run_with_the_header_rule_selected_but_no_header_file_is_an_error() {
    // Arrange -- a package with nothing to supply the header. Pointing at this
    // repository would not do: its own stern4rust.toml names a header file, so
    // the rule would run and the test would pass for the wrong reason.
    let path = probe_package("header_rule_with_no_header_file");
    let args = args_from(&[
        "cargo-stern4rust",
        "--manifest-path",
        path.join("Cargo.toml").to_str().expect("manifest path"),
        "--rule",
        "header",
    ]);

    // Act
    let result = Runner::run(args);

    // Assert
    assert!(result.is_err());
}

// Without --header-file the header rule cannot hold, but the structure rule
// needs nothing and still does. A tool that reported "all rules satisfied"
// having checked nothing would be worse than one that says so, which is why the
// registry leaves an unconfigurable rule out rather than registering it silent.
#[test]
fn run_without_a_header_file_still_applies_the_rules_that_need_no_configuration() {
    // Arrange
    let args = args_from(&["cargo-stern4rust", "--manifest-path", "Cargo.toml"]);

    // Act
    let result = Runner::run(args);

    // Assert
    assert!(result.is_ok());
}
