// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use anyhow::Result;
use clap::Parser;
use stern4rust::reporting::run_outcome::RunOutcome;
use stern4rust::runner::Runner;

use crate::invocation::args::Args;
use crate::system::walkdir_source_tree::WalkdirSourceTree;

// The composition root: the command line in, the report out on stdout, a
// verdict back. The one place the real file system is chosen.
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
        let source_tree = WalkdirSourceTree::new();
        Runner::new(&source_tree).run_reporting(&invocation)
    }
}
