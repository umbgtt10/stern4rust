// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::gates::gate::Gate;
use crate::process::command_runner::CommandRunner;

pub struct SternSelfGate<'a> {
    runner: &'a dyn CommandRunner,
    binary: String,
    manifest_path: String,
    packages: Vec<String>,
}

impl<'a> SternSelfGate<'a> {
    pub fn new(
        runner: &'a dyn CommandRunner,
        binary: String,
        manifest_path: String,
        packages: Vec<String>,
    ) -> Self {
        Self {
            runner,
            binary,
            manifest_path,
            packages,
        }
    }
}

impl Gate for SternSelfGate<'_> {
    fn label(&self) -> String {
        String::from("Own rules")
    }

    fn run(&self) -> Result<(), String> {
        // Built from source rather than run from whatever is installed. The
        // gate has to judge the tree it is standing in, not the last version
        // that happened to be published -- which for this repository is the
        // whole point, since a rule it breaks is a rule it is about to ship.
        //
        // `--bin` before the `--` chooses what cargo builds; `--manifest-path`
        // and `--package` after it tell the tool what to read. The workspace
        // root is a virtual manifest and names no single package, so the tool
        // is told which one rather than left to guess.
        let mut args = vec![
            String::from("run"),
            String::from("--quiet"),
            String::from("--bin"),
            self.binary.clone(),
            String::from("--"),
            String::from("--manifest-path"),
            self.manifest_path.clone(),
        ];
        for package in &self.packages {
            args.push(String::from("--package"));
            args.push(package.clone());
        }

        // 2 is the tool's own "a rule was broken"; anything else non-zero means
        // it could not run at all. Kept apart so a build failure cannot read as
        // a clean codebase.
        match self.runner.run_streaming("cargo", &args)? {
            Some(0) => Ok(()),
            Some(2) => Err(String::from("a house coding rule was broken")),
            Some(code) => Err(format!("could not run, exit code {code}")),
            None => Err(String::from("could not run, terminated by signal")),
        }
    }
}
