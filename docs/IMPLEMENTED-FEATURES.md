# Implemented Features

What `cargo-stern4rust` does today. Anything not listed here is not built, and
the known gaps in what *is* built are in [OPEN_POINTS.md](OPEN_POINTS.md).

## Version 0.4.0

### Rules

- **`module-registry`** -- a `lib.rs` or `mod.rs` outside `tests/` holds the
  header, inner attributes, `extern crate alloc;` and `pub mod` declarations
  only. Catches the re-export shim where it forms.
  [R006](ADRs/R006-ADR-ModuleRegistryRule.md)
- `RegistryPolicy` -- the shared seam letting one parser answer the registry
  question differently for `src/` and `tests/`.

## Version 0.3.0

### Rule selection

- `--rule <NAME>` and `--skip <NAME>`, both repeatable. `--rule` selects,
  `--skip` subtracts, skipping wins. Default is every rule, nothing excluded.
- Every report names the rules it applied, and names the ones it did not with
  the reason: `(skipped)` or `(needs --header-file)`. Three states, not two.
- `rules_applied`, `rules_skipped`, `rules_unconfigured` in the summary line and
  the JSON.
- An unknown rule name and `--rule header` without `--header-file` are both
  errors (exit `1`) rather than switches that quietly match nothing.

## Version 0.2.0

### Rules

- **`readable-source`** — every `.rs` file can be read and parsed. Registered
  first, because its failure explains every other rule's silence about the same
  file. [R004](ADRs/R004-ADR-ReadableSourceRule.md)
- **`header`** — every `.rs` file opens with the repository's header, supplied
  by `--header-file`. Exact after normalisation; one offence per file, carrying
  the whole expected header. [R001](ADRs/R001-ADR-HeaderRule.md)
- **`test-file-structure`** — header, imports, constants, helpers, tests; each
  group alphabetical; spacing part of the shape. `Helpers` defined by exclusion.
  [R002](ADRs/R002-ADR-TestFileStructureRule.md)
- **`test-free-source`** — no `#[test]`, `#[cfg(test)]` or `#[cfg_attr(test, …)]`
  outside `tests/`. Feature gates untouched.
  [R005](ADRs/R005-ADR-TestFreeSourceRule.md)
- **`tests-layout`** — one `tests/all_tests.rs`, a `mod.rs` in every subfolder
  on the way down, both holding only the header and `pub mod` declarations.
  [R003](ADRs/R003-ADR-TestsLayoutRule.md)

### Analysis

- AST-structural via `syn`, with `span-locations` for item positions. No type
  resolution, no build required — the tool runs on any syntactically valid
  source.
- Comment folding: `//` lines above an item count as part of that item, so a
  documented test is not a spacing offence.
- Predicate scanning for `cfg`/`cfg_attr`, by identifier rather than substring,
  so `any(test, …)` and `not(test)` are caught while `feature = "test"` is not.
- Normalisation once, in `SourceFile`: BOM stripped, `\r` stripped, paths
  forward-slashed. No rule can forget it.
- Whole-package read before judging, which the tree-wide rules require and which
  removes any dependence on walker order.

### Resolution

- `--manifest-path` and `--package`, resolved through `cargo_metadata`.
- `target/`, `.git/` and **any directory holding its own `Cargo.toml`** are not
  walked. A nested package is a different package.

### Reporting

- Text table, columns sized to contents, each offence followed by an indented
  `fix:` line.
- `--format json` — the same run as a document: `files_scanned`,
  `offences_found`, `offences_reported`, `offences_omitted`,
  `offence_threshold`, `rules_broken`, and the offence array. Every key always
  present, so the shape never varies.
- Every offence carries a **required** `correction`, plus optional `subject` and
  `expected`.
- `--offence-threshold N` (default 100, `0` for all) caps what is **printed**.
  The summary, the omitted count and the exit code all see every offence.
- Offences sorted by file, then line, then rule.
- Exit codes `0` / `1` / `2`, with `2` distinct from `1` so a wrapper can tell a
  finding from a broken CI step.

### Packaging

- Publishes as `cargo-stern4rust` with a `stern4rust` library, so cargo resolves
  `cargo stern4rust`.
- `Args::without_cargo_subcommand` — the argv fixup every cargo subcommand
  needs, conditional and positional, so a package named `stern4rust` survives.

### Project

- 265 tests, one test file per source file, `autotests = false` with a single
  `[[test]] all_tests`.
- Two gates: stage 1 (fmt, clippy `-D warnings`, tests), stage 2 (`crap4rust`,
  `twin4rust`, `iceberg4rust`, and stern4rust against its own tree).
- Self-gating: a rule that would fail this repository cannot be merged into it.
- Ten ADRs — five `R` rule ADRs, five unnumbered.
