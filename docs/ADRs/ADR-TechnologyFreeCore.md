# ADR-TechnologyFreeCore

- **Status:** Accepted
- **Date:** 2026-10-28

## Context

The tool is meant to be mutation-tested, and mutation testing pays for every
mutant with a build and a test run. While one crate held everything, a mutant
anywhere rebuilt a crate that linked cargo_metadata, clap, serde, toml and
walkdir, and its tests took twenty-five seconds: the runner's 38 tests spawned
`cargo metadata` and wrote packages to the temporary directory, the resolver's
17 ran cargo on this repository, and the config, header, baseline, walker and
workspace-sources tests wrote to disk. A run asked cargo to describe the
workspace six times. The rules -- what is wrong with a file and how to say so --
are what deserve mutating, and they were mixed with code whose only job is to
talk to the machine.

## Decision

The rules live in a technology-free crate, `stern4rust-core`: `#![no_std]` and
`#![forbid(unsafe_code)]`, depending on globset, proc-macro2, quote and syn
only, declaring a trait for every piece of technology it needs -- the source
tree, the workspace, the config file, the baseline file and the renderer -- and
returning what a run says as a string and its verdict as a `RunOutcome`. A
separate crate, `cargo-stern4rust`, implements those traits with the real
technology, parses the arguments, renders the JSON report, prints, and decides
the exit code.

Three of the five ports are this tool's own. `ConfigSource` and `BaselineStore`
keep TOML and JSON out of the core: the files' meaning -- which setting wins,
how a baseline forgives -- stays in the core under mutation testing, and only
the reading and parsing moves. `SourceTree` writes as well as reads, because
`--fix` repairs files: the rewriters were already pure, so only the write goes
behind the port.

## Forcing constraints / Evidence

`cargo mutants --list` names 2,931 mutants, all in the core, where the tree
before the split named 3,064 (2,956 in the crate, 108 in `xtask`). The rules are
most of the core, so the count barely moves; what a mutant costs does. The
core's 1,537 tests touch no disk and start no process. Measured on the same
machine, on fresh copies of the release before the split and of the split, with
`found > self.limit` turned `>=` in `DirectoryFileCountRule` as cargo-mutants
turns it, medians of three at a load average of one to four: a mutant's build
and test run went from 26.8 s to 1.7 s, the suite alone from 25.7 s to 0.29 s,
and a clean build of the package's tests -- what each cargo-mutants worker pays
first -- from 23.6 s to 15.1 s.

The binary's behaviour did not move: stdout, stderr and exit codes were compared
byte for byte against 0.18.0 on 77 invocations across twelve repositories after
every step. Two exceptions are deliberate: a `.rs` file whose name is not UTF-8
fails the run naming it, where 0.18.0 judged it under a lossy name, and a
`--manifest-path`, `--header-file` or `--baseline` that is not UTF-8 fails
naming the flag.

## Rejected alternatives

- **One crate, with cargo-mutants' `exclude_globs` on the technology.** Scopes
  the mutants but not the build or the tests, and nothing would stop the next
  change reaching for `std::fs` beside a rule.
- **globset behind a port.** It is not `no_std` at 0.4.19 but compiles under
  the core's `#![no_std]`, costs a second on a clean dependency build and
  nothing per mutant; a port would take two traits, an adapter and a glob-like
  fake, and move the exclusion tests -- `fixture/**` not matching
  `fixtures_of_mine/` -- out of mutation testing.
- **The config read through `SourceTree`, with a parser port.** Keeps "a
  missing file is none" under mutation testing, at the cost of a third
  TOML-facing seam; the adapter owns exists, read and parse, and the core words
  the two failures.
- **An output port for stdout.** The core returns a string; printing it is the
  composition root's one line.
- **Leaving the text report to `cli`.** It is pure string building, and the
  order, widths and summary it prints are worth mutating; it is the core's
  implementation of the `ReportRenderer` port.

## Consequences

Two published crates, `stern4rust-core` first, sharing one version; the
package name `cargo-stern4rust` and the binary are unchanged, so every
repository's `cargo install cargo-stern4rust` keeps working. A new technology
need is a trait in `core/src/traits/`, an in-memory fake in
`core/tests/support/` and an adapter in `cli/src/system/`, composed in
`EntryPoint`. Paths in the core are `SourcePath`s with `/` as their only
separator, converted by `cli` at the edge; their order is `PathBuf`'s, so the
order of tied offences in a report did not move. The core's errors are typed
and keep their causes, so `cli`'s `main` prints the chain it always printed.
The library surface is still not a public API --
[ADR-LibrarySurfaceIsNotAnApi](ADR-LibrarySurfaceIsNotAnApi.md) now speaks for
both crates.

## Enforcement

The "Technology-free core" gate, first in stage 2: core's manifest names only
globset, proc-macro2, quote and syn in any section; `core/src/lib.rs` carries
`#![no_std]` and `#![forbid(unsafe_code)]`; no file under `core/src` or
`core/tests` reaches `std::{env,fs,io,net,os,process}`, a printing macro or a
file-system method of `Path`. `.cargo/mutants.toml` scopes cargo-mutants to
`core/src`, each mutant tested by core's tests only.

## Related

- [ADR-LibrarySurfaceIsNotAnApi](ADR-LibrarySurfaceIsNotAnApi.md)
- [`../ARCHITECTURE.md`](../ARCHITECTURE.md)
- [`../history/[IMPLEMENTED] CORE_CLI_SPLIT.md`](../history/%5BIMPLEMENTED%5D%20CORE_CLI_SPLIT.md)
