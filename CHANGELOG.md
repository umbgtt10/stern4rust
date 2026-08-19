# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **`stern4rust.toml`**, beside the manifest it configures, holding
  `header-file`, `offence-threshold`, `rules`, `skip` and `exclude`. Every
  switch had to be repeated at every invocation, which is tolerable for a person
  running the tool once and useless for a repository wanting the same run in a
  gate script, a pre-commit hook and a developer's terminal.

  **The command line wins, per setting**, so one override does not mean
  restating the rest. For the list settings that is replacement rather than
  merging: `--rule header` meaning "header plus whatever the file selected"
  would be the opposite of what naming one rule means everywhere else.

  An unknown key is an error rather than a silently ignored line -- a misspelled
  `exclude` doing nothing would look exactly like one that worked -- and so is a
  file that exists and cannot be parsed. A missing file is the ordinary case and
  is not an error. `header-file` resolves relative to the config file, so a
  checkout works in any directory.

  **The report names the config it used**, and the JSON carries `config_file`,
  because a run configured by a file the reader never typed would otherwise have
  invisible switches in force.

- **`registry-completeness`**, the ninth rule. A registry declares every module
  beside it -- each sibling `.rs` file and each subfolder that has a registry of
  its own -- so nothing in the tree goes uncompiled. This closes the half of the
  registry question `tests-layout` and `module-registry` both leave open: they
  check that a registry exists and holds only declarations, not that the
  declarations are complete.

  **Only one direction needed a rule, and that was measured rather than
  assumed.** `pub mod missing;` with no `missing.rs` is a compile error --
  rustc reports `E0583` immediately. An orphan `.rs` file that no registry
  declares produces no error and no warning at all. Silence is the whole
  failure, so silence is all the rule looks for; the other half of the work
  proposed in `OPEN_POINTS.md` turned out to be the compiler's.

  `pub` is not required, since a private `mod name;` compiles the file just as
  well. An inline `mod name { ... }` declares no file and does not count.
  `main.rs` counts as a registry beside `lib.rs`, so a file declared only from
  the entry point is not reported as an orphan. The offence lands on the
  registry rather than the orphan, because the orphan needs no edit.

  On its first run across eight repositories it found **8 offences in one**:
  `grip4rust` has eight `*_analysis_tests.rs` files that `tests/all_tests.rs`
  declares none of -- roughly thirty tests in a published tool that have never
  once executed. Verified independently of the tool before it was believed.

- **`--exclude <GLOB>`**, repeatable, matched against the package-relative path,
  for the tree a repository cannot move -- vendored source, generated output.
  Separators are normalised, so a pattern checked in on one platform works on
  every other.

  **It is not a silent skip.** Every pattern is named in the report with the
  number of files it removed, `files_excluded=N` joins the summary line, and the
  JSON carries an `exclusions` array. A pattern that matched **nothing** is
  called out by name -- that is the case a bare total would hide, since an
  exclusion naming a tree that has moved or been deleted goes on looking like it
  is doing work.

  Exclusion happens after the walk rather than by pruning it: a tree that is
  never entered cannot be counted, and the count is the point. An uncompilable
  pattern is an error rather than a switch that quietly matches nothing.

  This is deliberately the opposite of the nested-package skip removed in
  `0.4.0`, which hid 27 files with no line accounting for them. See
  `docs/ADRs/ADR-ExclusionsAreCounted.md`.

- **`imported-paths`**, the eighth rule, and the first that applies to test
  files as well as productive ones. A function is called through a name the file
  imported, not through a path: `syn::parse_file(...)` compiles with nothing in
  the file mentioning `syn`, so a reader scanning the imports to see what this
  file needs is given a wrong answer.

  One imported segment stays legal and is the shape of the rule rather than an
  exception to it -- `use std::fs;` followed by `fs::read_to_string(...)` names
  the route once and still says at the call site which module the function came
  from. Type qualifiers (`Widget::new()`, `Self::inner()`) are left alone, told
  apart from modules by case, since the tool has no type information. Macros are
  not checked.

  The correction names the import to add and what the call then reads as, and
  the two path shapes split differently: `syn::parse_file` becomes
  `use syn::parse_file;` and `parse_file(...)`, while `std::env::args` becomes
  `use std::env;` and `env::args(...)`, keeping the qualifier that says
  something.

  It found 15 offences in this tool's own source on registration, five of them
  `syn::parse_file` -- one inside the finder implementing the rule.
  `etheram-core`, where the standard was written down, is at 0.
- **`single-implemented-type`**, the seventh rule. A source file outside
  `tests/` holds at most one type that carries behaviour: at most one struct or
  enum both declared in the file and given at least one impl block there.
  Structs and enums without impl blocks are unlimited, because plain data is not
  a subject and a file's payload types belong beside the subject that uses them.

  Both halves of that conjunction do work. *Declared here* means an
  `impl Display for SomeoneElsesType` does not make this file that type's home.
  *At least one impl block* counts trait impls as well as inherent ones, since
  both are behaviour -- `#[derive(...)]` is not an impl block in the syntax tree
  and correctly does not count.

  Measured across eight repositories before it was written, exactly one file
  broke it: this tool's own `src/report_printer.rs`, holding `ReportPrinter` and
  `ColumnWidths`. `ColumnWidths` now lives in `column_widths.rs` with three
  tests it never had while it was a private struct in another type's file.
- **`module-registry`**, the sixth rule. A `lib.rs` or `mod.rs` outside `tests/`
  is an index: it holds the header, the crate's inner attributes,
  `extern crate alloc;` and `pub mod` declarations, and nothing else.

  Inner attributes need no exception, because `syn` keeps `#![no_std]` on the
  file rather than among its items -- so a no_std crate root passes without the
  rule maintaining a list of attribute names. `extern crate alloc;` is the one
  non-`mod` item allowed, since a no_std crate has to say it somewhere and the
  crate root is where it belongs. `pub` is required, because a private `mod`
  hides part of the crate's shape from the file whose job is to state it.

  The sharpest thing it catches is the re-export shim -- `pub use` in a registry
  -- which these standards forbid outright and which forms in exactly this file.
  Measured across seven repositories it finds 24 offences in two: 17 `pub use`
  lines in `slotgate`, and 5 imports plus two entry-point functions in
  `crap4rust`'s crate root.

  `tests/` is left to `tests-layout`, which asks a different question of the
  same filenames and gives a different answer about a private `mod`. That
  disagreement is why `RegistryPolicy` is a type rather than a boolean threaded
  through a call site.

### Changed

- **`--offence-threshold` is no longer defaulted by clap.** It had to become
  optional internally so that "not passed" is distinguishable from "passed the
  default" -- without that, `stern4rust.toml` could never set the threshold,
  since every run would look like the reader had asked for 100. The default is
  unchanged at 100 and is applied after the config file is merged.
- **The summary line gained `files_excluded=N`**, between `files_scanned` and
  `offences`, always present including as `files_excluded=0`. A gate script
  matching on the summary text will need updating.
- **The walker no longer skips a directory holding its own `Cargo.toml`.** That
  skip shipped in `0.2.0` and was silence: a run reported `files_scanned=67`
  where the tree held 94 `.rs` files, with no line accounting for the other 27.
  It also hid real files -- ~40 of the offences written off as `grip4rust`
  "fixture noise" were that repository's own integration tests, which merely
  lived under `tests/fixtures/`.

  Sample code a tool analyses is input, and input does not belong inside the
  package that ships. The fix is layout, not a skip; where a tree genuinely
  cannot move, an explicit exclusion the reader can see in the report is the
  answer. See `docs/ADRs/ADR-WalkEveryFileInThePackage.md`.

  **This raises counts for any repository keeping analysis input inside its
  published package.** `crap4rust` goes from 130 offences to 161.

### Fixed

- **`RuleRegistry` kept the rule set in two hand-maintained lists.** `from_config`
  built one and `known_names` built another, so a rule added to only one of them
  was applied by a default run while `--rule <name>` rejected it as unknown --
  which is exactly what `imported-paths` did on registration. There is now a
  single `all()` list that `from_config` narrows and `known_names` reads.

  The registry no longer names any rule in particular either: whether a rule has
  what it needs is asked through a new `Rule::is_configured`, defaulting to true
  and answered false by `HeaderRule` without a header. The `if` that knew about
  the header rule was how the second list started.
- **`ImplementedTypeFinder::walk` took two `&mut` accumulators**, against the
  house standard preferring return values. The two halves are now gathered by
  `declared()` and `implemented()`, each answering one question and returning
  it, which makes the recursion an expression rather than a side effect.

- **`test-file-structure` could demand an import order `cargo fmt` refuses to
  write.** The stand-down for orders that rustfmt rather than the alphabet
  decides was keyed on an import's *first* segment, so it missed a pair that
  shares its first segment and diverges later: `use serde_json::Value;` beside
  `use serde_json::from_str;` left a file that no edit could make green, because
  stage 1 runs the formatter first and it undid every fix. The decision is now
  made per pair, standing down where two paths first differ and the segments
  there are of different case.

  Measuring rustfmt to fix this turned up behaviour worth recording: it treats
  case as significant in *opposite* directions at the two levels. An
  uppercase-initial crate sorts behind every lowercase one (`Bbb::gamma` after
  `zzz::last`), while an uppercase-initial segment later in a path sorts ahead of
  its lowercase siblings (`serde_json::Value` before `serde_json::from_str`).
  `cargo fmt` and a standalone `rustfmt <file>` also disagree here; only
  `cargo fmt` matters, since that is what the gate runs.


- **A shared helper inside the tests tree made a file unsatisfiable.** Everything
  under `tests/` is one crate rooted at `all_tests.rs`, so a sibling reaches a
  helper through `use crate::support::...`. rustfmt sorts `self`, `super` and
  `crate` ahead of every other path -- and an uppercase-initial path behind them
  all -- so demanding the alphabet there put `cargo fmt` and
  `test-file-structure` in a loop neither could win, with stage 1 running the
  formatter first. The alphabetic check now stands down on any import pair
  involving such a path and still orders everything else.

### Documentation

- README documents all eight rules. `module-registry` and `single-implemented-type`
  had been added without a section, so the rule list stopped at five while the
  tool applied seven; the adoption example's offence counts were stale by two
  rules and are now measured against `braintax4rust`.

## [0.3.0]

Adoption, and the report finally saying what it did.

### Fixed

- **A run that did not apply every rule said it had.** Without `--header-file`
  the registry drops the header rule, and the report printed
  `All rules are satisfied` with `rules_skipped=0` — four rules of five applied,
  the fifth never named anywhere. That is the exact comfortable lie this tool
  exists to catch, told by this tool, and `README.md` compounded it by claiming
  the report said which rules ran. It did not. The bug predates the switches
  below; adding them is what made it visible.
- Four ADR links in `README.md` still pointed at the pre-`R`-prefix filenames
  and were broken. A link check now runs over every markdown file.

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
- **Every report now names the rules it applied**, and names the ones it did not
  along with why. Three states rather than two: *applied*, *skipped* (a choice
  you made), and *unconfigured* (a flag you did not pass). Calling the third one
  skipped would blame you for a decision you never took; calling it nothing at
  all is what produced the bug above.

  ```text
  All applied rules are satisfied.

    applied: readable-source, test-file-structure, test-free-source, tests-layout
    not applied: header (needs --header-file)
  ```
- `rules_applied`, `rules_skipped` and `rules_unconfigured` in both the summary
  line and the JSON document.
- `RuleRegistry::known_names`, built by asking each rule its own name so the
  list the switches validate against cannot drift from the rules themselves.

### Changed

- **The text report's shape changed**, which matters if you parse it. The clean
  verdict is `All applied rules are satisfied` rather than
  `All rules are satisfied` whenever some rule did not run; an `applied:` line
  and, when relevant, a `not applied:` line precede the summary; and the summary
  gained `rules_applied=`, `rules_skipped=` and `rules_unconfigured=`. The
  existing summary prefix is unchanged, so a script matching
  `files_scanned=… offences=… rules_broken=…` still matches. The JSON gained
  keys only.
- An unknown rule name is an error (exit `1`), not a switch that quietly matches
  nothing. `--skip test-file-strucutre` that silently skipped nothing would look
  exactly like a switch that worked. The error lists the valid names.
- `--rule header` without `--header-file` is an error (exit `1`). The registry's
  habit of leaving an unconfigurable rule out silently is right for an omission
  and wrong for a request: asking for a rule by name and getting an empty run is
  worse than not asking.

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
