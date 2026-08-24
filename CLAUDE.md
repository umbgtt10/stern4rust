# stern4rust

## Meaning

`stern4rust` is a cargo subcommand that holds a Rust workspace to its house
coding rules and names every file that breaks one.

The rules are conventions, not correctness. `rustc` and `clippy` already own
correctness. These are the things a reviewer would otherwise have to say by
hand, every time, forever — and the ones that quietly stop being true across a
codebase the moment nobody is checking.

The rule set is **open by design**: there will be fifteen to twenty of them, and
there are twenty-one today, one past the range, and `core/src/rules/` is grouped into
subfolders because `directory-file-count` would not allow a twenty-first file. That is the constraint every decision here answers to.
Adding a rule costs a file under `core/src/rules/`, its `pub mod` line, one entry in
`RuleRegistry::all`, a mirrored test file, an `R<NNN>` ADR and a `RULES.md`
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

### Mandatory after every change to `core/src/` or `core/tests/`

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
failure messages are covered by `xtask`'s own integration tests. It runs four
gates, in this order:

| gate | asks |
|---|---|
| own rules | does this tool satisfy the rules it enforces |
| `cargo crap4rust` | is any function complex and untested |
| `cargo twin4rust` | does every source file have a mirrored test file |
| `cargo iceberg4rust` | is any file's private implementation risk too high |

The own-rules gate runs **first**, where the PowerShell script ran it last. Its
corrections are renames, file moves and directory splits, so a layout it is
about to reject is one the other three would have measured for nothing -- the
same reason every sibling repository puts `stern4rust` ahead of the rest. It
caught a misordered test in `xtask` on its first run in that position.

It runs `stern4rust` against itself, built from source rather than from whatever
is installed. A tool that enforces a rule it does not satisfy is not worth
installing -- and a rule this checkout breaks is a rule it is about to publish
-- so every `.rs` file here carries the header in `docs/header.txt`.

Both members are scanned: `cargo-stern4rust` and `xtask`. The crate that runs
the gates is not exempt from the rules it exists to enforce.

`cargo install just`
`cargo install cargo-llvm-cov`
`cargo install cargo-crap4rust`
`cargo install cargo-twin4rust`
`cargo install cargo-iceberg4rust`

cargo-stern4rust is deliberately not in that list; the gate builds it from this
checkout.

All twenty-one rules are enforced, with nothing skipped and nothing
unconfigured. `stern4rust.toml` names the header file, rather than the gate
script passing `--header-file`, so a hand-run of `cargo stern4rust` checks
exactly what the gate checks -- which is what every other repository in this
family does. The flag still overrides the config and is still covered by tests;
it simply is not the only way in any more.

One consequence is worth knowing before writing a test here. Any test that
needs the *absence* of a config cannot point at this repository's own manifest,
because a `stern4rust.toml` now sits beside it. `probe_package` in
`core/tests/runner_tests.rs` builds a package without one for exactly that reason.

## Layout

The repository is a workspace: `core/` is the published crate and `xtask/` runs
the gates. That split is load-bearing rather than tidy-minded. While the crate
sat at the repository root, its package directory *was* the repository root, so
a scan of the package walked `xtask/tests/**` and reported its test functions as
living "in the source tree" -- files belonging to a different package entirely.
`--package` does not narrow it, because the scope is the directory. Giving each
crate its own directory is what separates them.

Three tests in `core/tests/settings/manifest_resolver_tests.rs` assert facts
about this repository's own layout, so the split changed their answers rather
than breaking them: an unnamed scan now resolves to two members, the root now
declares `[workspace.dependencies]`, and the workspace root is now one directory
above the crate. All three were renamed to say what they now check.

## Architecture

One rule, one file, one implementation.

- `Rule` (`core/src/rule.rs`) is the seam. A rule sees a single `SourceFile` and
  answers with what is wrong with it. It does not walk, does not print, and
  does not know which other rules exist.
- `RuleRegistry` (`core/src/rule_registry.rs`) is the only place that knows the set.
  A rule with nothing to work from is **left out rather than registered and
  silently passing** — a run reporting "all rules satisfied" while a rule was
  never configured is worse than one that says so.
- `Offence` (`core/src/reporting/offence.rs`) is the single currency. Every rule reports in it,
  so the report is one table rather than a section per rule.
- `SourceFile` (`core/src/source_file.rs`) normalises once so no rule has to: a
  trailing carriage return and a leading byte order mark are stripped. Without
  that, every file on a Windows checkout fails a rule that is really about
  content, and never fails on the maintainer's machine.

A rule's expected values are **data, not constants**. The header differs by
repository — MIT here, Apache 2.0 in the etheram repositories — and a rule that
hardcoded one would be right for exactly one codebase.

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
only reason the end-to-end path is reachable from a test. Only `core/src/main.rs`
turns a verdict into an exit code.

A named package that does not exist is an **error**, not an empty result. A typo
in a gate script would otherwise scan nothing and report success.

## Publishing

The crate publishes as `cargo-stern4rust` so cargo resolves `cargo stern4rust`,
matching `cargo-crap4rust`, `cargo-twin4rust` and `cargo-iceberg4rust`. The
library is `stern4rust`.

`core/src/main.rs` strips the repeated subcommand name that cargo inserts as
`argv[1]`; running the binary directly does not repeat it, so the strip is
conditional. It is also positional — dropping every occurrence would swallow
`--package stern4rust`. `Args::without_cargo_subcommand` owns that rule and both
properties are tested.

Before publishing, `cargo publish --dry-run` must succeed.

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
