// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::gates::gate::Gate;
use crate::gates::gate_failure::GateFailure;
use crate::process::command_runner::CommandRunner;
use crate::shear::ignore_reasons::IgnoreReasons;
use std::path::Path;

const BINARY: &str = "cargo-shear";
const FOUND: i32 = 1;
const COULD_NOT_RUN: i32 = 2;

// Every declared dependency is one the code uses, in the section that uses it.
// The exceptions are read first: one without its reason fails the gate before
// cargo-shear runs, since cargo-shear would honour it.
pub struct ShearGate<'a> {
    runner: &'a dyn CommandRunner,
    manifest_path: String,
}

impl<'a> ShearGate<'a> {
    pub fn new(runner: &'a dyn CommandRunner, manifest_path: String) -> Self {
        Self {
            runner,
            manifest_path,
        }
    }

    fn explained(&self) -> Result<(), GateFailure> {
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
        let unexplained = IgnoreReasons::new()
            .unexplained(metadata.stdout())
            .map_err(|error| {
                GateFailure::Found(format!("could not read cargo metadata: {error}"))
            })?;
        if unexplained.is_empty() {
            Ok(())
        } else {
            Err(GateFailure::Found(format!(
                "cargo-shear ignores without a reason: {}",
                unexplained.join(", ")
            )))
        }
    }

    fn workspace_root(&self) -> String {
        Path::new(&self.manifest_path)
            .parent()
            .map(|root| root.to_string_lossy().into_owned())
            .unwrap_or_default()
    }
}

impl Gate for ShearGate<'_> {
    fn label(&self) -> String {
        String::from("Dependencies")
    }

    fn run(&self) -> Result<(), GateFailure> {
        if !self.runner.is_available(BINARY) {
            return Err(GateFailure::NotInstalled(String::from(BINARY)));
        }
        self.explained()?;

        // --locked: the gate reads the lock file and never writes it.
        // --deny-warnings: an optional dependency no feature's code uses fails
        // as an unused one does.
        let args = vec![
            String::from("shear"),
            String::from("--locked"),
            String::from("--deny-warnings"),
            self.workspace_root(),
        ];
        match self.runner.run_streaming("cargo", &args)? {
            Some(0) => Ok(()),
            Some(FOUND) => Err(GateFailure::Found(String::from(
                "an unused or misplaced dependency",
            ))),
            Some(COULD_NOT_RUN) => Err(GateFailure::CouldNotRun(Some(COULD_NOT_RUN))),
            code => Err(GateFailure::Exited(code)),
        }
    }
}
