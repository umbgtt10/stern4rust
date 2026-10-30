// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::gates::gate::Gate;
use crate::gates::gate_failure::GateFailure;
use crate::process::command_runner::CommandRunner;

// Stage 1 starts here while the tools are in development: the lock file moves
// to the newest compatible version of everything, so the tests run against what
// a fresh `cargo install` would resolve, and a fixed advisory arrives without
// anyone asking for it. Its changes are committed on their own.
pub struct CargoUpdateGate<'a> {
    runner: &'a dyn CommandRunner,
    manifest_path: String,
}

impl<'a> CargoUpdateGate<'a> {
    pub fn new(runner: &'a dyn CommandRunner, manifest_path: String) -> Self {
        Self {
            runner,
            manifest_path,
        }
    }
}

impl Gate for CargoUpdateGate<'_> {
    fn label(&self) -> String {
        String::from("Update")
    }

    fn run(&self) -> Result<(), GateFailure> {
        let args = vec![
            String::from("update"),
            String::from("--manifest-path"),
            self.manifest_path.clone(),
        ];
        match self.runner.run_streaming("cargo", &args)? {
            Some(0) => Ok(()),
            code => Err(GateFailure::Exited(code)),
        }
    }
}
