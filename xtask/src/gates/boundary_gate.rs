// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use syn::{File, parse_file};

use crate::boundary::boundary_rule::BoundaryRule;
use crate::boundary::crate_root_attributes::CrateRootAttributes;
use crate::boundary::package_dependencies::PackageDependencies;
use crate::boundary::source_files::SourceFiles;
use crate::boundary::technology_use_collector::TechnologyUseCollector;
use crate::gates::gate::Gate;
use crate::gates::gate_failure::GateFailure;
use crate::process::command_runner::CommandRunner;
use std::path::Path;

// A package kept free of technology: its manifest names only the crates its
// rule allows, its crate root carries the attributes that make the compiler
// guard its source, and no scanned file reaches the rule's modules, macros or
// methods. Every finding is reported at once: manifest, root, then files.
pub struct BoundaryGate<'a> {
    runner: &'a dyn CommandRunner,
    sources: &'a dyn SourceFiles,
    dependencies: &'a PackageDependencies,
    attributes: &'a CrateRootAttributes,
    manifest_path: String,
    rule: BoundaryRule,
}

impl<'a> BoundaryGate<'a> {
    pub fn new(
        runner: &'a dyn CommandRunner,
        sources: &'a dyn SourceFiles,
        dependencies: &'a PackageDependencies,
        attributes: &'a CrateRootAttributes,
        manifest_path: String,
        rule: BoundaryRule,
    ) -> Self {
        Self {
            runner,
            sources,
            dependencies,
            attributes,
            manifest_path,
            rule,
        }
    }

    fn manifest_findings(&self) -> Result<Vec<String>, GateFailure> {
        let metadata = self.runner.run_capturing(
            "cargo",
            &[
                String::from("metadata"),
                String::from("--no-deps"),
                String::from("--format-version"),
                String::from("1"),
                String::from("--manifest-path"),
                self.manifest_path.clone(),
            ],
        )?;
        if !metadata.is_success() {
            return Err(GateFailure::CouldNotRun(metadata.exit_code()));
        }
        let declared = self
            .dependencies
            .of(metadata.stdout(), &self.rule.package)
            .map_err(|error| GateFailure::Found(format!("could not read cargo metadata: {error}")))?
            .ok_or_else(|| {
                GateFailure::Found(format!("{} is not in the workspace", self.rule.package))
            })?;
        Ok(declared
            .iter()
            .filter(|crate_name| !self.rule.allowed_crates.contains(crate_name))
            .map(|crate_name| {
                format!(
                    "{} depends on {crate_name}, which its rule does not allow",
                    self.rule.package
                )
            })
            .collect())
    }

    fn parsed(&self, file: &Path) -> Result<File, GateFailure> {
        let source = self
            .sources
            .read(file)
            .map_err(|failure| GateFailure::Found(failure.to_string()))?;
        parse_file(&source).map_err(|error| {
            GateFailure::Found(format!("could not parse {}: {error}", file.display()))
        })
    }

    fn root_findings(&self) -> Result<Vec<String>, GateFailure> {
        let present = self.attributes.of(&self.parsed(&self.rule.crate_root)?);
        Ok(self
            .rule
            .required_attributes
            .iter()
            .filter(|required| !present.contains(required))
            .map(|missing| {
                format!(
                    "{} does not carry #![{missing}]",
                    self.rule.crate_root.display()
                )
            })
            .collect())
    }

    fn source_findings(&self) -> Result<Vec<String>, GateFailure> {
        let mut findings = Vec::new();
        let mut scanned = 0;
        for dir in &self.rule.source_dirs {
            let files = self
                .sources
                .rust_files(dir)
                .map_err(|failure| GateFailure::Found(failure.to_string()))?;
            for file in files {
                scanned += 1;
                let uses = TechnologyUseCollector::collect(&self.parsed(&file)?, &self.rule);
                if !uses.is_empty() {
                    findings.push(format!("{} uses {}", file.display(), uses.join(", ")));
                }
            }
        }
        if scanned == 0 {
            return Err(GateFailure::Found(format!(
                "no source file of {} was scanned",
                self.rule.package
            )));
        }
        Ok(findings)
    }
}

impl Gate for BoundaryGate<'_> {
    fn label(&self) -> String {
        String::from("Technology-free core")
    }

    fn run(&self) -> Result<(), GateFailure> {
        let mut findings = self.manifest_findings()?;
        findings.extend(self.root_findings()?);
        findings.extend(self.source_findings()?);
        if findings.is_empty() {
            Ok(())
        } else {
            Err(GateFailure::Found(findings.join("\n")))
        }
    }
}
