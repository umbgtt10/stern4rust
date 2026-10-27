# A technology-free core and a cli crate

Status: **in progress** -- one commit per slice, in the order below.

## Why

Mutation testing pays for every mutant with a build and a test run. A mutant in
code that runs `cargo metadata`, walks a disk or prints is slow to kill, and a
test that needs a workspace to run in is slow to run: today the runner's 38
tests spawn `cargo metadata` and write packages to the temporary directory, the
resolver's 17 run cargo on this repository, and the config, header, baseline,
walker and workspace-sources tests write to disk. A run describes the workspace
six times over. So the rules, package selection, the configuration's meaning,
the adoption tools and the text report move into a crate that touches no
technology at all, and mutation testing is scoped to that crate alone.

## The shape

The workspace grows a third member:

| member | package | library | what it holds |
|---|---|---|---|
| `core/` | `stern4rust-core` | `stern4rust` | `#![no_std]`, `#![forbid(unsafe_code)]`; the rules, the rewriters `--fix` applies, package selection, the configuration's meaning, the baseline and the exclusions, the text report. Depends on `globset`, `proc-macro2`, `quote` and `syn` only |
| `cli/` | `cargo-stern4rust` | `stern4rust_cli` | the binary, `Args` (clap), the composition root `EntryPoint`, the JSON report, and every adapter: cargo metadata, the manifests, `stern4rust.toml`, the baseline file and the file system |
| `xtask/` | `xtask` | `xtask` | the gates |

`cli` depends on `core`, never the reverse. Both published crates share one
workspace version, unchanged at 0.18.0.

### Ports

Core declares a trait for everything technological it needs, implemented in
`cli/src/` and, for core's own tests, by in-memory fakes under
`core/tests/support/`:

| port (`core/src/traits/`) | what it answers | adapter |
|---|---|---|
| `SourceTree` | the `.rs` files under a root, in path order with `target/` and `.git/` left out; a file's text; whether a path exists; and writing a file back, for `--fix` | cli `WalkdirSourceTree` |
| `WorkspaceReader` | the workspace once per run: its root, whether it declares a workspace, and each member's name, manifest, licence and declared dependencies | cli `CargoWorkspaceReader` (one `cargo metadata --no-deps`, each member manifest read as TOML) |
| `ConfigSource` | the `stern4rust.toml` beside a manifest, if there is one | cli `TomlConfigSource` |
| `BaselineStore` | a baseline file's counts, and writing them | cli `JsonBaselineStore` |
| `ReportRenderer` | the report and the rule listing as text | core `TextReportRenderer`; cli `JsonReportRenderer` |

Core works on its own records -- `Invocation` (what the command line asked),
`WorkspaceDescription`, `PackageDescription`, `RunReport` -- on `SourcePath`, a
path held as text with `/` as its only separator, which cli converts to at the
edge, and fails with typed errors, each worded as the run words it today.
`Runner::run` returns the verdict and the rendered text; printing it and
turning the verdict into an exit code is `EntryPoint`'s and `main`'s.

The binary's behaviour does not change: stdout, stderr and exit codes are
compared byte for byte against the released 0.18.0 after every slice from the
split on. Two exceptions are deliberate: a walked `.rs` file whose name is not
UTF-8 fails the run naming it, where 0.18.0 judged it under a lossy name; and a
`--manifest-path`, `--header-file` or `--baseline` that is not UTF-8 fails
naming the flag.

### The gate

A stage-2 gate, first in the run, keeps core technology-free: its manifest may
name only `globset`, `proc-macro2`, `quote` and `syn` in any section, its crate
root must carry `#![no_std]` and `#![forbid(unsafe_code)]`, and no file under
`core/src` or `core/tests` may reach `std::{env,fs,io,net,os,process}`, the
printing macros or the file-system methods. The measuring gates cover both
published crates; the house rules cover all three members.

### Mutation testing

`.cargo/mutants.toml` scopes cargo-mutants to `core/src`; each mutant runs
core's tests only.

## Slices

| # | slice | status |
|---|---|---|
| S0 | this record | done |
| S1 | `DryGate`: one run from the workspace root, the trees it must not judge excluded by full path; the ignore file dry4rust no longer reads goes | done |
| S2 | `adoption/` moves under `settings/`, sources and tests, paths only, so `traits/` can be core's fifth folder | done |
| S3 | the split: `cli` takes `main.rs`, `Args` and a new `EntryPoint`; core is renamed; `Runner` reads an `Invocation`; `[workspace.lints]` forbids unsafe code | done |
| S4 | the `SourceTree` port and `SourcePath`; `SourceReader` and `WorkspaceSources` read through it, `--fix` writes through it; the walker becomes cli's `WalkdirSourceTree` | done |
| S5 | the `WorkspaceReader` port and core's workspace records; `CargoWorkspaceReader` in cli, with the manifest tables; the workspace described once per run | done |
| S6 | the `ConfigSource` and `BaselineStore` ports; `TomlConfigSource` and `JsonBaselineStore` in cli; toml leaves core | |
| S7 | the `ReportRenderer` port and `RunReport`: the text report in core, the JSON report and the output format in cli; serde, serde_json and clap leave core | |
| S8 | `#![no_std]`: alloc imports, typed errors; anyhow leaves core | |
| S9 | the technology-free gate, first in stage 2 | |
| S10 | `.cargo/mutants.toml` | |
| S11 | docs, and this record moves to `docs/history/` | |

## Tests that move

No test is deleted, and no assertion is relaxed. Tests whose subject moves go
with it:

| from | to |
|---|---|
| `core/tests/runner_tests.rs` (38) | `cli/tests/invocation/entry_point_tests.rs`, same names, argv-driven through `EntryPoint` |
| `settings/args_tests.rs` (13), `reporting/output_format_tests.rs` (4), `reporting/json_printer_tests.rs` (18), `settings/dependency_tables_tests.rs` (13) | `cli/tests/{invocation,reporting,system}/`, same names |
| `source_walker_tests.rs` (11, `walk_*`) | `cli/tests/system/walkdir_source_tree_tests.rs`, renamed `rust_files_*` for the method, same suffixes and assertions |
| `settings/config_file_tests.rs` (15) | `cli/tests/system/toml_config_source_tests.rs`, same names |
| `settings/manifest_resolver_tests.rs` (14 of 17) | `cli/tests/system/cargo_workspace_reader_tests.rs`, same names; the other three stay |
| `adoption/baseline_tests`: the two `load_*` errors and `save_then_load_round_trips` | `cli/tests/system/json_baseline_store_tests.rs`, same names |
| `reporting/rule_listing_tests`: the two JSON tests | `cli/tests/reporting/json_rule_listing_tests.rs`, same names |

Core's runner keeps tests of its own, arranged in memory, and the tests of the
header source, the source reader and the workspace sources arrange an
in-memory tree instead of the temporary directory or this crate's own files,
their assertions kept.

Four printer tests are rewritten because the method they called no longer
exists: `print_with_a_path_far_wider_than_its_heading_does_not_panic`,
`print_with_no_offences_does_not_panic`,
`print_with_offences_from_several_rules_does_not_panic` and
`print_with_one_offence_does_not_panic` become
`render_with_a_path_far_wider_than_its_heading_widens_its_column`,
`render_with_no_offences_says_every_rule_is_satisfied`,
`render_with_offences_from_several_rules_lists_each_with_its_fix` and
`render_with_one_offence_lists_it_with_its_fix`, which assert the exact text
where the old ones asserted only that nothing panicked, on the same offences.

Two resolver tests change with the workspace: the member count of
`packages_without_a_named_package_returns_every_workspace_member` goes from 2
to 3, with `stern4rust-core` asserted by name; and
`workspace_package_names_is_unchanged_by_scoping_the_run` is rewritten,
because the function no longer takes the scope and "scoped equals unscoped"
would compare a value with itself -- it asserts that a scoped selection
returns one package while `workspace_package_names` returns every member.
