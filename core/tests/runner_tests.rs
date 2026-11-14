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

use crate::support::capturing_report_renderer::CapturingReportRenderer;
use crate::support::in_memory_baseline_store::InMemoryBaselineStore;
use crate::support::in_memory_config_source::InMemoryConfigSource;
use crate::support::in_memory_source_tree::InMemorySourceTree;
use crate::support::in_memory_source_tree::path_of;
use crate::support::in_memory_workspace_reader::InMemoryWorkspaceReader;
use crate::support::in_memory_workspace_reader::workspace_of;
use crate::support::summary_report_renderer::SummaryReportRenderer;
use std::collections::BTreeMap;
use stern4rust::finding::model::manifest_dependency::ManifestDependency;
use stern4rust::reporting::offence_threshold::OffenceThreshold;
use stern4rust::reporting::run_outcome::RunOutcome;
use stern4rust::reporting::run_report::RunReport;
use stern4rust::run_failure::RunFailure;
use stern4rust::runner::Runner;
use stern4rust::settings::config_file::ConfigFile;
use stern4rust::settings::invocation::Invocation;
use stern4rust::settings::package_config::PackageConfig;

const HEADER: &str = "/repo/docs/header.txt";
const MISORDERED_TESTS: &str = "#[test]\nfn zeta() {}\n\n#[test]\nfn alpha() {}\n";

// The rules each offence of a report broke, in report order.
fn broken(report: &RunReport) -> Vec<&'static str> {
    report
        .offences
        .iter()
        .map(|offence| offence.rule())
        .collect()
}

fn header_run(rules: &[&str]) -> Invocation {
    Invocation {
        header_file: Some(path_of(HEADER)),
        ..run_of(rules)
    }
}

// The report of a run, kept whole rather than summarised.
fn report_of(
    workspace: &InMemoryWorkspaceReader,
    tree: &InMemorySourceTree,
    config: &InMemoryConfigSource,
    invocation: &Invocation,
) -> RunReport {
    let renderer = CapturingReportRenderer::default();
    Runner::new(
        workspace,
        tree,
        config,
        &InMemoryBaselineStore::new(),
        &renderer,
    )
    .run(invocation)
    .expect("a run");
    renderer.captured()
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

// A workspace of alpha and beta, alpha declaring these dependencies.
fn workspace_declaring(declared: Vec<ManifestDependency>) -> InMemoryWorkspaceReader {
    let mut description = workspace_of("/repo", &["alpha", "beta"]);
    description.packages[0].declared = declared;
    InMemoryWorkspaceReader::new(description)
}

#[test]
fn run_hands_each_package_its_manifest_licence() {
    // Arrange
    let tree = InMemorySourceTree::new().with_file(
        "/repo/alpha/src/lib.rs",
        "// SPDX-License-Identifier: Apache-2.0\npub fn f() {}\n",
    );

    // Act
    let report = report_of(
        &workspace(),
        &tree,
        &InMemoryConfigSource::absent(),
        &run_of(&["spdx-matches-manifest"]),
    );

    // Assert
    assert_eq!(broken(&report), vec!["spdx-matches-manifest"]);
}

#[test]
fn run_hands_each_package_the_dependencies_its_manifest_declares() {
    // Arrange
    let workspace = workspace_declaring(vec![
        ManifestDependency::new("alpha/Cargo.toml", "serde", "dependencies", true),
        ManifestDependency::new("alpha/Cargo.toml", "serde", "dev-dependencies", true),
    ]);

    // Act
    let report = report_of(
        &workspace,
        &tree(),
        &InMemoryConfigSource::absent(),
        &run_of(&["distinct-dev-dependencies"]),
    );

    // Assert
    assert_eq!(broken(&report), vec!["distinct-dev-dependencies"]);
}

#[test]
fn run_hands_each_package_the_workspace_dependencies_its_manifest_declares() {
    // Arrange
    let workspace = workspace_declaring(vec![ManifestDependency::new(
        "alpha/Cargo.toml",
        "serde",
        "dependencies",
        false,
    )]);

    // Act
    let report = report_of(
        &workspace,
        &tree(),
        &InMemoryConfigSource::absent(),
        &run_of(&["workspace-dependencies"]),
    );

    // Assert
    assert_eq!(broken(&report), vec!["workspace-dependencies"]);
}

#[test]
fn run_lists_each_packages_rules_in_its_roster() {
    // Arrange & Act
    let report = report_of(
        &workspace(),
        &tree(),
        &InMemoryConfigSource::absent(),
        &run_of(&["arrange-act-assert"]),
    );

    // Assert
    assert_eq!(report.rosters.len(), 1);
    assert_eq!(report.rosters[0].package(), "alpha");
    assert_eq!(report.rosters[0].applied(), ["arrange-act-assert"]);
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

// A constant beta reads is read all the same when the run walks alpha alone:
// with alpha's own reader, it has two and sits in a constants file.
#[test]
fn run_scoped_to_one_package_reads_a_constants_readers_across_every_member() {
    // Arrange
    let tree = InMemorySourceTree::new()
        .with_file("/repo/alpha/src/limits.rs", "pub const LIMIT: u8 = 1;\n")
        .with_file("/repo/alpha/src/a.rs", "fn a() -> u8 {\n    LIMIT\n}\n")
        .with_file("/repo/beta/src/b.rs", "fn b() -> u8 {\n    LIMIT\n}\n");
    let invocation = Invocation {
        packages: vec![String::from("alpha")],
        ..run_of(&["shared-constants"])
    };

    // Act
    let report = report_of(
        &workspace_declaring(Vec::new()),
        &tree,
        &InMemoryConfigSource::absent(),
        &invocation,
    );

    // Assert
    assert!(broken(&report).is_empty(), "{:?}", report.offences);
}

#[test]
fn run_states_a_rule_a_package_section_skips_as_not_applied() {
    // Arrange
    let config = InMemoryConfigSource::holding(ConfigFile::new(
        None,
        None,
        PackageConfig::default(),
        BTreeMap::from([(
            String::from("alpha"),
            PackageConfig::default().with_skip(vec![String::from("test-naming")]),
        )]),
    ));

    // Act
    let report = report_of(
        &workspace(),
        &tree(),
        &config,
        &run_of(&["arrange-act-assert", "test-naming"]),
    );

    // Assert
    assert_eq!(report.applied, vec![String::from("arrange-act-assert")]);
    assert!(report.skipped.contains(&String::from("test-naming")));
}

#[test]
fn run_sums_an_exclusions_counts_over_every_package() {
    // Arrange
    let tree = InMemorySourceTree::new()
        .with_file("/repo/alpha/src/generated/x.rs", "pub fn x() {}\n")
        .with_file("/repo/beta/src/generated/y.rs", "pub fn y() {}\n");
    let invocation = Invocation {
        excludes: vec![String::from("**/generated/**")],
        ..run_of(&["arrange-act-assert"])
    };

    // Act
    let report = report_of(
        &workspace_declaring(Vec::new()),
        &tree,
        &InMemoryConfigSource::absent(),
        &invocation,
    );

    // Assert
    assert_eq!(
        report.exclusions,
        vec![(String::from("**/generated/**"), 2)]
    );
}

// A package section's rules stand in for the root's; a section that states
// none takes the root's.
#[test]
fn run_takes_a_sections_rules_over_the_roots_and_the_roots_where_it_states_none() {
    // Arrange
    let config = InMemoryConfigSource::holding(ConfigFile::new(
        None,
        None,
        PackageConfig::new(
            None,
            None,
            None,
            vec![String::from("test-naming")],
            None,
            Vec::new(),
        ),
        BTreeMap::from([
            (
                String::from("alpha"),
                PackageConfig::new(
                    None,
                    None,
                    None,
                    vec![String::from("arrange-act-assert")],
                    None,
                    Vec::new(),
                ),
            ),
            (
                String::from("beta"),
                PackageConfig::new(
                    None,
                    None,
                    None,
                    Vec::new(),
                    None,
                    vec![String::from("**/none/**")],
                ),
            ),
        ]),
    ));
    let invocation = Invocation {
        manifest_directory: path_of("/repo"),
        ..Invocation::default()
    };

    // Act
    let report = report_of(
        &workspace_declaring(Vec::new()),
        &tree(),
        &config,
        &invocation,
    );

    // Assert
    let applied: Vec<(&str, &[String])> = report
        .rosters
        .iter()
        .map(|roster| (roster.package(), roster.applied()))
        .collect();
    assert_eq!(
        applied,
        vec![
            ("alpha", &[String::from("arrange-act-assert")][..]),
            ("beta", &[String::from("test-naming")][..]),
        ]
    );
}

#[test]
fn run_takes_the_directory_limits_from_the_config_file() {
    // Arrange
    let config = InMemoryConfigSource::holding(ConfigFile::new(
        None,
        None,
        PackageConfig::new(None, Some(1), Some(1), Vec::new(), None, Vec::new()),
        BTreeMap::new(),
    ));
    let tree = InMemorySourceTree::new()
        .with_file("/repo/alpha/src/lib.rs", "pub fn f() {}\n")
        .with_file("/repo/alpha/src/other.rs", "pub fn g() {}\n")
        .with_file("/repo/alpha/src/third.rs", "pub fn h() {}\n")
        .with_file("/repo/alpha/src/x/a.rs", "pub fn a() {}\n")
        .with_file("/repo/alpha/src/y/b.rs", "pub fn b() {}\n");

    // Act
    let report = report_of(
        &workspace(),
        &tree,
        &config,
        &run_of(&["directory-file-count", "directory-subfolder-count"]),
    );

    // Assert
    let mut rules = broken(&report);
    rules.sort_unstable();
    rules.dedup();
    assert_eq!(
        rules,
        vec!["directory-file-count", "directory-subfolder-count"]
    );
}

#[test]
fn run_takes_the_offence_threshold_from_the_config_file() {
    // Arrange
    let config = InMemoryConfigSource::holding(ConfigFile::new(
        None,
        Some(5),
        PackageConfig::default(),
        BTreeMap::new(),
    ));

    // Act
    let report = report_of(
        &workspace(),
        &tree(),
        &config,
        &run_of(&["arrange-act-assert"]),
    );

    // Assert
    assert_eq!(report.threshold, OffenceThreshold::new(5));
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
    assert_eq!(listing.lines().count(), 39);
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
