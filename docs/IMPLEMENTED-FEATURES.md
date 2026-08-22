# Implemented Features

What `cargo-stern4rust` does today. Anything not listed here is not built, and
the known gaps in what *is* built are in [OPEN_POINTS.md](OPEN_POINTS.md).

## Version 0.10.3

### Rules

- `ordered-imports` judges and quotes an attributed import by its path rather
  than by the `#[cfg(...)]` above it. An item's span starts at its first
  attribute, which had made the gate the sort key.
  [R019](ADRs/R019-ADR-OrderedImportsRule.md)

## Version 0.10.2

### Reporting

- A workspace run counts every member's offences. Deduplication is per package
  rather than per run, so two members sharing a package-relative path -- and in
  a large workspace they nearly all share `src/lib.rs` and `tests/all_tests.rs`
  -- no longer lose the second finding. Found in `etheram-embassy`: 390
  offences reported as 364, across four rules.

## Version 0.10.1

### Rules

- `test-free-source` reads a `cfg` predicate with its polarity. `not(test)`
  gates an item out of the test build, making it production-only, and is no
  longer reported as test code in the source tree. `any(test, ...)` is
  unchanged. [R005](ADRs/R005-ADR-TestFreeSourceRule.md)

## Version 0.10.0

### Discoverability

- **`--rules`** lists every rule with a line saying what it wants, a scrap of
  source that breaks it and the same scrap put right, then exits without
  scanning. All twenty-one, in registry order, so the listing reads in the same
  order as the roster a report prints. Honours `--format`, so the same listing
  is available as JSON. Nothing is walked, read or judged, so it works in a
  checkout with no manifest worth reading and always exits clean.
  [ADR-RulesExplainThemselves](ADRs/ADR-RulesExplainThemselves.md)
- Every rule the registry can hold is listed, including the two that stay out
  of an ordinary run until configured -- `header` and `spdx-matches-manifest`.
  A listing missing a rule reads as a tool that does not have it.
- `--manifest-path` and `--package` carry help text. They were the only two
  flags printing an empty description.

### Rules

- **`imported-paths` names the primitives instead of guessing at them.**
  `u64::from_le_bytes` was reported with the correction `use
  u64::from_le_bytes;`, which does not compile at all. `u8` through `u128`, the
  signed and pointer-sized forms, `f32`, `f64`, `bool`, `char` and `str`,
  matched whole. [R008](ADRs/R008-ADR-ImportedPathsRule.md)
- **`ordered-imports` reads case as a shape rather than as a first letter.**
  `WAL_V2_MAGIC` beside `WalRecord` both open with a capital and the two style
  editions order them oppositely, so the pair stands down like every other
  cross-case pair. [R019](ADRs/R019-ADR-OrderedImportsRule.md)

## Version 0.9.1

### Rules

- **`tested-public-api` counts two shapes, not three.** A free `pub fn` and a
  `pub fn` in an inherent impl. Neither half of a trait counts: an implementing
  method is reached through the trait rather than named, and a declared one has
  no behaviour behind it to test. Across the family, 72 offences to 34.

## Version 0.9.0

### Reporting

- A rule that could not run **says what it was waiting for**:
  `header (needs --header-file)`,
  `spdx-matches-manifest (needs a `license` field in Cargo.toml)`.
  `Rule::requirement` answers it, so the printer names no rule in particular.
- `tests-layout` reports a registry it cannot parse as **unchecked** rather than
  saying nothing. One offence, not one per sibling file.

### Baselines

- Keyed on file + rule + **subject**, not the description. A rule may reword an
  offence without every baseline in the family reporting it as new.

### Structure

- `src/finding/` is grouped into `model/` and `parsing/`, mirrored in `tests/`.
- `ItemNaming` holds the identifier an item declares and the source-line
  fallback, where three parsers each carried both.

### Library surface

- **Decided: the library is not a public API.** Depend on the binary. Everything
  is `pub` because this repository forbids unit tests, so the surface is as wide
  as the test suite needs and no wider a promise than that.
  [ADR-LibrarySurfaceIsNotAnApi](ADRs/ADR-LibrarySurfaceIsNotAnApi.md)

## Version 0.8.0

### Rules

- **`workspace-dependencies`** -- a workspace declares its dependencies once, in
  the root, and every member takes them with `.workspace`. The three
  requirements asked for are one check, because `cargo` refuses to build a
  `workspace = true` the root does not declare. Read from the TOML, since
  resolution erases how a dependency was written; all three dependency tables
  count; intra-workspace path dependencies are included. 27 offences across
  three workspaces, and silent on the five repositories that are not workspaces.
  [R021](ADRs/R021-ADR-WorkspaceDependenciesRule.md)

- **`ordered-imports`** -- imports in `src/` run in alphabetic order, reusing
  the `ImportPath` stand-down so the rule and `cargo fmt` cannot disagree.
  Verified against rustfmt by controlled experiment before it was written. 10
  offences, all in `etheram-ibft`. Records that 56% of `src/` import pairs stand
  down. [R019](ADRs/R019-ADR-OrderedImportsRule.md)
- **`spdx-matches-manifest`** -- every file's SPDX identifier says what the
  manifest's `license` says. The first rule configured from the package being
  judged rather than the command line, so it needs no `--header-file`; a
  manifest naming no licence leaves it unconfigured rather than offended.
  [R020](ADRs/R020-ADR-SpdxMatchesManifestRule.md)

### Structure

- `src/rules/` is grouped into `source/`, `layout/`, `testing/` and `manifest/`,
  with `tests/rules/` mirroring exactly. `directory-file-count` would not allow a
  twenty-first file in one directory, so the tool's own rule decided the layout.

### Reporting

- Offences are deduplicated by content, per package. The workspace question is
  asked once per package root, so a rule whose subject is the workspace rather
  than the package stated each finding once per member -- 36 where there were 6.
  Per package rather than per run because a path is relative to its package: two
  members' `src/lib.rs` are two files with one name, and collapsing across the
  run discarded the second.
- An unconfigured rule now reads `(not configured)` rather than
  `(needs --header-file)`. Two rules can go unconfigured, and the header rule is
  not what the other is waiting on.

## Version 0.7.0

### Rules

- **`declared-by-name`** -- a module is declared by name; `#[path = "..."]` on a
  `mod` is an offence, package-wide. It exists to hold up the convention
  `registry-completeness` resolves by: without it, a file reached through an
  explicit path is reported as never compiled when it compiles fine. Three
  documents described that standard and no rule enforced it. `cfg_attr`-gated
  paths are left alone as the one honest use. Zero offences across ten
  repositories. [R018](ADRs/R018-ADR-DeclaredByNameRule.md)
- **`arrange-act-assert`** -- a test reads `Arrange`, then one or more
  `Act`/`Assert` pairs, sections separated by a blank line. The original
  motivating example for the tool, shipped seventeenth. Markers expand to the
  phases they name and the expansion must match, so the merged forms need no
  special case. The markers are comments, which `syn` discards, so the rule
  reads lines and skips every line a literal occupies -- without which it
  reports this crate's own Rust-in-a-raw-string fixtures. 45 offences across the
  family, 0 here. [R017](ADRs/R017-ADR-ArrangeActAssertRule.md)
- **`paired-test-file`** -- a `tests/a/b_tests.rs` names the source file it
  exercises and `src/a/b.rs` exists, matched by path rather than by name alone.
  The other side of the pairing from `twin4rust`, and the direction nothing
  checked: a test file outlives the module it was named for silently, still
  compiling and still passing. Found 40 tests in four such files in
  `etheram-ibft/node`, and independently flagged the same eight `grip4rust`
  files `registry-completeness` found never compiled. `all_tests.rs` and
  `_proptest_tests.rs` are exempt; a harness crate skips the rule.
  [R016](ADRs/R016-ADR-PairedTestFileRule.md)
- **`test-file-name-postfix`** -- a file under `tests/` holding at least one
  test is named `<X>_tests.rs`. Closes the side of the mirrored-layout pairing
  nothing enforced: `twin4rust` starts from a source file and looks for its
  test, so a file full of tests under any other name was invisible to every tool
  in the family. One direction only. `src/` and registries are exempt, because
  the rule that owns each already reports them and this rule's correction would
  be wrong for both. Zero offences across the family.
  [R015](ADRs/R015-ADR-TestFileNamePostfixRule.md)

## Version 0.6.0

### Rules

- **`test-naming`** -- a test's name has at least three underscore-separated
  parts, following `<method>_<conditions>_<result>`. The name and nothing else:
  three earlier versions tried to verify the leading part was the method under
  test and all three accused correct code across 1559 tests, taking 592 offences
  down to 5. [R012](ADRs/R012-ADR-TestNamingRule.md)
- **`tested-public-api`** -- every public entry point in `src/` is called by at
  least one test, matched on name and arity. Call sites are gathered from macro
  token streams as well as parsed expressions, since a test's assertion lives in
  `assert!` and never becomes syntax. Found six printer builders shipped
  untested in 0.4.0. Under-reports by design.
  [R013](ADRs/R013-ADR-TestedPublicApiRule.md)
- **`pure-traits`** -- a trait declares, it does not implement: no method in a
  `trait` declaration in `src/` may have a default body. The other direction,
  that every implementor implements every method, is `rustc`'s `E0046` and needs
  no rule -- the same split `registry-completeness` made. Four offences across
  the family, three of them in this crate's own `Rule` trait; the other six
  repositories hold 21 trait methods and no defaults at all. Associated types
  and constants may still default; blanket impls are not caught.
  [R014](ADRs/R014-ADR-PureTraitsRule.md)

### Trait surface

- `Rule` has no default bodies. Every rule answers all six questions in its own
  file -- `name`, `check`, `check_workspace`, `is_configured`, `requirement`
  and `explanation` -- including the answers that are "nothing", so that a
  rule's file says which question it is about instead of leaving it to be
  inferred from an absence. It began at four methods and 27 such bodies; each
  method added since has been added the same way, breaking every rule at once
  rather than compiling silently.
- `Rule::is_configured` is answered explicitly by every rule. Its old default of
  `true` meant a new rule was configured because nobody said otherwise, which is
  the silent pass this tool exists to catch.

## Version 0.4.0

### Rules

- **`directory-file-count`** -- a directory holds at most 20 `.rs` files, not
  counting its own index. `max-files-per-directory` in `stern4rust.toml` changes
  it; the only rule whose number is taste rather than fact. Ten offences across
  five of eight repositories. [R010](ADRs/R010-ADR-DirectoryFileCountRule.md)
- **`directory-subfolder-count`** -- at most 5 subfolders per directory, checked
  at every level. The counterweight, so splitting is not the answer to
  everything. Finds nothing today and says so.
  [R011](ADRs/R011-ADR-DirectorySubfolderCountRule.md)

### Autofix

- `--fix` repairs `test-file-structure` offences: item order within a section,
  section order, and blank lines. Everything else is reported untouched.
- Works from `syn` spans, moving whole line ranges without reading them, so a
  string literal holding Rust travels like any other line.
- Only touches files `test-file-structure` governs; never reorders imports,
  which is rustfmt's decision; never loses the preamble or a trailing comment.
- `fixed=N` in the summary and `files_fixed` in the JSON. The checks run against
  the repaired tree, so the report is the truth after fixing.

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
  `offence-threshold`, `rules`, `skip`, `exclude`, `baseline`,
  `max-files-per-directory` and `max-subfolders-per-directory`. Every key
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
- `[package.<name>]` sections — one `stern4rust.toml` at a workspace root
  configures every member and one call judges them all. Precedence runs command
  line, package, root, default, replacing rather than merging; `baseline` and
  `offence-threshold` are root-only; a section naming no scanned package is an
  error.
- Manifest data resolved per package — the licence and the dependency list come
  from the package being walked rather than once for the run, which is what lets
  `spdx-matches-manifest` apply to a workspace at all.
- A roster per package in the report, stated once where they agree so an
  ordinary run is unchanged, and a block each where they differ.

### Project

- 725 tests, one test file per source file, `autotests = false` with a single
  `[[test]] all_tests`.
- Two gates: stage 1 (fmt, clippy `-D warnings`, tests), stage 2 (`crap4rust`,
  `twin4rust`, `iceberg4rust`, and stern4rust against its own tree, built from
  source rather than from whatever is installed).
- Self-gating: a rule that would fail this repository cannot be merged into it.
- Thirty-three ADRs — twenty-one `R` rule ADRs, twelve unnumbered.
