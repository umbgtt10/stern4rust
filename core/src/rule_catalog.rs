// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::finding::model::manifest_dependency::ManifestDependency;
use crate::leading_constants_rewriter::LeadingConstantsRewriter;
use crate::ordered_impl_functions_rewriter::OrderedImplFunctionsRewriter;
use crate::reporting::unconfigured_rule::UnconfiguredRule;
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
use crate::rules::source::distinct_tuple_elements_rule::DistinctTupleElementsRule;
use crate::rules::source::header_rule::HeaderRule;
use crate::rules::source::immutable_returns_rule::ImmutableReturnsRule;
use crate::rules::source::implemented_in_one_file_rule::ImplementedInOneFileRule;
use crate::rules::source::imported_paths_rule::ImportedPathsRule;
use crate::rules::source::injected_time_rule::InjectedTimeRule;
use crate::rules::source::leading_constants_rule::LeadingConstantsRule;
use crate::rules::source::named_subject_rule::NamedSubjectRule;
use crate::rules::source::ordered_impl_functions_rule::OrderedImplFunctionsRule;
use crate::rules::source::ordered_imports_rule::OrderedImportsRule;
use crate::rules::source::owned_functions_rule::OwnedFunctionsRule;
use crate::rules::source::public_or_private_rule::PublicOrPrivateRule;
use crate::rules::source::pure_traits_rule::PureTraitsRule;
use crate::rules::source::readable_source_rule::ReadableSourceRule;
use crate::rules::source::returned_mutations_rule::ReturnedMutationsRule;
use crate::rules::source::shared_constants_rule::SharedConstantsRule;
use crate::rules::source::single_implemented_type_rule::SingleImplementedTypeRule;
use crate::rules::source::test_free_source_rule::TestFreeSourceRule;
use crate::rules::source::typed_errors_rule::TypedErrorsRule;
use crate::rules::testing::arrange_act_assert_rule::ArrangeActAssertRule;
use crate::rules::testing::test_file_structure_rule::TestFileStructureRule;
use crate::rules::testing::test_naming_rule::TestNamingRule;
use crate::rules::testing::tested_public_api_rule::TestedPublicApiRule;
use crate::rules::testing::timeless_tests_rule::TimelessTestsRule;
use crate::rules::types::injected_dependencies_rule::InjectedDependenciesRule;
use crate::rules::types::record_home_rule::RecordHomeRule;
use crate::rules::types::records_and_objects_rule::RecordsAndObjectsRule;
use crate::settings::config::Config;
use crate::settings::rule_selection::RuleSelection;
use crate::source_rewriter::SourceRewriter;
use crate::test_file_rewriter::TestFileRewriter;

// The one place that knows which rules and fixers exist, and builds them for a
// run: the composition root of the rule set. Adding a rule is a line here and a
// file under rules/, and nothing else in the tool changes.
//
// A rule with nothing to work from is left out of the selection rather than
// selected and silently passing: a run that reports "all rules satisfied" while
// a rule was never configured is worse than one that says which rules it
// actually applied. The registry and the rewriter registry take what this
// builds; neither builds its own parts.
pub struct RuleCatalog;

impl RuleCatalog {
    // Every rule this tool has, whether or not this run configured or selected
    // it. Read off the same list `selected` narrows, so a rule cannot be
    // applied by a default run while `--rule <name>` calls it unknown.
    pub fn known_names() -> Vec<&'static str> {
        Self::all(&Config::default())
            .iter()
            .map(|rule| rule.name())
            .collect()
    }

    // The fixers --fix runs, in the order a file is handed through them.
    pub fn rewriters() -> Vec<Box<dyn SourceRewriter>> {
        vec![
            Box::new(TestFileRewriter::new()),
            Box::new(LeadingConstantsRewriter::new()),
            Box::new(OrderedImplFunctionsRewriter::new()),
        ]
    }

    // The rules a run applies: each one selected that has what it needs.
    pub fn selected(config: &Config) -> Vec<Box<dyn Rule>> {
        Self::all(config)
            .into_iter()
            .filter(|rule| rule.is_configured() && config.selection.includes(rule.name()))
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

    // Each with what it was waiting for, taken from the rule rather than
    // guessed at by the printer: selected, but left out of `selected` for want
    // of what it needs, and still built by `all`, which is what lets it answer.
    pub fn unconfigured(config: &Config) -> Vec<UnconfiguredRule> {
        Self::all(config)
            .iter()
            .filter(|rule| config.selection.includes(rule.name()) && !rule.is_configured())
            .map(|rule| UnconfiguredRule {
                name: rule.name(),
                requirement: rule.requirement().unwrap_or("not configured"),
            })
            .collect()
    }

    // Selected, but not registered, because it had nothing to work from. The
    // third state: neither applied nor skipped. Reporting it as skipped would
    // blame the reader for a choice they did not make, and reporting it as
    // nothing at all would let a run check less than it appears to.
    pub fn unconfigured_names(config: &Config) -> Vec<&'static str> {
        Self::unconfigured(config)
            .into_iter()
            .map(|rule| rule.name)
            .collect()
    }

    // Every rule this tool has, in report order. The single list: `from_config`
    // narrows it and `known_names` reads its names, so neither can hold an idea
    // of the rule set that the other does not share.
    //
    // readable-source comes first because it is the one rule whose failure
    // explains every other rule's silence on the same file. The header rule is
    // built here even without a header, so that it can still name itself -- it
    // answers `is_configured` with false and `selected` drops it.
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
            Box::new(DistinctTupleElementsRule::new()),
            Box::new(ImmutableReturnsRule::new()),
            Box::new(ImplementedInOneFileRule::new()),
            Box::new(ImportedPathsRule::new()),
            Box::new(InjectedDependenciesRule::new()),
            Box::new(InjectedTimeRule::new()),
            Box::new(LeadingConstantsRule::new()),
            Box::new(ModuleRegistryRule::new()),
            Box::new(NamedSubjectRule::new()),
            Box::new(OrderedImplFunctionsRule::new()),
            Box::new(OrderedImportsRule::new()),
            Box::new(OwnedFunctionsRule::new()),
            Box::new(PairedTestFileRule::new()),
            Box::new(PublicOrPrivateRule::new()),
            Box::new(PureTraitsRule::new()),
            Box::new(RecordHomeRule::new(config.name_readers.clone())),
            Box::new(RecordsAndObjectsRule::new()),
            Box::new(RegistryCompletenessRule::new()),
            Box::new(ReturnedMutationsRule::new()),
            Box::new(SharedConstantsRule::new(config.name_readers.clone())),
            Box::new(SingleImplementedTypeRule::new()),
            Box::new(SpdxMatchesManifestRule::new(
                config.manifest_license.as_deref().map(str::to_string),
            )),
            Box::new(TestFileNamePostfixRule::new()),
            Box::new(TestFileStructureRule::new()),
            Box::new(TestFreeSourceRule::new()),
            Box::new(TestNamingRule::new()),
            Box::new(TestedPublicApiRule::new()),
            Box::new(TestsLayoutRule::new()),
            Box::new(TimelessTestsRule::new()),
            Box::new(TypedErrorsRule::new()),
            Box::new(WorkspaceDependenciesRule::new(
                config
                    .workspace_dependencies
                    .as_deref()
                    .map(<[ManifestDependency]>::to_vec),
            )),
            Box::new(HeaderRule::new(config.expected_header.to_vec())),
        ]
    }
}
