// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The wiring from an invocation to a verdict, against a workspace held in
// memory: no cargo, no disk, no stdout. What the real ones do is the command
// line's to test.
//
// The verdict is a return value rather than a call to exit, which is the only
// reason any of this is reachable from a test at all. Two outcomes have to stay
// tellable apart: a broken rule is a successful run with a finding, while a run
// that could not judge anything is an Err.

use crate::support::in_memory_baseline_store::InMemoryBaselineStore;
use crate::support::in_memory_config_source::InMemoryConfigSource;
use crate::support::in_memory_source_tree::InMemorySourceTree;
use crate::support::in_memory_source_tree::path_of;
use crate::support::in_memory_workspace_reader::InMemoryWorkspaceReader;
use crate::support::in_memory_workspace_reader::workspace_of;
use crate::support::summary_report_renderer::SummaryReportRenderer;
use stern4rust::reporting::run_outcome::RunOutcome;
use stern4rust::run_failure::RunFailure;
use stern4rust::runner::Runner;
use stern4rust::settings::config_file::ConfigFile;
use stern4rust::settings::invocation::Invocation;
use stern4rust::settings::package_config::PackageConfig;

const HEADER: &str = "/repo/docs/header.txt";
const MISORDERED_TESTS: &str = "#[test]\nfn zeta() {}\n\n#[test]\nfn alpha() {}\n";

fn header_run(rules: &[&str]) -> Invocation {
    Invocation {
        header_file: Some(path_of(HEADER)),
        ..run_of(rules)
    }
}

fn run(
    workspace: &InMemoryWorkspaceReader,
    tree: &InMemorySourceTree,
    config: &InMemoryConfigSource,
    baselines: &InMemoryBaselineStore,
    invocation: &Invocation,
) -> Result<(RunOutcome, String), RunFailure> {
    Runner::new(workspace, tree, config, baselines, &SummaryReportRenderer).run(invocation)
}

fn run_of(rules: &[&str]) -> Invocation {
    Invocation {
        manifest_directory: path_of("/repo"),
        rules: rules.iter().map(|rule| (*rule).to_string()).collect(),
        ..Invocation::default()
    }
}

fn tree() -> InMemorySourceTree {
    InMemorySourceTree::new()
        .with_file(HEADER, "// H\n")
        .with_file("/repo/alpha/src/lib.rs", "pub fn f() {}\n")
}

fn workspace() -> InMemoryWorkspaceReader {
    InMemoryWorkspaceReader::new(workspace_of("/repo", &["alpha"]))
}

#[test]
fn run_over_a_file_breaking_a_rule_reports_it_and_breaks_the_rule() {
    // Arrange
    let invocation = header_run(&["header"]);

    // Act
    let (outcome, report) = run(
        &workspace(),
        &tree(),
        &InMemoryConfigSource::absent(),
        &InMemoryBaselineStore::new(),
        &invocation,
    )
    .expect("a run");

    // Assert
    assert_eq!(outcome, RunOutcome::RulesBroken);
    assert_eq!(
        report,
        "scanned=1 fixed=0 baselined=0 stale=0 applied=1 config=none\nsrc/lib.rs:1 header\n"
    );
}

#[test]
fn run_over_a_file_keeping_the_rule_is_clean() {
    // Arrange
    let tree = tree().with_file("/repo/alpha/src/lib.rs", "// H\npub fn f() {}\n");

    // Act
    let (outcome, report) = run(
        &workspace(),
        &tree,
        &InMemoryConfigSource::absent(),
        &InMemoryBaselineStore::new(),
        &header_run(&["header"]),
    )
    .expect("a run");

    // Assert
    assert_eq!(outcome, RunOutcome::Clean);
    assert_eq!(
        report,
        "scanned=1 fixed=0 baselined=0 stale=0 applied=1 config=none\n"
    );
}

#[test]
fn run_with_a_config_file_names_it_in_the_report() {
    // Arrange
    let config = InMemoryConfigSource::holding(ConfigFile::new(
        None,
        None,
        PackageConfig::default().with_header_file(path_of("docs/header.txt")),
        Default::default(),
    ));

    // Act
    let (outcome, report) = run(
        &workspace(),
        &tree(),
        &config,
        &InMemoryBaselineStore::new(),
        &run_of(&["header"]),
    )
    .expect("a run");

    // Assert
    assert_eq!(outcome, RunOutcome::RulesBroken);
    assert_eq!(
        report,
        "scanned=1 fixed=0 baselined=0 stale=0 applied=1 config=/repo/stern4rust.toml\n\
         src/lib.rs:1 header\n"
    );
}

#[test]
fn run_with_a_config_that_cannot_be_read_fails_naming_it() {
    // Arrange
    let config = InMemoryConfigSource::unreadable();

    // Act
    let failure = run(
        &workspace(),
        &tree(),
        &config,
        &InMemoryBaselineStore::new(),
        &header_run(&["header"]),
    )
    .expect_err("an unreadable config");

    // Assert
    assert_eq!(
        failure.to_string(),
        "/repo/stern4rust.toml could not be read"
    );
}

#[test]
fn run_with_a_workspace_that_cannot_be_read_fails() {
    // Arrange
    let workspace = InMemoryWorkspaceReader::failing();

    // Act
    let failure = run(
        &workspace,
        &tree(),
        &InMemoryConfigSource::absent(),
        &InMemoryBaselineStore::new(),
        &header_run(&["header"]),
    )
    .expect_err("an unreadable workspace");

    // Assert
    assert!(matches!(failure, RunFailure::Workspace(_)));
    assert_eq!(failure.to_string(), "not held");
}

#[test]
fn run_with_a_written_baseline_forgives_what_it_recorded() {
    // Arrange
    let baselines = InMemoryBaselineStore::new();
    let recording = Invocation {
        write_baseline: true,
        ..header_run(&["header"])
    };
    let judging = Invocation {
        baseline: Some(path_of("/repo/stern4rust-baseline.json")),
        ..header_run(&["header"])
    };
    let config = InMemoryConfigSource::absent();

    // Act
    let (recorded, said) =
        run(&workspace(), &tree(), &config, &baselines, &recording).expect("a recording");

    // Assert
    assert_eq!(recorded, RunOutcome::Clean);
    assert_eq!(
        said,
        "stern4rust wrote 1 offence(s) to /repo/stern4rust-baseline.json\n"
    );

    // Act
    let (judged, report) =
        run(&workspace(), &tree(), &config, &baselines, &judging).expect("a judged run");

    // Assert
    assert_eq!(judged, RunOutcome::Clean);
    assert_eq!(
        report,
        "scanned=1 fixed=0 baselined=1 stale=0 applied=1 config=none\n"
    );
}

#[test]
fn run_with_an_exclusion_covering_every_file_scans_nothing() {
    // Arrange
    let invocation = Invocation {
        excludes: vec![String::from("**")],
        ..header_run(&["header"])
    };

    // Act
    let (outcome, report) = run(
        &workspace(),
        &tree(),
        &InMemoryConfigSource::absent(),
        &InMemoryBaselineStore::new(),
        &invocation,
    )
    .expect("a run");

    // Assert
    assert_eq!(outcome, RunOutcome::Clean);
    assert_eq!(
        report,
        "scanned=0 fixed=0 baselined=0 stale=0 applied=1 config=none\n"
    );
}

#[test]
fn run_with_an_unknown_package_fails_naming_it() {
    // Arrange
    let invocation = Invocation {
        packages: vec![String::from("gamma")],
        ..header_run(&["header"])
    };

    // Act
    let failure = run(
        &workspace(),
        &tree(),
        &InMemoryConfigSource::absent(),
        &InMemoryBaselineStore::new(),
        &invocation,
    )
    .expect_err("an unknown package");

    // Assert
    assert_eq!(
        failure.to_string(),
        "package `gamma` is not in this workspace"
    );
}

#[test]
fn run_with_an_unknown_rule_name_fails_naming_it() {
    // Arrange
    let invocation = header_run(&["no-such-rule"]);

    // Act
    let failure = run(
        &workspace(),
        &tree(),
        &InMemoryConfigSource::absent(),
        &InMemoryBaselineStore::new(),
        &invocation,
    )
    .expect_err("an unknown rule");

    // Assert
    assert!(
        failure
            .to_string()
            .starts_with("unknown rule name(s): no-such-rule -- the rules are: readable-source,"),
        "{failure}"
    );
}

#[test]
fn run_with_an_unreadable_file_reports_it_against_readable_source() {
    // Arrange
    let tree = tree().with_unreadable_file("/repo/alpha/src/lib.rs");

    // Act
    let (outcome, report) = run(
        &workspace(),
        &tree,
        &InMemoryConfigSource::absent(),
        &InMemoryBaselineStore::new(),
        &run_of(&["readable-source"]),
    )
    .expect("a run");

    // Assert
    assert_eq!(outcome, RunOutcome::RulesBroken);
    assert_eq!(
        report,
        "scanned=1 fixed=0 baselined=0 stale=0 applied=1 config=none\nsrc/lib.rs:1 readable-source\n"
    );
}

#[test]
fn run_with_an_unreadable_header_file_fails_naming_it() {
    // Arrange
    let tree = InMemorySourceTree::new().with_file("/repo/alpha/src/lib.rs", "pub fn f() {}\n");

    // Act
    let failure = run(
        &workspace(),
        &tree,
        &InMemoryConfigSource::absent(),
        &InMemoryBaselineStore::new(),
        &header_run(&["header"]),
    )
    .expect_err("no header file");

    // Assert
    assert_eq!(
        failure.to_string(),
        "failed to read header file /repo/docs/header.txt"
    );
}

#[test]
fn run_with_every_rule_skipped_fails_for_want_of_a_rule() {
    // Arrange
    let skipped: Vec<String> = Runner::new(
        &workspace(),
        &tree(),
        &InMemoryConfigSource::absent(),
        &InMemoryBaselineStore::new(),
        &SummaryReportRenderer,
    )
    .run(&Invocation {
        list_rules: true,
        ..run_of(&[])
    })
    .expect("a listing")
    .1
    .lines()
    .map(str::to_string)
    .collect();
    let invocation = Invocation {
        skipped_rules: skipped,
        ..run_of(&[])
    };

    // Act
    let failure = run(
        &workspace(),
        &tree(),
        &InMemoryConfigSource::absent(),
        &InMemoryBaselineStore::new(),
        &invocation,
    )
    .expect_err("no rule left");

    // Assert
    assert!(matches!(failure, RunFailure::NoRules));
    assert_eq!(
        failure.to_string(),
        "no rules are configured, so nothing would be checked -- pass --header-file to enable \
         the header rule"
    );
}

#[test]
fn run_with_fix_writes_the_repaired_file_back_and_counts_it() {
    // Arrange
    let tree = tree().with_file("/repo/alpha/tests/a_tests.rs", MISORDERED_TESTS);
    let invocation = Invocation {
        fix: true,
        ..run_of(&["test-file-structure"])
    };

    // Act
    let (outcome, report) = run(
        &workspace(),
        &tree,
        &InMemoryConfigSource::absent(),
        &InMemoryBaselineStore::new(),
        &invocation,
    )
    .expect("a run");

    // Assert
    assert_eq!(outcome, RunOutcome::Clean);
    assert_eq!(
        report,
        "scanned=2 fixed=1 baselined=0 stale=0 applied=1 config=none\n"
    );
    assert_eq!(tree.written(), [path_of("/repo/alpha/tests/a_tests.rs")]);
    assert_eq!(
        tree.contents_of("/repo/alpha/tests/a_tests.rs"),
        Some(String::from(
            "#[test]\nfn alpha() {}\n\n#[test]\nfn zeta() {}\n"
        ))
    );
}

#[test]
fn run_with_rules_lists_every_rule_the_registry_holds() {
    // Arrange
    let invocation = Invocation {
        list_rules: true,
        ..run_of(&[])
    };

    // Act
    let (outcome, listing) = run(
        &InMemoryWorkspaceReader::failing(),
        &InMemorySourceTree::new(),
        &InMemoryConfigSource::unreadable(),
        &InMemoryBaselineStore::new(),
        &invocation,
    )
    .expect("a listing");

    // Assert
    assert_eq!(outcome, RunOutcome::Clean);
    assert_eq!(listing.lines().count(), 38);
    assert!(listing.starts_with("readable-source\n"), "{listing}");
    assert!(listing.ends_with("header\n"), "{listing}");
}

#[test]
fn run_with_the_header_rule_selected_but_no_header_file_fails() {
    // Arrange
    let invocation = run_of(&["header"]);

    // Act
    let failure = run(
        &workspace(),
        &tree(),
        &InMemoryConfigSource::absent(),
        &InMemoryBaselineStore::new(),
        &invocation,
    )
    .expect_err("no header");

    // Assert
    assert_eq!(
        failure.to_string(),
        "--rule header needs --header-file, otherwise the run would apply no rules at all"
    );
}
