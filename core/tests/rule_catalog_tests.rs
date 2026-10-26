// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The one place that knows which rules and fixers exist, built for a run.
//
// The property worth stating is that a rule with nothing to work from is left
// out of the selection rather than selected and silently passing. A run that
// reports "all rules satisfied" while a rule was never configured is worse than
// one that says so, because the green is indistinguishable from a real pass.

use stern4rust::reporting::unconfigured_rule::UnconfiguredRule;
use stern4rust::rewriter_registry::RewriterRegistry;
use stern4rust::rule::Rule;
use stern4rust::rule_catalog::RuleCatalog;
use stern4rust::settings::config::Config;
use stern4rust::settings::rule_selection::RuleSelection;
use stern4rust::source_file::SourceFile;

// Everything a run can be given: a header to compare against and a manifest
// that names a licence. Two rules need one each, and both are dropped without.
fn fully_configured(lines: &[&str]) -> Config {
    Config {
        expected_header: lines.iter().map(|line| (*line).to_string()).collect(),
        manifest_license: Some("MIT".to_string()),
        ..Config::default()
    }
}

fn names(rules: &[Box<dyn Rule>]) -> Vec<&'static str> {
    rules.iter().map(|rule| rule.name()).collect()
}

// Built by asking each rule its own name, so the list cannot drift from the
// rules themselves -- which is what the switches validate against.
#[test]
fn known_names_lists_every_rule_the_tool_has() {
    // Arrange & Act
    let known = RuleCatalog::known_names();

    // Assert
    assert_eq!(
        known,
        [
            "readable-source",
            "arrange-act-assert",
            "declared-by-name",
            "directory-file-count",
            "directory-subfolder-count",
            "distinct-dev-dependencies",
            "distinct-tuple-elements",
            "immutable-returns",
            "implemented-in-one-file",
            "imported-paths",
            "injected-dependencies",
            "injected-time",
            "leading-constants",
            "module-registry",
            "named-subject",
            "ordered-impl-functions",
            "ordered-imports",
            "owned-functions",
            "paired-test-file",
            "public-or-private",
            "pure-traits",
            "record-home",
            "records-and-objects",
            "registry-completeness",
            "returned-mutations",
            "shared-constants",
            "single-implemented-type",
            "spdx-matches-manifest",
            "test-file-name-postfix",
            "test-file-structure",
            "test-free-source",
            "test-naming",
            "tested-public-api",
            "tests-layout",
            "timeless-tests",
            "typed-errors",
            "workspace-dependencies",
            "header"
        ]
    );
}

// The regression this closes. known_names once kept its own list of rules
// beside the one from_config builds, so a rule added to only one of them was
// applied by a default run while `--rule <name>` rejected it as unknown. Both
// now read the same list, and this fails if they stop doing so.
#[test]
fn known_names_matches_what_a_fully_configured_run_applies() {
    // Arrange
    let config = fully_configured(&["// Copyright"]);

    // Act
    let applied = names(&RuleCatalog::selected(&config));

    // Assert
    assert_eq!(applied, RuleCatalog::known_names());
}

#[test]
fn rewriters_reorder_an_impl_where_ordered_impl_functions_applies() {
    // Arrange
    let file = SourceFile::new(
        "src/store.rs",
        "impl Store {\n    fn b() {}\n\n    fn a() {}\n}\n",
    );

    // Act
    let rewritten =
        RewriterRegistry::new(RuleCatalog::rewriters()).rewrite(&file, &["ordered-impl-functions"]);

    // Assert
    assert_eq!(
        rewritten.as_deref(),
        Some("impl Store {\n    fn a() {}\n\n    fn b() {}\n}\n")
    );
}

#[test]
fn rewriters_repair_a_test_file_where_test_file_structure_applies() {
    // Arrange
    let file = SourceFile::new(
        "tests/widget_tests.rs",
        "#[test]\nfn zeta() {}\n\n#[test]\nfn alpha() {}\n",
    );

    // Act
    let rewritten =
        RewriterRegistry::new(RuleCatalog::rewriters()).rewrite(&file, &["test-file-structure"]);

    // Assert
    assert_eq!(
        rewritten.as_deref(),
        Some("#[test]\nfn alpha() {}\n\n#[test]\nfn zeta() {}\n")
    );
}

// The header rule cannot hold until it is told what the header says, so it joins
// the set only once there is one to compare against.
#[test]
fn selected_with_a_header_keeps_the_header_rule_alongside_the_others() {
    // Arrange & Act
    let selected = names(&RuleCatalog::selected(&fully_configured(&["// header"])));

    // Assert
    assert_eq!(
        selected,
        [
            "readable-source",
            "arrange-act-assert",
            "declared-by-name",
            "directory-file-count",
            "directory-subfolder-count",
            "distinct-dev-dependencies",
            "distinct-tuple-elements",
            "immutable-returns",
            "implemented-in-one-file",
            "imported-paths",
            "injected-dependencies",
            "injected-time",
            "leading-constants",
            "module-registry",
            "named-subject",
            "ordered-impl-functions",
            "ordered-imports",
            "owned-functions",
            "paired-test-file",
            "public-or-private",
            "pure-traits",
            "record-home",
            "records-and-objects",
            "registry-completeness",
            "returned-mutations",
            "shared-constants",
            "single-implemented-type",
            "spdx-matches-manifest",
            "test-file-name-postfix",
            "test-file-structure",
            "test-free-source",
            "test-naming",
            "tested-public-api",
            "tests-layout",
            "timeless-tests",
            "typed-errors",
            "workspace-dependencies",
            "header"
        ]
    );
}

// Naming one rule is what lets a codebase facing hundreds of offences gate on
// something today rather than on nothing.
#[test]
fn selected_with_a_selection_keeps_only_the_selected_rules() {
    // Arrange
    let config = Config {
        selection: RuleSelection::new(vec!["tests-layout".to_string()], Vec::new()),
        ..Config::default()
    };

    // Act
    let selected = names(&RuleCatalog::selected(&config));

    // Assert
    assert_eq!(selected, ["tests-layout"]);
}

#[test]
fn selected_with_a_skip_leaves_that_rule_out() {
    // Arrange
    let config = Config {
        selection: RuleSelection::new(Vec::new(), vec!["test-file-structure".to_string()]),
        ..Config::default()
    };

    // Act
    let selected = names(&RuleCatalog::selected(&config));

    // Assert
    assert_eq!(
        selected,
        [
            "readable-source",
            "arrange-act-assert",
            "declared-by-name",
            "directory-file-count",
            "directory-subfolder-count",
            "distinct-dev-dependencies",
            "distinct-tuple-elements",
            "immutable-returns",
            "implemented-in-one-file",
            "imported-paths",
            "injected-dependencies",
            "injected-time",
            "leading-constants",
            "module-registry",
            "named-subject",
            "ordered-impl-functions",
            "ordered-imports",
            "owned-functions",
            "paired-test-file",
            "public-or-private",
            "pure-traits",
            "record-home",
            "records-and-objects",
            "registry-completeness",
            "returned-mutations",
            "shared-constants",
            "single-implemented-type",
            "test-file-name-postfix",
            "test-free-source",
            "test-naming",
            "tested-public-api",
            "tests-layout",
            "timeless-tests",
            "typed-errors",
            "workspace-dependencies"
        ]
    );
}

// The structure rule needs nothing configured, so it holds from the first run.
// A tool that does nothing until it is given a flag is a tool nobody switches on.
#[test]
fn selected_without_a_header_keeps_the_rules_that_need_no_configuration() {
    // Arrange & Act
    let selected = names(&RuleCatalog::selected(&Config::default()));

    // Assert
    assert_eq!(
        selected,
        [
            "readable-source",
            "arrange-act-assert",
            "declared-by-name",
            "directory-file-count",
            "directory-subfolder-count",
            "distinct-dev-dependencies",
            "distinct-tuple-elements",
            "immutable-returns",
            "implemented-in-one-file",
            "imported-paths",
            "injected-dependencies",
            "injected-time",
            "leading-constants",
            "module-registry",
            "named-subject",
            "ordered-impl-functions",
            "ordered-imports",
            "owned-functions",
            "paired-test-file",
            "public-or-private",
            "pure-traits",
            "record-home",
            "records-and-objects",
            "registry-completeness",
            "returned-mutations",
            "shared-constants",
            "single-implemented-type",
            "test-file-name-postfix",
            "test-file-structure",
            "test-free-source",
            "test-naming",
            "tested-public-api",
            "tests-layout",
            "timeless-tests",
            "typed-errors",
            "workspace-dependencies"
        ]
    );
}

// Not the same as what went unregistered: a header rule left out for want of a
// header file was never deselected by anybody, and saying it was would blame
// the user for an omission they did not make.
#[test]
fn skipped_names_lists_what_the_switches_turned_off() {
    // Arrange
    let selection = RuleSelection::new(Vec::new(), vec!["tests-layout".to_string()]);

    // Act
    let skipped = RuleCatalog::skipped_names(&selection);

    // Assert
    assert_eq!(skipped, ["tests-layout"]);
}

#[test]
fn unconfigured_names_does_not_list_a_skipped_rule() {
    // Arrange
    // A licence, so that the header rule is the only one that could be
    // unconfigured -- and it is the one being skipped.
    let config = Config {
        selection: RuleSelection::new(Vec::new(), vec!["header".to_string()]),
        manifest_license: Some("MIT".to_string()),
        ..Config::default()
    };

    // Act
    let unconfigured = RuleCatalog::unconfigured_names(&config);

    // Assert
    assert!(unconfigured.is_empty(), "got {unconfigured:?}");
}

// The requirement comes from the rule, so the report can say what to pass
// rather than only that something is unset.
#[test]
fn unconfigured_names_each_rule_with_what_it_is_waiting_for() {
    // Arrange & Act
    let unconfigured = RuleCatalog::unconfigured(&Config::default());

    // Assert
    assert_eq!(
        unconfigured,
        [
            UnconfiguredRule {
                name: "spdx-matches-manifest",
                requirement: "needs a `license` field in Cargo.toml"
            },
            UnconfiguredRule {
                name: "header",
                requirement: "needs --header-file"
            }
        ]
    );
}

// A rule dropped for want of configuration is not a rule anybody skipped, and
// reporting it as skipped would blame the user for a choice they did not make.
// Reporting it as nothing at all is worse: the run silently checks less than it
// appears to.
#[test]
fn unconfigured_names_lists_a_rule_that_could_not_run() {
    // Arrange & Act
    let unconfigured = RuleCatalog::unconfigured_names(&Config::default());

    // Assert
    assert_eq!(unconfigured, ["spdx-matches-manifest", "header"]);
}
