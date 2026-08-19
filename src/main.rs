// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::process::exit;

use clap::Parser;
use std::env::args;
use stern4rust::runner::Runner;
use stern4rust::settings::args::Args;

// The only place that turns a verdict into an exit code. A failure to run is an
// Err, which reaches the shell as 1; a broken rule is a successful run with a
// finding, which is 2. Collapsing the two would make "I could not read your
// code" indistinguishable from "your code is fine".
fn main() {
    match Runner::run(Args::parse_from(Args::without_cargo_subcommand(args()))) {
        Ok(outcome) => exit(outcome.exit_code()),
        Err(error) => {
            eprintln!("stern4rust: {error:#}");
            exit(1);
        }
    }
}
