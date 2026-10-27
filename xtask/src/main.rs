// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::env::args;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;
use xtask::crap::crap_report_parser::CrapReportParser;
use xtask::gates::crap_gate::CrapGate;
use xtask::gates::dry_gate::DryGate;
use xtask::gates::gate::Gate;
use xtask::gates::iceberg_gate::IcebergGate;
use xtask::gates::shear_gate::ShearGate;
use xtask::gates::stage2::Stage2;
use xtask::gates::stern_self_gate::SternSelfGate;
use xtask::gates::twin_gate::TwinGate;
use xtask::process::system_command_runner::SystemCommandRunner;

const CLI_PACKAGE: &str = "cargo-stern4rust";
const CORE_PACKAGE: &str = "stern4rust-core";
const XTASK_PACKAGE: &str = "xtask";
const DRY_EXCLUDES: [&[&str]; 3] = [&["xtask"], &["core", "tests"], &["cli", "tests"]];
const CRAP_THRESHOLD: &str = "15";
const ICEBERG_THRESHOLD: &str = "20";
const DRY_BASELINE: &str = "dry4rust-baseline.json";
const DRY_MIN_NODES: &str = "25";

// Reading the real process argv and wiring the concrete runner are the two
// things no test can reach, so they are all this binary does.
fn main() -> ExitCode {
    match args().nth(1).as_deref() {
        Some("stage2") => run_stage2(),
        _ => {
            eprintln!("usage: cargo xtask stage2");
            ExitCode::FAILURE
        }
    }
}

fn run_stage2() -> ExitCode {
    let manifest_path = workspace_manifest_path();
    let runner = SystemCommandRunner::new();
    let parser = CrapReportParser::new();

    // Every member. The house rules reach xtask as well, so the crate that runs
    // the gates is held to the rules it exists to enforce.
    let every = vec![
        String::from(CORE_PACKAGE),
        String::from(CLI_PACKAGE),
        String::from(XTASK_PACKAGE),
    ];

    // First, not last. Its corrections are renames, file moves and directory
    // splits, so a layout it is about to reject is one the others would have
    // measured for nothing -- the same reason every sibling repository
    // runs stern4rust ahead of the rest. The PowerShell script ran it last.
    // Built from this checkout's cli, which holds the binary.
    let stern = SternSelfGate::new(
        &runner,
        String::from(CLI_PACKAGE),
        manifest_path.clone(),
        every,
    );

    // Second: every declared dependency used, in the section that uses it. Its
    // corrections edit manifests only, so they move nothing the gates behind it
    // measure, and it costs a second.
    let shear = ShearGate::new(&runner, manifest_path.clone());

    // Third, for the same reason stern4rust is first: removing a duplicate
    // moves code between files, which changes what the three behind it measure.
    // One run from the workspace root over both published crates' sources, so a
    // copy from one into the other is seen, the trees it must not judge excluded
    // by full path -- tests repeat their arrangement by design, and xtask is not
    // published -- against a baseline of what was already duplicated when the
    // gate arrived, so it fails on what a change adds. The floor leaves out units
    // whose body is a single call -- the accessors every rule must write and a
    // `check` that hands its two halves to `SourceItems` -- whose sameness is the
    // `Rule` trait's, not a copy; a rule set that is open by design would grow
    // those groups with every rule added.
    let root = workspace_root();
    let dry = DryGate::new(
        &runner,
        root.to_string_lossy().into_owned(),
        DRY_EXCLUDES
            .iter()
            .map(|exclude| below(&root, exclude).to_string_lossy().into_owned())
            .collect(),
        root.join(DRY_BASELINE).to_string_lossy().into_owned(),
        String::from(DRY_MIN_NODES),
    );

    // The published crates only. CRAP scores functions against their coverage,
    // and the measuring gates are the tool's own bill rather than xtask's.
    let published = vec![String::from(CORE_PACKAGE), String::from(CLI_PACKAGE)];

    let crap = CrapGate::new(
        &runner,
        &parser,
        manifest_path.clone(),
        published.clone(),
        String::from(CRAP_THRESHOLD),
    );
    let twin = TwinGate::new(&runner, manifest_path.clone(), published.clone());
    let iceberg = IcebergGate::new(
        &runner,
        manifest_path,
        published,
        String::from(ICEBERG_THRESHOLD),
    );

    let gates: Vec<&dyn Gate> = vec![&stern, &shear, &dry, &crap, &twin, &iceberg];

    match Stage2::new(gates).run() {
        Ok(()) => {
            println!("\nstern4rust Stage 2 passed!");
            ExitCode::SUCCESS
        }
        Err(reason) => {
            eprintln!("\nFailed: {reason}");
            ExitCode::FAILURE
        }
    }
}

fn workspace_manifest_path() -> String {
    workspace_root()
        .join("Cargo.toml")
        .to_string_lossy()
        .into_owned()
}

// Joined a component at a time, so the path carries the platform's separator
// throughout -- dry4rust matches its exclusions as text.
fn below(root: &Path, components: &[&str]) -> PathBuf {
    components
        .iter()
        .fold(root.to_path_buf(), |path, component| path.join(component))
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives one directory below the workspace root")
        .to_path_buf()
}
