// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::finding::model::manifest_dependency::ManifestDependency;
use crate::finding::model::name_readers::NameReaders;
use crate::finding::parsing::read_names::ReadNames;
use crate::reporting::offence::Offence;
use crate::reporting::offence_threshold::OffenceThreshold;
use crate::reporting::package_roster::PackageRoster;
use crate::reporting::run_outcome::RunOutcome;
use crate::reporting::run_report::RunReport;
use crate::rewriter_registry::RewriterRegistry;
use crate::rule_catalog::RuleCatalog;
use crate::rule_registry::RuleRegistry;
use crate::rules::source::header_rule::HeaderRule;
use crate::settings::adoption::baseline::Baseline;
use crate::settings::adoption::baseline_outcome::BaselineOutcome;
use crate::settings::adoption::exclusion_outcome::ExclusionOutcome;
use crate::settings::adoption::exclusion_set::ExclusionSet;
use crate::settings::config::Config;
use crate::settings::config_file::ConfigFile;
use crate::settings::header_source::HeaderSource;
use crate::settings::invocation::Invocation;
use crate::settings::manifest_resolver::ManifestResolver;
use crate::settings::package_config::PackageConfig;
use crate::settings::package_sections::PackageSections;
use crate::settings::rule_selection::RuleSelection;
use crate::settings::scanned_package::ScannedPackage;
use crate::source_file::SourceFile;
use crate::source_path::SourcePath;
use crate::source_reader::SourceReader;
use crate::traits::baseline_store::BaselineStore;
use crate::traits::config_source::ConfigSource;
use crate::traits::report_renderer::ReportRenderer;
use crate::traits::source_tree::SourceTree;
use crate::traits::workspace_reader::WorkspaceReader;
use crate::workspace_sources::WorkspaceSources;
use anyhow::Result;
use std::collections::HashSet;

// What a scan did, as opposed to what it found.
//
// The offences are the verdict; these are the work behind it. They travelled to
// the report as separate arguments until the per-package rosters arrived and
// took the count past the point where a reader could tell one from another.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ScanTotals {
    pub files_scanned: usize,
    pub fixed: usize,
}

// Exit codes are the whole contract with a gate script:
//
//   0  every rule satisfied
//   1  could not run -- returned as an Err and turned into 1 by main
//   2  at least one rule broken
//
// 2 is kept distinct from 1 on purpose. A script that treats every non-zero code
// alike cannot tell "your code has a problem" from "I could not read your code",
// and the second one silently passing is how a gate stops meaning anything.
//
// The line between the two is what can still be enumerated. A bad manifest or an
// unknown package is a 1: without it there is no list of files to judge. A single
// unreadable file is a 2, reported against readable-source like any other
// finding -- it is a fact about the tree, and aborting on it would hide every
// offence already found in every other file.
pub struct Runner<'a> {
    workspace_reader: &'a dyn WorkspaceReader,
    source_tree: &'a dyn SourceTree,
    config_source: &'a dyn ConfigSource,
    baseline_store: &'a dyn BaselineStore,
    renderer: &'a dyn ReportRenderer,
}

impl<'a> Runner<'a> {
    pub const BASELINE_NAME: &'static str = "stern4rust-baseline.json";

    pub fn new(
        workspace_reader: &'a dyn WorkspaceReader,
        source_tree: &'a dyn SourceTree,
        config_source: &'a dyn ConfigSource,
        baseline_store: &'a dyn BaselineStore,
        renderer: &'a dyn ReportRenderer,
    ) -> Self {
        Self {
            workspace_reader,
            source_tree,
            config_source,
            baseline_store,
            renderer,
        }
    }

    // The run, handing back what it found and what it has to say. Printing it
    // is the caller's.
    //
    // Both per-package bugs left the run perfectly Ok while the report
    // contradicted itself, so a test asserting on the outcome could not see
    // either. This is the seam that lets one assert on what was said.
    pub fn run(&self, invocation: &Invocation) -> Result<(RunOutcome, String)> {
        // Before anything is read. The listing answers from the registry alone,
        // so it works in a checkout with no manifest worth reading and cannot
        // fail the way a run can.
        if invocation.list_rules {
            return Ok((RunOutcome::Clean, self.rule_listing()));
        }
        let directory = &invocation.manifest_directory;
        let sections = PackageSections::load(self.config_source, directory)?;
        let config = self.config_from(invocation, None)?;
        Self::validate_selection(&config)?;
        let workspace = self.workspace_reader.describe()?;
        let workspace_dependencies = ManifestResolver::workspace_dependencies(&workspace);
        let declared_dependencies = ManifestResolver::declared_dependencies(&workspace);
        let config = Config {
            workspace_dependencies,
            declared_dependencies,
            ..config
        };
        let packages = ManifestResolver::packages(&workspace, &config)?;
        let names = ManifestResolver::workspace_package_names(&workspace);
        sections.validate(&names.iter().map(String::as_str).collect::<Vec<_>>())?;
        // What the report answers for. A rule that stood down for any package
        // did not apply to this run, so the licence stated here is the one every
        // scanned package agrees on and nothing otherwise. Checking is per
        // package; only the summary is aggregate, and it understates rather than
        // overstates -- see
        // [ADR-PerPackageConfiguration](../docs/ADRs/ADR-PerPackageConfiguration.md),
        // where the per-package report is the piece still to come.
        let selection = config.selection.also_skipping(
            &sections.skipped_anywhere(
                &packages
                    .iter()
                    .map(|package| package.name())
                    .collect::<Vec<_>>(),
            ),
        );
        let config = Config {
            manifest_license: ScannedPackage::agreed_license(&packages),
            selection,
            ..config
        };
        let registry = RuleRegistry::new(RuleCatalog::selected(&config));
        if registry.is_empty() {
            return Err(anyhow::anyhow!(
                "no rules are configured, so nothing would be checked -- pass --header-file to \
                 enable the header rule"
            ));
        }

        let mut offences = Vec::new();
        let mut files_scanned = 0usize;
        let mut excluded = Vec::new();
        let mut fixed = 0usize;
        let mut rosters: Vec<PackageRoster> = Vec::new();
        // Read once: every manifest path in `workspace_dependencies` is stated
        // relative to it.
        let workspace_root = ManifestResolver::workspace_root(&workspace);
        // Every member, not only those this run walks: a constant one member
        // declares and only another reads is shared all the same, and a run
        // scoped to the first must still see the second.
        let members = ManifestResolver::packages(
            &workspace,
            &Config {
                packages: Vec::new(),
                ..config.clone()
            },
        )?
        .into_iter()
        .map(|member| {
            Ok((
                member.root().clone(),
                ExclusionSet::new(
                    &self
                        .config_from(invocation, sections.of(member.name()))?
                        .excludes,
                )?,
            ))
        })
        .collect::<Result<Vec<_>>>()?;
        let readers = NameReaders::new(ReadNames::index(
            &WorkspaceSources::new(self.source_tree).read(&workspace_root, &members),
        ));
        let reader = SourceReader::new(self.source_tree);

        for package in &packages {
            // Everything the manifest decides is decided here, by the package
            // about to be walked, rather than once for the run -- including
            // which manifest declarations this package is answerable for.
            //
            // The whole workspace's declarations used to be handed to every
            // package, and `check_workspace` runs once per package, so each
            // finding was stated once per member. Twenty real findings became
            // 580 in `etheram-ibft-embassy`, which has twenty-nine of them, and
            // the report gave no sign: every copy was identical, so the count
            // simply tracked the member count.
            //
            // Filtering here rather than deduplicating afterwards, because a
            // finding about `alpha/Cargo.toml` belongs to `alpha` and to no
            // other package. Deduplication would have to guess that; the loop
            // already knows it.
            let manifest = ManifestResolver::relative_to(
                &workspace_root,
                &package.root().join(ManifestResolver::MANIFEST),
            );
            let package_config = Config {
                manifest_license: package.license().map(str::to_string),
                workspace_dependencies: ManifestDependency::in_manifest(
                    config.workspace_dependencies.as_deref(),
                    &manifest,
                ),
                declared_dependencies: ManifestDependency::in_manifest(
                    Some(&config.declared_dependencies),
                    &manifest,
                )
                .unwrap_or_default(),
                name_readers: readers.in_package(&ManifestResolver::relative_to(
                    &workspace_root,
                    package.root(),
                )),
                ..self.config_from(invocation, sections.of(package.name()))?
            };
            let registry = RuleRegistry::new(RuleCatalog::selected(&package_config));
            rosters.push(PackageRoster::new(
                package.name(),
                Self::owned(&registry.names()),
                Self::owned(&RuleCatalog::skipped_names(&package_config.selection)),
                RuleCatalog::unconfigured(&package_config),
            ));
            let exclusions = ExclusionSet::new(&package_config.excludes)?;
            let root = package.root();
            // Read the whole package before judging it. Rules whose subject is
            // the tree -- "there is exactly one all_tests.rs" -- cannot be
            // answered a file at a time, and the file that carries the offence
            // is often the one that does not exist.
            let listed = self.source_tree.rust_files(root)?;
            let (kept, patterns) = exclusions.apply(listed, root).into_parts();
            let mut files: Vec<SourceFile> = Vec::new();
            let mut found: Vec<Offence> = Vec::new();
            for path in kept {
                files_scanned += 1;
                match reader.read(root, &path) {
                    Ok(file) => files.push(file),
                    Err(offence) => found.push(*offence),
                }
            }
            if config.fix {
                let (rewritten, count) = self.repair(root, files, &registry)?;
                files = rewritten;
                fixed += count;
            }
            for file in &files {
                found.extend(registry.check(file));
            }
            found.extend(registry.check_workspace(&files));

            // Deduplicated per package, because a package is the widest scope in
            // which two identical offences are certainly one finding.
            //
            // The workspace question is asked once per package root, so a rule
            // whose subject is the workspace rather than the package can state
            // the same finding twice while walking one member. Those collapse.
            //
            // Across members they must not. A path is relative to its package,
            // so `src/lib.rs` in one member and `src/lib.rs` in another are two
            // real files rendered as one string, and collapsing by content alone
            // threw the second away. Measured on `etheram-embassy`, whose 31
            // members repeat `src/lib.rs` and `tests/all_tests.rs` throughout:
            // 390 offences reported as 364, across four rules, with the summary
            // and the exit code both counting the smaller number. A checker that
            // quietly reports less than it found is the failure this tool exists
            // to refuse, and it was doing it to itself.
            let mut seen = HashSet::new();
            found.retain(|offence| seen.insert(offence.clone()));
            offences.extend(found);
            excluded.push(patterns);
        }

        // Rules run in registration order and the tree-wide pass runs last, so
        // without this the report jumps between files. Sorting is the report's
        // business rather than any rule's -- a rule states facts, and their
        // order on the page is not one of them.
        offences.sort_by(|left, right| left.sort_key().cmp(&right.sort_key()));

        if config.write_baseline {
            return self
                .record(&config, offences)
                .map(|message| (RunOutcome::Clean, message));
        }
        let baselined = self.baselined(&config, offences)?;
        let summary = BaselineOutcome::new(Vec::new(), baselined.suppressed(), baselined.stale());
        let offences = baselined.into_kept();
        let report = Self::report(
            &config,
            &registry,
            ScanTotals {
                files_scanned,
                fixed,
            },
            &Self::merged(excluded),
            &summary,
            &rosters,
            offences,
        );
        Ok((
            RunOutcome::of(report.offences.len()),
            self.renderer.render(&report),
        ))
    }

    // A baseline that was asked for and is not there is an error rather than an
    // empty one. A gate whose baseline path has a typo would otherwise report
    // every existing offence and look like a regression.
    fn baselined(&self, config: &Config, offences: Vec<Offence>) -> Result<BaselineOutcome> {
        let Some(path) = &config.baseline else {
            return Ok(BaselineOutcome::new(offences, 0, 0));
        };
        Ok(self.baseline_store.load(path)?.apply(offences))
    }

    // The command line wins over the file, every time and per setting. A
    // repository states its defaults in stern4rust.toml; a person overrides one
    // of them for one run without having to restate the rest.
    //
    // "Wins" is replacement rather than merging for the list settings. Merging
    // would make `--rule header` mean "header plus whatever the file already
    // selected", which is the opposite of what naming one rule means everywhere
    // else in this tool.
    fn config_from(
        &self,
        invocation: &Invocation,
        section: Option<&PackageConfig>,
    ) -> Result<Config> {
        let directory = &invocation.manifest_directory;
        let file = self.config_source.load(directory)?;
        let found = file.as_ref();
        let header_file = invocation
            .header_file
            .clone()
            .or_else(|| section.and_then(|s| s.header_file_from(directory)))
            .or_else(|| found.and_then(|file| file.header_file_from(directory)));
        let expected_header = match &header_file {
            Some(path) => HeaderSource::read(self.source_tree, path)?,
            None => Vec::new(),
        };
        let threshold = invocation
            .offence_threshold
            .or_else(|| found.and_then(|file| file.offence_threshold()))
            .unwrap_or(OffenceThreshold::DEFAULT);
        // The manifest's licence, the workspace's dependencies and the name
        // readers keep their defaults: `run` fills them in once the manifest
        // has been read, and the command line has nothing to say about them.
        Ok(Config {
            // Discovered beside the manifest when nobody named one, the same
            // way stern4rust.toml is. Implicit suppression would be
            // unacceptable if it were invisible; every report that used a
            // baseline names it and states how many offences it hid, so a
            // reader can always see that one is in force.
            baseline: invocation
                .baseline
                .clone()
                .or_else(|| found.and_then(|file| file.baseline_from(directory)))
                .or_else(|| self.discovered_baseline(directory, invocation.write_baseline)),
            write_baseline: invocation.write_baseline,
            fix: invocation.fix,
            config_file: found.map(|_| directory.join(ConfigFile::NAME)),
            max_files_per_directory: section
                .and_then(|s| s.max_files_per_directory())
                .or_else(|| found.and_then(|file| file.max_files_per_directory())),
            max_subfolders_per_directory: section
                .and_then(|s| s.max_subfolders_per_directory())
                .or_else(|| found.and_then(|file| file.max_subfolders_per_directory())),
            packages: invocation.packages.to_vec(),
            excludes: Self::preferred(
                invocation.excludes.to_vec(),
                Self::section_or_root(
                    section.map(|s| s.exclude()),
                    found.map(|file| file.exclude()),
                ),
            ),
            expected_header,
            offence_threshold: OffenceThreshold::new(threshold),
            selection: RuleSelection::new(
                Self::preferred(
                    invocation.rules.to_vec(),
                    Self::section_or_root(
                        section.map(|s| s.rules()),
                        found.map(|file| file.rules()),
                    ),
                ),
                Self::preferred(
                    invocation.skipped_rules.to_vec(),
                    section
                        .and_then(|s| s.skip())
                        .or(found.map(|file| file.skip())),
                ),
            ),
            ..Config::default()
        })
    }

    // When writing, the default path is the destination whether or not it
    // exists yet. When reading, only an existing file counts -- otherwise every
    // run without a baseline would fail trying to load one.
    fn discovered_baseline(&self, directory: &SourcePath, writing: bool) -> Option<SourcePath> {
        let path = directory.join(Self::BASELINE_NAME);
        (writing || self.source_tree.contains(&path)).then_some(path)
    }

    // One package's exclusions say nothing on their own: a pattern matching
    // nothing in package A and forty files in package B has done its job, and
    // reporting it as dead for A would be a wrong answer rather than a missing
    // one. So the counts are summed across roots before anybody looks at them.
    fn merged(per_root: Vec<Vec<(String, usize)>>) -> ExclusionOutcome {
        let mut totals: Vec<(String, usize)> = Vec::new();
        for counts in per_root {
            for (pattern, count) in counts {
                match totals.iter_mut().find(|(known, _)| *known == pattern) {
                    Some(entry) => entry.1 += count,
                    None => totals.push((pattern, count)),
                }
            }
        }
        ExclusionOutcome::new(Vec::new(), totals)
    }

    fn owned(names: &[&str]) -> Vec<String> {
        names.iter().map(|name| (*name).to_string()).collect()
    }

    fn preferred(from_args: Vec<String>, from_file: Option<&[String]>) -> Vec<String> {
        if !from_args.is_empty() {
            return from_args;
        }
        from_file.map(<[String]>::to_vec).unwrap_or_default()
    }

    // Recording is not judging. The run exits clean because nothing was
    // assessed -- the offences were written down, which is what was asked for.
    //
    // What it says ends with a blank line, as it always has: the caller prints
    // it the way it prints a report.
    fn record(&self, config: &Config, offences: Vec<Offence>) -> Result<String> {
        let path = config
            .baseline
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("--write-baseline needs a path to write to"))?;
        let baseline = Baseline::of(&offences);
        self.baseline_store.save(path, &baseline)?;
        Ok(format!(
            "stern4rust wrote {} offence(s) to {}\n",
            baseline.len(),
            path
        ))
    }

    // Rewrites what can be rewritten and hands back the files as they now are,
    // so the checks that follow judge the repaired tree. Whatever is still
    // wrong is reported exactly as it would have been without --fix -- a fixer
    // that quietly swallowed what it could not fix would be worse than no fixer
    // at all.
    //
    // Each fixer runs only where the package applies the rule it repairs: a
    // rule the package skips, or a run did not select, governs nothing, and a
    // fixer must never edit a file no rule governs.
    fn repair(
        &self,
        root: &SourcePath,
        files: Vec<SourceFile>,
        registry: &RuleRegistry,
    ) -> Result<(Vec<SourceFile>, usize)> {
        let applied = registry.names();
        let rewriters = RewriterRegistry::new(RuleCatalog::rewriters());
        let mut repaired = Vec::with_capacity(files.len());
        let mut count = 0;
        for file in files {
            match rewriters.rewrite(&file, &applied) {
                Some(contents) => {
                    let path = root.join(file.relative_path());
                    self.source_tree.write_source(&path, &contents)?;
                    repaired.push(SourceFile::new(file.relative_path(), &contents));
                    count += 1;
                }
                None => repaired.push(file),
            }
        }
        Ok((repaired, count))
    }

    fn report(
        config: &Config,
        registry: &RuleRegistry,
        totals: ScanTotals,
        excluded: &ExclusionOutcome,
        baselined: &BaselineOutcome,
        rosters: &[PackageRoster],
        offences: Vec<Offence>,
    ) -> RunReport {
        RunReport {
            files_scanned: totals.files_scanned,
            threshold: config.offence_threshold,
            applied: Self::owned(&registry.names()),
            skipped: Self::owned(&RuleCatalog::skipped_names(&config.selection)),
            unconfigured: RuleCatalog::unconfigured(config),
            rosters: rosters.to_vec(),
            exclusions: excluded.excluded().to_vec(),
            config_file: Self::shown(config.config_file.as_ref()),
            baseline: Self::shown(config.baseline.as_ref()),
            suppressed: baselined.suppressed(),
            stale: baselined.stale(),
            fixed: totals.fixed,
            offences,
        }
    }

    // Every rule the registry can hold, not the subset this run selected: the
    // reader asking what a rule wants has not chosen one yet.
    //
    // Two rules stay out of a registry until something configures them -- the
    // header rule until it is told what the header says, and
    // spdx-matches-manifest until a manifest declares a licence. Both are
    // handed a stand-in that nothing ever reads, because a listing missing a
    // rule reads as a tool that does not have it. The first draft supplied only
    // the header and quietly listed twenty.
    fn rule_listing(&self) -> String {
        let registry = RuleRegistry::new(RuleCatalog::selected(&Config {
            expected_header: vec![String::new()],
            manifest_license: Some(String::new()),
            ..Config::default()
        }));
        self.renderer.list(&registry.explanations())
    }

    // A section states its whole list rather than adding to the root's, which is
    // the argument already made for the command line against the file, one level
    // down: a reader who wants to know what a package skips reads one list.
    fn section_or_root<'b>(
        section: Option<&'b [String]>,
        root: Option<&'b [String]>,
    ) -> Option<&'b [String]> {
        match section {
            Some(values) if !values.is_empty() => Some(values),
            _ => root,
        }
    }

    fn shown(path: Option<&SourcePath>) -> Option<String> {
        path.map(|path| path.as_str().replace('\\', "/"))
    }

    // A misspelled rule name is an error rather than a switch that quietly
    // matches nothing, and asking for the header rule without a header file is
    // an error rather than an empty run. Both would otherwise look exactly like
    // a run that worked.
    fn validate_selection(config: &Config) -> Result<()> {
        let known = RuleCatalog::known_names();
        let unknown = config.selection.unknown_in(&known);
        if !unknown.is_empty() {
            return Err(anyhow::anyhow!(
                "unknown rule name(s): {} -- the rules are: {}",
                unknown.join(", "),
                known.join(", ")
            ));
        }
        if config.selection.selects_explicitly(HeaderRule::NAME)
            && config.expected_header.is_empty()
        {
            return Err(anyhow::anyhow!(
                "--rule {} needs --header-file, otherwise the run would apply no rules at all",
                HeaderRule::NAME
            ));
        }
        Ok(())
    }
}
