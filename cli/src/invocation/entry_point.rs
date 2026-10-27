// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::path::Path;

use anyhow::Result;
use clap::Parser;
use stern4rust::reporting::run_outcome::RunOutcome;
use stern4rust::runner::Runner;

use crate::invocation::args::Args;
use crate::system::cargo_workspace_reader::CargoWorkspaceReader;
use crate::system::json_baseline_store::JsonBaselineStore;
use crate::system::toml_config_source::TomlConfigSource;
use crate::system::walkdir_source_tree::WalkdirSourceTree;

// The composition root: the command line in, the report out on stdout, a
// verdict back. The one place cargo metadata, the real file system, TOML and
// JSON are chosen.
#[derive(Debug, Clone, Copy, Default)]
pub struct EntryPoint;

impl EntryPoint {
    pub fn new() -> Self {
        Self
    }

    pub fn run<I>(&self, argv: I) -> Result<RunOutcome>
    where
        I: IntoIterator<Item = String>,
    {
        let (outcome, report) = self.run_reporting(argv)?;
        println!("{report}");
        Ok(outcome)
    }

    // The same run, handing back what it would have printed, so a test can
    // assert on what was said as well as on the verdict.
    pub fn run_reporting<I>(&self, argv: I) -> Result<(RunOutcome, String)>
    where
        I: IntoIterator<Item = String>,
    {
        let args = Args::parse_from(Args::without_cargo_subcommand(argv));
        let invocation = args.invocation()?;
        let workspace_reader =
            CargoWorkspaceReader::new(args.manifest_path().map(Path::to_path_buf));
        let source_tree = WalkdirSourceTree::new();
        let config_source = TomlConfigSource::new();
        let baseline_store = JsonBaselineStore::new();
        Runner::new(
            &workspace_reader,
            &source_tree,
            &config_source,
            &baseline_store,
        )
        .run_reporting(&invocation)
    }
}
