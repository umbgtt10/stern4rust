# Implemented Features

What `cargo-stern4rust` does today. Anything not listed here is not built, and
the known gaps in what *is* built are in [OPEN_POINTS.md](OPEN_POINTS.md).

## Version 0.4.0

### Baselines

- `--write-baseline` records the current offences and exits clean;
  `--baseline <PATH>` judges against one. Discovered as
  `stern4rust-baseline.json` beside the manifest when nobody names one.
- Keyed on file + rule + description, **never the line**, so an offence that
  moved is still the same offence.
- Counts rather than a set: fixing one of two identical offences and adding
  another passes; adding a third does not.
- Every run that used one names it and states how many offences it suppressed;
  `baselined=N` in the summary and the JSON. Entries matching nothing are
  reported as stale.
- A baseline that was asked for and is missing is an error, not an empty one.

### Configuration file

- `stern4rust.toml` beside the manifest, holding `header-file`,
  `offence-threshold`, `rules`, `skip`, `exclude` and `baseline`. Every key
  optional.
- The command line wins per setting; for the list settings that is replacement
  rather than merging.
- An unknown key or an unparseable file is an error, not a silently ignored
  line. A missing file is the ordinary case and is not.
- `header-file` resolves relative to the config, so a checkout works anywhere.
- The report names the config it used, and the JSON carries `config_file`.

### Excludability

- `--exclude <GLOB>`, repeatable, matched as a glob against the package-relative
  path, separators normalised so a checked-in pattern works on every platform.
- Every pattern is named in the report with the files it removed, including
  zero; `files_excluded=N` in the summary and an `exclusions` array in the JSON.
- A pattern that matched nothing is called out by name. An uncompilable pattern
  is an error, exit `1`.
- Exclusion happens after the walk rather than by pruning it, because a tree
  that is never entered cannot be counted.

### Rules

- **`registry-completeness`** -- a registry declares every module beside it, so
  no file goes uncompiled. Only the silent direction is checked: `pub mod x;`
  with no `x.rs` is a compile error rustc already reports, while an orphan
  `x.rs` produces no error and no warning. Found 8 never-compiled test files in
  `grip4rust`. [R009](ADRs/R009-ADR-RegistryCompletenessRule.md)
- **`imported-paths`** -- a function is called through a name the file imported,
  not through a path. One imported segment stays legal (`use std::fs;` with
  `fs::read_to_string(...)`); a path no import names does not. Type qualifiers
  are left alone, told apart from modules by case. The first rule to apply to
  test files as well as productive ones.
  [R008](ADRs/R008-ADR-ImportedPathsRule.md)
- **`module-registry`** -- a `lib.rs` or `mod.rs` outside `tests/` holds the
  header, inner attributes, `extern crate alloc;` and `pub mod` declarations
  only. Catches the re-export shim where it forms.
  [R006](ADRs/R006-ADR-ModuleRegistryRule.md)
- **`single-implemented-type`** -- a source file holds at most one type that is
  both declared there and carries an impl block. Plain data declarations are
  unlimited. [R007](ADRs/R007-ADR-SingleImplementedTypeRule.md)
- `RegistryPolicy` -- the shared seam letting one parser answer the registry
  question differently for `src/` and `tests/`.
- `Rule::is_configured` -- a rule answers for itself whether it has what it needs,
  so the registry names no rule in particular and keeps one list of rules rather
  than two.

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
