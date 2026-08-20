# stern4rust

## Meaning

`stern4rust` is a cargo subcommand that holds a Rust workspace to its house
coding rules and names every file that breaks one.

The rules are conventions, not correctness. `rustc` and `clippy` already own
correctness. These are the things a reviewer would otherwise have to say by
hand, every time, forever — and the ones that quietly stop being true across a
codebase the moment nobody is checking.

The rule set is **open by design**: there will be fifteen to twenty of them.
That is the constraint every decision here answers to. Adding a rule must cost
one file under `src/rules/` and one line in `RuleRegistry`, and nothing else.

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

### Mandatory after every change to `src/` or `tests/`

Run gates:

`powershell -File scripts\run_stage_1.ps1`
`powershell -File scripts\run_stage_2.ps1`

If either gate is not green, the work is not complete.

Stage 2 runs `stern4rust` against itself, built from source rather than from
whatever is installed. A tool that enforces a rule it does not satisfy is not
worth installing, so every `.rs` file here carries the header in
`docs/header.txt`.

## Architecture

One rule, one file, one implementation.

- `Rule` (`src/rule.rs`) is the seam. A rule sees a single `SourceFile` and
  answers with what is wrong with it. It does not walk, does not print, and
  does not know which other rules exist.
- `RuleRegistry` (`src/rule_registry.rs`) is the only place that knows the set.
  A rule with nothing to work from is **left out rather than registered and
  silently passing** — a run reporting "all rules satisfied" while a rule was
  never configured is worse than one that says so.
- `Offence` (`src/offence.rs`) is the single currency. Every rule reports in it,
  so the report is one table rather than a section per rule.
- `SourceFile` (`src/source_file.rs`) normalises once so no rule has to: a
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
only reason the end-to-end path is reachable from a test. Only `src/main.rs`
turns a verdict into an exit code.

A named package that does not exist is an **error**, not an empty result. A typo
in a gate script would otherwise scan nothing and report success.

## Publishing

The crate publishes as `cargo-stern4rust` so cargo resolves `cargo stern4rust`,
matching `cargo-crap4rust`, `cargo-twin4rust` and `cargo-iceberg4rust`. The
library is `stern4rust`.

`src/main.rs` strips the repeated subcommand name that cargo inserts as
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
