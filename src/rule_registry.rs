// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::reporting::offence::Offence;
use crate::rule::Rule;
use crate::rules::layout::directory_file_count_rule::DirectoryFileCountRule;
use crate::rules::layout::directory_subfolder_count_rule::DirectorySubfolderCountRule;
use crate::rules::layout::module_registry_rule::ModuleRegistryRule;
use crate::rules::layout::paired_test_file_rule::PairedTestFileRule;
use crate::rules::layout::registry_completeness_rule::RegistryCompletenessRule;
use crate::rules::layout::test_file_name_postfix_rule::TestFileNamePostfixRule;
use crate::rules::layout::tests_layout_rule::TestsLayoutRule;
use crate::rules::manifest::spdx_matches_manifest_rule::SpdxMatchesManifestRule;
use crate::rules::manifest::workspace_dependencies_rule::WorkspaceDependenciesRule;
use crate::rules::source::declared_by_name_rule::DeclaredByNameRule;
use crate::rules::source::header_rule::HeaderRule;
use crate::rules::source::imported_paths_rule::ImportedPathsRule;
use crate::rules::source::ordered_imports_rule::OrderedImportsRule;
use crate::rules::source::pure_traits_rule::PureTraitsRule;
use crate::rules::source::readable_source_rule::ReadableSourceRule;
use crate::rules::source::single_implemented_type_rule::SingleImplementedTypeRule;
use crate::rules::source::test_free_source_rule::TestFreeSourceRule;
use crate::rules::testing::arrange_act_assert_rule::ArrangeActAssertRule;
use crate::rules::testing::test_file_structure_rule::TestFileStructureRule;
use crate::rules::testing::test_naming_rule::TestNamingRule;
use crate::rules::testing::tested_public_api_rule::TestedPublicApiRule;
use crate::settings::config::Config;
use crate::settings::rule_selection::RuleSelection;
use crate::source_file::SourceFile;

// The one place that knows which rules exist. Adding a rule is a line here and a
// file under rules/, and nothing else in the tool changes.
//
// A rule with nothing to work from is left out rather than registered and
// silently passing: a run that reports "all rules satisfied" while a rule was
// never configured is worse than one that says which rules it actually applied.
pub struct RuleRegistry {
    rules: Vec<Box<dyn Rule>>,
}

impl RuleRegistry {
    // Every rule this tool has, in report order. The single list: `from_config`
    // narrows it and `known_names` reads its names, so neither can hold an idea
    // of the rule set that the other does not share.
    //
    // readable-source comes first because it is the one rule whose failure
    // explains every other rule's silence on the same file. The header rule is
    // built here even without a header, so that it can still name itself -- it
    // answers `is_configured` with false and `from_config` drops it.
    fn all(config: &Config) -> Vec<Box<dyn Rule>> {
        vec![
            Box::new(ReadableSourceRule::new()),
            Box::new(ArrangeActAssertRule::new()),
            Box::new(DeclaredByNameRule::new()),
            Box::new(DirectoryFileCountRule::new(
                config
                    .max_files_per_directory
                    .unwrap_or(DirectoryFileCountRule::DEFAULT_LIMIT),
            )),
            Box::new(DirectorySubfolderCountRule::new(
                config
                    .max_subfolders_per_directory
                    .unwrap_or(DirectorySubfolderCountRule::DEFAULT_LIMIT),
            )),
            Box::new(ImportedPathsRule::new()),
            Box::new(ModuleRegistryRule::new()),
            Box::new(OrderedImportsRule::new()),
            Box::new(PairedTestFileRule::new()),
            Box::new(PureTraitsRule::new()),
            Box::new(RegistryCompletenessRule::new()),
            Box::new(SingleImplementedTypeRule::new()),
            Box::new(SpdxMatchesManifestRule::new(
                config.manifest_license.clone(),
            )),
            Box::new(TestFileNamePostfixRule::new()),
            Box::new(TestFileStructureRule::new()),
            Box::new(TestFreeSourceRule::new()),
            Box::new(TestNamingRule::new()),
            Box::new(TestedPublicApiRule::new()),
            Box::new(TestsLayoutRule::new()),
            Box::new(WorkspaceDependenciesRule::new(
                config.workspace_dependencies.clone(),
            )),
            Box::new(HeaderRule::new(config.expected_header.clone())),
        ]
    }

    pub fn from_config(config: &Config) -> Self {
        let rules = Self::all(config)
            .into_iter()
            .filter(|rule| rule.is_configured() && config.selection.includes(rule.name()))
            .collect();
        Self { rules }
    }

    pub fn new(rules: Vec<Box<dyn Rule>>) -> Self {
        Self { rules }
    }

    // Every rule this tool has, whether or not this run configured or selected
    // it. Read off the same list `from_config` narrows, so a rule cannot be
    // applied by a default run while `--rule <name>` calls it unknown.
    pub fn known_names() -> Vec<&'static str> {
        Self::all(&Config::default())
            .iter()
            .map(|rule| rule.name())
            .collect()
    }

    // What the switches turned off, which is not the same as what went
    // unregistered: a header rule left out for want of a header file was never
    // deselected by anybody.
    pub fn skipped_names(selection: &RuleSelection) -> Vec<&'static str> {
        Self::known_names()
            .into_iter()
            .filter(|name| !selection.includes(name))
            .collect()
    }

    // Selected, but not registered, because it had nothing to work from. The
    // third state: neither applied nor skipped. Reporting it as skipped would
    // blame the reader for a choice they did not make, and reporting it as
    // nothing at all would let a run check less than it appears to.
    pub fn unconfigured_names(&self, config: &Config) -> Vec<&'static str> {
        self.unconfigured(config)
            .into_iter()
            .map(|(name, _)| name)
            .collect()
    }

    // Each with what it was waiting for, taken from the rule rather than
    // guessed at by the printer. A rule that could not run is built by `all`
    // even though `from_config` dropped it, which is what lets it still answer.
    pub fn unconfigured(&self, config: &Config) -> Vec<(&'static str, &'static str)> {
        let applied = self.names();
        Self::all(config)
            .iter()
            .filter(|rule| {
                config.selection.includes(rule.name()) && !applied.contains(&rule.name())
            })
            .map(|rule| (rule.name(), rule.requirement().unwrap_or("not configured")))
            .collect()
    }

    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    pub fn names(&self) -> Vec<&'static str> {
        self.rules.iter().map(|rule| rule.name()).collect()
    }

    pub fn check(&self, file: &SourceFile) -> Vec<Offence> {
        self.rules
            .iter()
            .flat_map(|rule| rule.check(file))
            .collect()
    }

    // Asked once, after every file has been read, for the rules whose subject is
    // the tree rather than a file in it.
    pub fn check_workspace(&self, files: &[SourceFile]) -> Vec<Offence> {
        self.rules
            .iter()
            .flat_map(|rule| rule.check_workspace(files))
            .collect()
    }
}
