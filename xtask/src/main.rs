// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::env::args;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;
use xtask::boundary::boundary_rule::BoundaryRule;
use xtask::boundary::crate_root_attributes::CrateRootAttributes;
use xtask::boundary::disk_source_files::DiskSourceFiles;
use xtask::boundary::package_dependencies::PackageDependencies;
use xtask::crap::crap_report_parser::CrapReportParser;
use xtask::gates::boundary_gate::BoundaryGate;
use xtask::gates::cargo_update_gate::CargoUpdateGate;
use xtask::gates::crap_gate::CrapGate;
use xtask::gates::deny_gate::DenyGate;
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
const ALLOWED_CRATES: [&str; 4] = ["globset", "proc-macro2", "quote", "syn"];
const CORE_DIRS: [[&str; 2]; 2] = [["core", "src"], ["core", "tests"]];
const CORE_ROOT: [&str; 3] = ["core", "src", "lib.rs"];
const DRY_EXCLUDES: [&[&str]; 3] = [&["xtask"], &["core", "tests"], &["cli", "tests"]];
const FORBIDDEN_MODULES: [&str; 6] = [
    "std::env",
    "std::fs",
    "std::io",
    "std::net",
    "std::os",
    "std::process",
];
const FORBIDDEN_MACROS: [&str; 5] = ["dbg", "eprint", "eprintln", "print", "println"];
const REQUIRED_ATTRIBUTES: [&str; 2] = ["no_std", "forbid(unsafe_code)"];
const FORBIDDEN_METHODS: [&str; 10] = [
    "canonicalize",
    "exists",
    "is_dir",
    "is_file",
    "is_symlink",
    "metadata",
    "read_dir",
    "read_link",
    "symlink_metadata",
    "try_exists",
];
const CRAP_THRESHOLD: &str = "15";
const ICEBERG_THRESHOLD: &str = "20";
const DRY_BASELINE: &str = "dry4rust-baseline.json";
const DRY_MIN_NODES: &str = "25";

// Reading the real process argv and wiring the concrete runner are the two
// things no test can reach, so they are all this binary does.
fn main() -> ExitCode {
    match args().nth(1).as_deref() {
        Some("stage2") => run_stage2(),
        Some("sync") => run_sync(),
        _ => {
            eprintln!("usage: cargo xtask <stage2|sync>");
            ExitCode::FAILURE
        }
    }
}

// Stage 1's first step: the lock file to the newest compatible versions.
fn run_sync() -> ExitCode {
    let runner = SystemCommandRunner::new();
    let update = CargoUpdateGate::new(&runner, workspace_manifest_path());
    match Stage2::new(vec![&update]).run() {
        Ok(()) => {
            println!("\nstern4rust sync done");
            ExitCode::SUCCESS
        }
        Err(reason) => {
            eprintln!("\nFailed: {reason}");
            ExitCode::FAILURE
        }
    }
}

fn run_stage2() -> ExitCode {
    let manifest_path = workspace_manifest_path();
    let runner = SystemCommandRunner::new();
    let parser = CrapReportParser::new();

    // First: core stays free of technology -- only its allowed crates in its
    // manifest, `no_std` and `forbid(unsafe_code)` on its root, and no file
    // system, process, environment or stdout in its tests -- so mutation testing
    // it needs nothing but its own tests. A breach is fixed by moving code into
    // cli, which changes what every later gate, the own-rules gate included,
    // measures; and it costs a second.
    let root = workspace_root();
    let sources = DiskSourceFiles::new();
    let dependencies = PackageDependencies::new();
    let attributes = CrateRootAttributes::new();
    let boundary = BoundaryGate::new(
        &runner,
        &sources,
        &dependencies,
        &attributes,
        manifest_path.clone(),
        BoundaryRule {
            package: String::from(CORE_PACKAGE),
            source_dirs: CORE_DIRS.iter().map(|dir| below(&root, dir)).collect(),
            allowed_crates: ALLOWED_CRATES.map(String::from).to_vec(),
            crate_root: below(&root, &CORE_ROOT),
            required_attributes: REQUIRED_ATTRIBUTES.map(String::from).to_vec(),
            forbidden_modules: FORBIDDEN_MODULES.map(String::from).to_vec(),
            forbidden_macros: FORBIDDEN_MACROS.map(String::from).to_vec(),
            forbidden_methods: FORBIDDEN_METHODS.map(String::from).to_vec(),
        },
    );

    // Every member. The house rules reach xtask as well, so the crate that runs
    // the gates is held to the rules it exists to enforce.
    let every = vec![
        String::from(CORE_PACKAGE),
        String::from(CLI_PACKAGE),
        String::from(XTASK_PACKAGE),
    ];

    // Second, behind the boundary and ahead of the rest. Its corrections are
    // renames, file moves and directory splits, so a layout it is about to
    // reject is one the others would have measured for nothing -- the same
    // reason every sibling repository runs stern4rust ahead of the rest. Built
    // from this checkout's cli, which holds the binary.
    let stern = SternSelfGate::new(
        &runner,
        String::from(CLI_PACKAGE),
        manifest_path.clone(),
        every,
    );

    // Third: every declared dependency used, in the section that uses it. Its
    // corrections edit manifests only, so they move nothing the gates behind it
    // measure, and it costs a second.
    let shear = ShearGate::new(&runner, manifest_path.clone());
    // Last in stage 2: its advisories change without the code changing (see DenyGate).
    let deny = DenyGate::new(&runner, manifest_path.clone());

    // Fourth, for the same reason stern4rust runs early: removing a duplicate
    // moves code between files, which changes what the three behind it measure.
    // One run over both published crates' sources, so a copy from one into the
    // other is seen -- tests repeat their arrangement by design, and xtask is not
    // published -- against a baseline of what was already duplicated when the
    // gate arrived, so it fails on what a change adds. The floor leaves out units
    // whose body is a single call -- the accessors every rule must write and a
    // `check` that hands its two halves to `SourceItems` -- whose sameness is the
    // `Rule` trait's, not a copy; a rule set that is open by design would grow
    // those groups with every rule added.
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

    let gates: Vec<&dyn Gate> = vec![
        &boundary, &stern, &shear, &dry, &crap, &twin, &iceberg, &deny,
    ];

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
