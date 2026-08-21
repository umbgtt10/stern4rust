// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::reporting::output_format::OutputFormat;
use clap::Parser;
use std::path::PathBuf;

#[derive(Clone, Debug, Parser)]
#[command(name = "cargo-stern4rust")]
#[command(bin_name = "cargo stern4rust")]
#[command(version)]
#[command(about = "Check Rust packages and fail the build when the rule is broken")]
pub struct Args {
    /// Workspace or package manifest to analyse. Defaults to the Cargo.toml in
    /// the current directory, which is what makes a bare `cargo stern4rust`
    /// work from inside the thing being judged.
    #[arg(long)]
    pub manifest_path: Option<PathBuf>,

    /// Restrict the run to these packages; repeatable. Omit to take the
    /// manifest's own package, or every member if it is a workspace root.
    /// Naming a package that the manifest does not scan is an error rather than
    /// an empty run, because a typo in a gate script otherwise reads as a pass.
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

    /// How many offences the report prints. A first run against a large
    /// codebase can find a thousand, and a thousand rows is a wall rather than
    /// a report. The cap is on what is shown and never on what is counted:
    /// the summary, the omitted count and the exit code all see every offence.
    /// Use 0 for no limit.
    /// Option rather than a defaulted value so that "not passed" is
    /// distinguishable from "passed the default": without that, a
    /// stern4rust.toml could never set the threshold, because every run would
    /// look like the reader had asked for 100 on the command line.
    #[arg(long)]
    pub offence_threshold: Option<usize>,

    /// List every rule with a line saying what it wants, a scrap of source
    /// that breaks it and the same scrap put right, then exit without
    /// scanning. Answers the question a first run raises -- what does this
    /// rule actually want -- without needing a codebase to ask it against.
    #[arg(long = "rules")]
    pub list_rules: bool,

    /// Apply only these rules; repeatable. Omit to apply every rule. Naming one
    /// makes the selection a whitelist, which is what lets a codebase facing
    /// hundreds of offences gate on one rule today and the rest as it goes.
    #[arg(long = "rule")]
    pub rules: Vec<String>,

    /// Repair what can be repaired mechanically, then report what is left.
    /// Only test-file-structure offences are fixable today: item order, section
    /// order and blank lines. Everything else is reported unchanged, and the
    /// report says how many offences were fixed and how many were not.
    #[arg(long)]
    pub fix: bool,

    /// Offences recorded here are not reported and do not fail the run. What
    /// lets a codebase with hundreds of existing offences enforce every rule
    /// against new code without first fixing the old. The count of suppressed
    /// offences is always in the summary, never hidden.
    #[arg(long)]
    pub baseline: Option<PathBuf>,

    /// Record the current offences as the baseline and exit clean, instead of
    /// judging against one. Writes to --baseline, or to stern4rust-baseline.json
    /// beside the manifest.
    #[arg(long)]
    pub write_baseline: bool,

    /// Keep these paths out of the run; repeatable, matched as a glob against
    /// the package-relative path. For a tree the repository cannot move --
    /// vendored source, generated output. Every pattern is named in the report
    /// with how many files it removed, including zero, so an exclusion is
    /// something the reader can see rather than a silence.
    #[arg(long = "exclude")]
    pub excludes: Vec<String>,

    /// Do not apply these rules; repeatable. Subtracted from whatever --rule
    /// selected, so skipping wins over selecting.
    #[arg(long = "skip")]
    pub skipped_rules: Vec<String>,
}

impl Args {
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
