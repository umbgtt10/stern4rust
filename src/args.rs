// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

use clap::Parser;

use crate::output_format::OutputFormat;

#[derive(Debug, Parser)]
#[command(name = "cargo-stern4rust")]
#[command(bin_name = "cargo stern4rust")]
#[command(version)]
#[command(about = "Check Rust packages and fail the build when the rule is broken")]
pub struct Args {
    #[arg(long)]
    pub manifest_path: Option<PathBuf>,

    #[arg(long = "package")]
    pub packages: Vec<String>,

    /// File holding the header every .rs file must open with. It is data rather
    /// than a built-in constant because it is never the same twice: MIT here,
    /// Apache 2.0 in a sibling repository, and a different year again next year.
    #[arg(long)]
    pub header_file: Option<PathBuf>,

    /// How to report. The table is for a person; `json` is the same run as a
    /// document, for a gate script or an agent that would otherwise have to
    /// guess where one column of the table ends and the next begins.
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    pub format: OutputFormat,
}

impl Args {
    pub fn parse_args() -> Self {
        Self::parse_from(Self::without_cargo_subcommand(std::env::args()))
    }

    /// Cargo invokes `cargo stern4rust` as `cargo-stern4rust stern4rust ...`, so
    /// the subcommand name arrives as an extra leading argument that clap would
    /// otherwise reject. Running the binary directly does not repeat it, which
    /// is why the strip is conditional rather than unconditional.
    pub fn without_cargo_subcommand<I>(args: I) -> Vec<String>
    where
        I: IntoIterator<Item = String>,
    {
        let args: Vec<String> = args.into_iter().collect();
        if args.get(1).map(String::as_str) != Some("stern4rust") {
            return args;
        }
        let mut forwarded = Vec::with_capacity(args.len() - 1);
        forwarded.extend(args.iter().take(1).cloned());
        forwarded.extend(args.into_iter().skip(2));
        forwarded
    }
}
