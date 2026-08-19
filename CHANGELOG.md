# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **`--rule <NAME>` and `--skip <NAME>`**, both repeatable. Naming any rule with
  `--rule` makes the selection a whitelist; `--skip` subtracts from whatever is
  left. The default is unchanged: every rule, nothing excluded. Skipping wins
  over selecting, because between two readings of a contradictory instruction
  the one that checks less is the one that cannot quietly claim more.

  This exists for adoption. The survey against the six sibling tools found 717
  offences across 338 files and not one repository that could gate on this tool
  — `grip4rust` alone faces 233, which nobody switches on. `--rule header`
  narrows that same run to 6. A gate on one rule is a gate somebody turns on
  this afternoon.
- `rules_applied` and `rules_skipped` in the summary line and the JSON document,
  and a `note:` line naming the rules that were not applied. A run with rules
  switched off reports `All selected rules are satisfied` rather than
  `All rules are satisfied` — what was not looked at is part of the finding.
- `RuleRegistry::known_names`, built by asking each rule its own name so the
  list the switches validate against cannot drift from the rules themselves.

### Changed

- An unknown rule name is an error (exit `1`), not a switch that quietly matches
  nothing. `--skip test-file-strucutre` that silently skipped nothing would look
  exactly like a switch that worked. The error lists the valid names.
- `--rule header` without `--header-file` is an error (exit `1`). The registry's
  habit of leaving an unconfigurable rule out silently is right for an omission
  and wrong for a request: asking for a rule by name and getting an empty run is
  worse than not asking.

### Fixed

- Four ADR links in `README.md` still pointed at the pre-`R`-prefix filenames
  and were broken.

## [0.2.0]

The release that makes the tool do something. `0.1.0` published a scaffold with
no rules; this one has five, two output formats, and a report built to be acted
on rather than only read.

### Changed

- **An unreadable source file now exits `2` instead of `1`.** This is a change
  to a published interface, not an addition. The line between the two codes is
  whether the work can still be enumerated: a bad manifest leaves no list of
  files to judge and stays a `1`, while one unreadable file among fifty leaves
  forty-nine worth reporting on and is a finding like any other. A wrapper that
  treats `1` as "investigate the tooling" will now see such a file as an
  ordinary offence. See `docs/ADRs/ADR-ExitCodeContract.md`.
- A directory holding its own `Cargo.toml` is no longer walked. It is a
  different package, and cargo would not compile its files as part of this one
  either. Measured against `crap4rust`: 94 files scanned down to 67, and its
  fourteen fixture crates stopped being judged as though they were its own code.
- Offences are sorted by file, then line, then rule before rendering. Rules run
  in registration order with the tree-wide pass last, so the report used to jump
  between files.

### Added

- **`readable-source`** — every `.rs` file can be read and parsed, and failing
  either is an offence rather than a reason to say nothing. Registered first,
  because its failure explains every other rule's silence about the same file.
  Written after a file on disk became 41 NUL bytes mid-development and the tool
  reported one fewer offence with no indication anything had been skipped.
- **`header`** — every `.rs` file opens with the repository's header, supplied
  by `--header-file` because it is never the same twice. Exact after
  normalisation, so a wrong year fails while a BOM, CRLF line endings and a
  trailing newline do not. One offence per file, carrying the whole expected
  header so the fix is one pass rather than a loop.
- **`test-file-structure`** — header, imports, constants, helpers, tests; each
  group alphabetical; spacing part of the shape. `Helpers` is defined by
  exclusion, so a kind of item nobody has thought of yet lands where a reader
  would put it.
- **`tests-layout`** — exactly one `tests/all_tests.rs` and a `mod.rs` in every
  subfolder on the way down, both holding only the header and `pub mod`
  declarations. The failure it exists for is silent by construction: a test that
  is never compiled cannot fail.
- **`test-free-source`** — no `#[test]`, `#[cfg(test)]` or
  `#[cfg_attr(test, ...)]` outside `tests/`. The line is `test` rather than
  conditional compilation: `#[cfg(feature = "...")]` and
  `#[cfg_attr(feature = "serde", derive(Serialize))]` are ordinary library work
  and are left alone, because a feature is selectable by the shipped build while
  `test` is the one predicate no shipped build ever sets.
- **A required `correction` on every offence** — what to do, not only what is
  wrong. Required rather than optional and enforced by `Offence::new`'s
  signature, so a new rule cannot be added without answering it. In the table it
  renders on its own indented line beneath the offence.
- **`--format json`** — the same run as a document with a stable shape, for a
  gate script or an agent. Nothing can parse the table reliably: paths and
  descriptions both contain spaces, and descriptions carry backticks, quotes and
  semicolons.
- `subject` and `expected` on an offence — the thing it is about, named, and the
  correct text where the rule knows it. The header rule puts the entire header
  in `expected`.
- **`--offence-threshold <N>`** — how many offences the report prints, default
  `100`, `0` for all. The cap is on what is shown and never on what is counted:
  the summary reports the true total, the omitted count is stated outright with
  the flag that raises it, and the exit code is decided from every offence, so
  capping to 1 against 200 offences still exits `2`.
- `Rule::check_workspace` beside `Rule::check`, both defaulting to reporting
  nothing. Some offences are about a tree rather than a file, and the file that
  carries such an offence is usually the one that does not exist.
- `docs/` — `ARCHITECTURE.md`, `RULES.md`, `IMPLEMENTED-FEATURES.md`,
  `OPEN_POINTS.md`, `ROADMAP.md`, and nine ADRs split into `R<NNN>-ADR-` for
  rules and unnumbered `ADR-` for everything else.

### Fixed

- A registry holding several strays produced that many byte-identical rows, all
  pointing at line 1, naming none of them. Each stray is now reported at its own
  line and by its own name — "the constant `LIMIT`", "the import
  `use std::fmt;`".
- An unreadable file no longer aborts the whole run, discarding every offence
  already found in every other file.

### Known limitations

See `docs/OPEN_POINTS.md`. The sharpest: a fixture tree with no `Cargo.toml` is
still walked and cannot be excluded — 101 of `grip4rust`'s 233 offences are this
and nothing else. Rules cannot be selected or disabled, and there is no
baseline, so adopting the tool on a large existing codebase is currently
all-or-nothing.

## [0.1.0]

### Added

- Publishable crate skeleton. Packages as `cargo-stern4rust` with a `stern4rust`
  library, so cargo resolves `cargo stern4rust`, matching `cargo-crap4rust`,
  `cargo-twin4rust` and `cargo-iceberg4rust`.
- `Args::without_cargo_subcommand`, the argv fixup every cargo subcommand needs:
  cargo runs `cargo stern4rust ...` as `cargo-stern4rust stern4rust ...`, so the
  name arrives twice, while running the binary directly does not repeat it. The
  strip is conditional and positional, so a package that happens to be named
  `stern4rust` survives.
- `--manifest-path` and `--package`, the two flags the whole family shares.
- Test harness: `autotests = false` with a single `[[test]] all_tests`, one test
  file per source file.

### Not implemented in this version

- The rules themselves. A run reported that nothing was implemented and exited
  `0`.
