# stern4rust

## Meaning

`stern4rust` is a cargo subcommand that holds a Rust workspace to its house
coding rules and names every file that breaks one.

The rules are conventions, not correctness. `rustc` and `clippy` already own
correctness. These are the things a reviewer would otherwise have to say by
hand, every time, forever — and the ones that quietly stop being true across a
codebase the moment nobody is checking.

The rule set is **open by design**: there will be fifteen to twenty of them, and
there are thirty-eight today, eighteen past the range, and `core/src/rules/` is grouped into
subfolders because `directory-file-count` would not allow a twenty-first file. That is the constraint every decision here answers to.
Adding a rule costs a file under `core/src/rules/`, its `pub mod` line, one entry in
`RuleCatalog::all`, a mirrored test file, an `R<NNN>` ADR and a `RULES.md`
section — and nothing in the walker, the printer or any other rule.

`docs/RULES.md` is the canonical policy description. If a rule's meaning
changes, update `docs/RULES.md`, the rule and its tests together.

It is self-contained.

## Boundary Rule

This repository is **SELF-CONTAINED**.

The LLM **SHALL NOT cross its boundaries without asking**.

That means:
- do not inspect, edit, or rely on files outside `stern4rust/` unless the user explicitly asks
- do not pull assumptions from sibling repositories or crates
- do not propose cross-repository changes by default

## Quality Gates

### Mandatory after every change to `core/`, `cli/` or `xtask/`

Run gates:

`just stage1`
`just stage2`

If either gate is not green, the work is not complete.

Both run identically on Windows, Linux and macOS, and CI runs the same two
commands -- there is no second definition of the gates to drift out of step.

Stage 1 is formatting, clippy and tests -- cargo built-ins only, so it works on
a fresh checkout with none of the house tools installed.

Stage 2 is `cargo xtask stage2` -- a real crate under `xtask/`, gated like any
other code, rather than a script. Each gate is a `Gate` implementation
constructed against a `CommandRunner` trait, so the argument lists and the
failure messages are covered by `xtask`'s own integration tests. It runs eight
gates, in this order:

| gate | asks |
|---|---|
| Technology-free core | does `core` still touch no technology |
| own rules | does this tool satisfy the rules it enforces |
| `cargo shear` | is every declared dependency used, in the section that uses it |
| `cargo dry4rust` | did this change add duplicated code |
| `cargo crap4rust` | is any function complex and untested |
| `cargo twin4rust` | does every source file have a mirrored test file |
| `cargo iceberg4rust` | is any file's private implementation risk too high |
| `cargo deny` | do the dependencies carry an advisory, a refused licence or a foreign source |

The technology-free gate runs **first**. It is the xtask's own -- no tool to
install -- and asks three things of `core`: that its manifest names only
`globset`, `proc-macro2`, `quote` and `syn`, in any section (read from `cargo
metadata --no-deps`; a package missing from the metadata fails); that
`core/src/lib.rs` carries `#![no_std]` and `#![forbid(unsafe_code)]`, which
hand `core/src` to the compiler; and that no file under `core/src` or
`core/tests` reaches `std::{env,fs,io,net,os,process}`, a printing macro or a
file-system method of `Path`, read with syn so a string literal -- every rule's
example is one -- is data. The `core/src` scan is a backstop to the compiler;
`core/tests`, where std is linked, depends on it. Calls inside a macro's
arguments are not seen, and a method of one's own named like a file-system one
would be flagged -- which is why the port asking whether a path exists calls it
`contains`. Its fix is moving code into `cli`, which changes what every gate
behind it measures, and it takes a second.

The own-rules gate runs **second**, behind the boundary and ahead of the rest,
where the PowerShell script ran it last. Its corrections are renames, file moves
and directory splits, so a layout it is about to reject is one the others would
have measured for nothing -- the same reason every sibling repository puts
`stern4rust` ahead of the rest. It caught a misordered test in `xtask` on its
first run in that position.

cargo-shear runs **third**: its corrections edit manifests only -- a
dependency removed, or moved to the section that uses it -- so they change
nothing the gates behind it measure, and it takes a second. It runs `--locked`,
reading the lock file and never writing it, and `--deny-warnings`, so an
optional dependency no feature's code uses fails as an unused one does. A
dependency it cannot see used -- linked only for what it provides, declared
only to switch on a feature, used only by generated code -- goes in
`[package.metadata.cargo-shear] ignored`, and the gate refuses an entry without
its reason in the same table's `reasons`; there are none today. Pinned at
1.14.0, as CI installs it.

`dry4rust` runs **fourth** for the same reason the own rules run early:
removing a duplicate moves code between files, which changes what the three
behind it measure. It makes one run from the workspace root over both published
crates' sources, so a copy from one into the other is seen, with the trees it
must not judge -- `xtask`, `core/tests`, `cli/tests`; tests repeat their
arrangement by design -- excluded by full path, because dry4rust matches an
exclusion as a substring of a file's path and a bare `tests` would also exclude
a checkout under a directory of that name. It checks with zero ceilings against
`dry4rust-baseline.json`, the duplication already there when the gate arrived. So it fails on what a change adds, not on what it
inherited; a duplicate removed is admitted, a copy added to a recorded group is
not.

It counts only code units of 25 AST nodes or more. Below that sit units whose
body is a single call: the accessors every rule must write -- `explanation`,
`check_workspace`, `applies_to` -- and a `check` that hands its two halves to
`SourceItems::offences`. Each such unit carries the `Rule` signature, so any two
look alike; at dry4rust's default of 10 they form groups that every rule added
would grow, failing the gate for a rule set that is open by design. 25 leaves
them out and still catches a rule's twelve-line `check` copied from another's,
which is what this gate first caught.

Re-record the baseline only to drop groups that are gone, never to admit new
ones, and at the same floor -- a baseline matches only at the floor it was
recorded at, and over the same trees: `cargo dry4rust --path "$PWD" --exclude "$PWD/xtask" --exclude "$PWD/core/tests" --exclude "$PWD/cli/tests" --min-nodes 25 --baseline "$PWD/dry4rust-baseline.json" baseline`.

It runs `stern4rust` against itself, built from this checkout's `cli` rather
than from whatever is installed. A tool that enforces a rule it does not satisfy is not worth
installing -- and a rule this checkout breaks is a rule it is about to publish
-- so every `.rs` file here carries the header in `docs/header.txt`.

All three members are scanned: `stern4rust-core`, `cargo-stern4rust` and
`xtask`. The crate that runs the gates is not exempt from the rules it exists to
enforce. crap4rust, twin4rust and iceberg4rust measure the two published crates,
`stern4rust-core` and `cargo-stern4rust`; `xtask` is not their bill.

cargo-deny runs **last**, and is the one gate that can turn red with nothing
changed: its advisories come from the RustSec database, so a newly published
one fails an untouched checkout, and it fetches that database over the network.
That is the point of it, and running last keeps it from hiding what the house
gates say about a change. `deny.toml` is the same file in every *4rust
repository and slotgate: licences MIT, Apache-2.0 and Unicode-3.0 -- the union
of what their trees carry -- crates.io as the only source, duplicate versions a
warning, and a wildcard requirement refused except a path dependency in an
unpublished crate. An accepted advisory goes in `[advisories] ignore` with its
id and its reason. The gate fetches the advisory database first, on its own and
up to three times, then checks `--frozen` -- locked and offline -- so a fetch
that keeps failing reports the gate as unable to run, never as an advisory
found. It decodes cargo-deny's exit code, one bit per failed check, into the
checks' names; a `deny.toml` it cannot read exits 1, as advisories does, and the
streamed output says which.

`cargo install just`
`cargo install cargo-llvm-cov`
`cargo install cargo-dry4rust`
`cargo install cargo-crap4rust`
`cargo install cargo-twin4rust`
`cargo install cargo-iceberg4rust`
`cargo install --locked cargo-shear --version 1.14.0`
`cargo install --locked cargo-deny --version 0.20.2`

cargo-stern4rust is deliberately not in that list; the gate builds it from this
checkout.

All thirty-eight rules are enforced, with nothing skipped and nothing
unconfigured. `stern4rust.toml` names the header file, rather than the gate
script passing `--header-file`, so a hand-run of `cargo stern4rust` checks
exactly what the gate checks -- which is what every other repository in this
family does. The flag still overrides the config and is still covered by tests;
it simply is not the only way in any more.

One consequence is worth knowing before writing a test here. Any test that
needs the *absence* of a config cannot point at this repository's own manifest,
because a `stern4rust.toml` now sits beside it. `probe_package` in
`cli/tests/invocation/entry_point_tests.rs` builds a package without one for
exactly that reason.

## Layout

The repository is a workspace of three members:

| member | what it is |
|---|---|
| `core/` | the rules: package `stern4rust-core`, library `stern4rust`. `#![no_std]`, `#![forbid(unsafe_code)]`, no technology -- see "Technology-free core" below. Published. |
| `cli/` | the command: package `cargo-stern4rust`, library `stern4rust_cli`, binary `cargo-stern4rust`. `Args`, the composition root `EntryPoint`, the JSON report, and every adapter. Published. |
| `xtask/` | gate orchestration, run as `cargo xtask`. `publish = false`. |

`core` and `cli` share the workspace's `version`. `[workspace.lints.rust]`
forbids `unsafe_code` in every member and every target; `core` says so at its
crate root as well.

Giving each crate a directory of its own is load-bearing rather than
tidy-minded. While the crate sat at the repository root, its package directory
*was* the repository root, so a scan of the package walked `xtask/tests/**` and
reported its test functions as living "in the source tree" -- files belonging to
a different package entirely. `--package` does not narrow it, because the scope
is the directory. Giving each crate its own directory is what separates them.

Tests in `cli/tests/system/cargo_workspace_reader_tests.rs` assert facts about
this repository's own layout, so a change to it changes their answers rather
than breaking them: an unnamed scan resolves to all three members, each named,
the root declares `[workspace.dependencies]`, and the workspace root is one
directory above the crate.

## Architecture

One rule, one file, one implementation.

- `Rule` (`core/src/rule.rs`) is the seam. A rule sees a single `SourceFile` and
  answers with what is wrong with it. It does not walk, does not print, and
  does not know which other rules exist.
- `RuleCatalog` (`core/src/rule_catalog.rs`) is the only place that knows the set,
  and composes it for a run; `RuleRegistry` (`core/src/rule_registry.rs`) and
  `RewriterRegistry` take the parts it builds, as `injected-dependencies` asks.
  A rule with nothing to work from is **left out rather than selected and
  silently passing** — a run reporting "all rules satisfied" while a rule was
  never configured is worse than one that says so.
- `Offence` (`core/src/reporting/offence.rs`) is the single currency. Every rule reports in it,
  so the report is one table rather than a section per rule.
- `SourceFile` (`core/src/source_file.rs`) normalises once so no rule has to: a
  trailing carriage return and a leading byte order mark are stripped. Without
  that, every file on a Windows checkout fails a rule that is really about
  content, and never fails on the maintainer's machine.
- `Runner` (`core/src/runner.rs`) is the run: built over the core's five ports,
  it reads an `Invocation` -- what the command line asked -- and returns the
  verdict and what the run has to say. It does not print.

A rule's expected values are **data, not constants**. The header differs by
repository — MIT here, Apache 2.0 in the etheram repositories — and a rule that
hardcoded one would be right for exactly one codebase.

## Technology-free core

`core` touches no technology -- no cargo, file system, process, environment,
argument parsing, serialisation or stdout -- so mutation testing it needs
nothing but its own tests, which run in under a tenth of a second.
`.cargo/mutants.toml` scopes cargo-mutants to `core/src`, each mutant tested by
core's tests only.

Whatever technology core needs is a port: a trait in `core/src/traits/` --
`SourceTree`, `WorkspaceReader`, `ConfigSource`, `BaselineStore`,
`ReportRenderer` -- the records it speaks in beside their users
(`settings/invocation.rs`, `settings/workspace_description.rs`,
`reporting/run_report.rs`), an in-memory fake for core's tests in
`core/tests/support/`, and the real adapter in `cli/src/system/` (the JSON
renderer in `cli/src/reporting/`), composed in `cli`'s `EntryPoint`. A new need
is the same three pieces: trait, fake, adapter. Paths in core are
`SourcePath`s, text whose only separator is `/`; `cli`'s `PortablePath`
converts at the edge, and a path that is not UTF-8 fails the run rather than
being judged under a lossy name. core's failures are typed (`RunFailure` and the
failures it wraps) and keep their causes, so `cli`'s `main` prints the same
chain anyhow always printed.

## Exit codes

| code | meaning |
|---|---|
| `0` | every rule satisfied |
| `1` | could not run — bad manifest, unknown package, unreadable source |
| `2` | at least one rule broken |

`2` is kept distinct from `1` deliberately. A wrapper that treats every non-zero
code alike cannot tell "your code has a problem" from "I could not read your
code", and the second silently passing is how a gate stops meaning anything.

`Runner::run` returns a `RunOutcome` rather than calling `exit`, which is the
only reason the end-to-end path is reachable from a test. Only `cli/src/main.rs`
turns a verdict into an exit code.

A named package that does not exist is an **error**, not an empty result. A typo
in a gate script would otherwise scan nothing and report success.

## Publishing

Two crates publish. The command publishes as `cargo-stern4rust` so cargo
resolves `cargo stern4rust`, matching `cargo-crap4rust`, `cargo-twin4rust`,
`cargo-iceberg4rust` and `cargo-dry4rust`; it depends on `stern4rust-core`, the
rules, whose library is `stern4rust`. A release publishes `stern4rust-core`
first (`cargo publish --workspace` orders them), and every repository's
`cargo install cargo-stern4rust` keeps working, the package name unchanged.

`EntryPoint` strips the repeated subcommand name that cargo inserts as
`argv[1]`; running the binary directly does not repeat it, so the strip is
conditional. It is also positional — dropping every occurrence would swallow
`--package stern4rust`. `Args::without_cargo_subcommand`, in
`cli/src/invocation/args.rs`, owns that rule and both properties are tested.

Before publishing, `cargo publish --workspace --dry-run` must succeed.

## Orthogonality, trait surface and cognitive complexity

**When changing productive code, always maximize orthogonality and testable surface through traits, and minimize cognitive complexity.**

Specifically:
- prefer extracting behavior behind traits so individual pieces can be tested and swapped independently
- prefer small, focused methods with a single responsibility over large methods with many branches
- prefer named structs with methods over free functions operating on external state
- when `crap4rust` or a reviewer flags a function as too complex, reduce it by extracting internal structs with methods and adding integration coverage — not by extracting standalone helper functions
- never increase cognitive complexity to pass a test; find the root cause and fix it there
- make constructors depend on traits, not directly on concrete implementations
- ALL dependencies are injected through the SINGLE constructor and stored in the struct

## User coding standards

- one struct per file
- no unnecessary comments in code
- unit tests are not allowed. Only integration tests are
- consolidate scattered functions inside structs as appropriate
- no `&mut` input parameters; prefer return values
- only use `pub mod` in `mod.rs` and `lib.rs`
- split test files so there is one test file per source file, named `<source file name>_tests.rs`
- in `all_tests.rs`, reference test files one by one without `#[path = ...]`
- apply AAA (`Arrange`, `Act`, `Assert`) structure to tests with blank-line separation between the three sections
- use `// Arrange & Act` if there is no separate `Arrange`
- use `// Act & Assert` if there is no separate `Act`
- use `// Arrange & Act & Assert` if none of the three is separate
- a test may hold several `Act`/`Assert` pairs after its `Arrange`
- a marker may carry trailing prose after `--`, `:` or `.`
- add the repository copyright and license header to every Rust source file
- tests should be named as follows `<method under test>_<test description>_<result>`
- do not use fully qualified paths; use `use` imports instead
