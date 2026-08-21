# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.9.4] - 2026-08-21

### Fixed

- **`--package` no longer errors in a repository whose config has sections.**
  `0.9.3` checked each `[package.<name>]` section against the packages the run
  was *walking*, so scoping a run to one member made every section for the
  others look like a typo — and scoping a run to one member is an ordinary thing
  to do. Sections are checked against the workspace now. Only a name no member
  carries is an error.

## [0.9.3] - 2026-08-21

### Added

- **A `[package.<name>]` section per member, in one `stern4rust.toml` at the
  workspace root.** A workspace whose members want different rules says so in
  one file and is judged in one call, rather than a config beside every manifest
  and a call each.

  Precedence runs command line, package, root, default, and each level
  *replaces* the one below rather than adding to it -- the argument already made
  for the command line against the file, one level down.

  `baseline` and `offence-threshold` are not keys a section has: a baseline is a
  set of fingerprints for one run and the threshold is about how the report
  ends, so neither is a property of a package. The per-package struct simply
  lacks them, so `deny_unknown_fields` rejects them with the message it already
  gives a misspelled key.

  A section naming a package the run does not scan is an error, for the reason a
  misspelled `--rule` is: it reads as a rule set being applied, and a package
  quietly running every rule when its section said to skip one is the silence
  this tool refuses. `deny_unknown_fields` cannot catch it, because the section
  name is data rather than a key.

  Measured on `etheram-raft`: four files and four calls became one file and one
  call, over 250 files, and the trap they carried went with them --
  `--manifest-path node/Cargo.toml` without `--package` measured 243 files and
  read as though it had measured 127.

  See [ADR-PerPackageConfiguration](docs/ADRs/ADR-PerPackageConfiguration.md).

### Fixed

- **`spdx-matches-manifest` can now apply to a workspace.** It never has.
  `ManifestResolver::license` compared a set of distinct licence strings against
  a count of packages, and those are only ever the same length when there is one
  package -- so four members all declaring `Apache-2.0` read as none declaring
  it, and the rule stood down saying the licence was missing.

  The licence is now resolved per package inside the scan loop, along with the
  rule registry and the exclusion set. `package_roots` and `license` are
  replaced by `packages()`, which returns each package with its own.

  **This is a behaviour change.** Any repository scanning more than one package
  at once will see the rule move from *not applied* to *applied*, and offences
  may appear. They are real: the rule was never able to speak there.

  See [ADR-ManifestDataIsPerPackage](docs/ADRs/ADR-ManifestDataIsPerPackage.md).

### Changed

- **The report names which rules ran against which package.** A workspace whose
  members answer to different rule sets gets a block each; one whose members
  agree gets one roster, said once and without a package name, which is exactly
  what every run prints today. An ordinary single-package run is unchanged.

  The `summary:` line keeps its shape and keeps answering for the run as a
  whole, so it understates: `rules_skipped=1` where one package of four stood a
  rule down is true and coarse. The blocks above it carry the detail, and every
  gate script in this family goes on parsing the line below.

## [0.9.2] - 2026-08-21

### Fixed

- **`ordered-imports` and `test-file-structure` no longer demand an order
  rustfmt will not write, for a path that extends another.**

  `use alloc::vec;` beside `use alloc::vec::Vec;` was reported as out of order,
  and could not be fixed: rustfmt writes the shorter path first, the rule
  demanded the longer, and stage 1 runs the formatter. Each run undid the last.

  The stand-down that exists for exactly this reason was not firing. It looks
  for the first pair of segments that differ, and here there is no such pair --
  the difference is between a segment and nothing at all, which `zip` cannot
  see. Compared as written the shorter line ends in `;` (59) where the longer
  carries on with `::` (58), so a plain sort demands the longer path first
  whatever follows. Every extension disagreed, not only the uppercase ones.

  Measured against rustfmt at both style editions before choosing the fix,
  because they disagree with each other about the rest of this: 2021 sorts an
  uppercase-initial crate last and `from_str` ahead of `Value`, 2024 does the
  opposite of both. On an extended path they agree -- shorter first -- for
  uppercase, lowercase, brace groups, globs and groups broken across lines.

  A rename is not an extension. `use aaa::bbb as ccc;` is one segment rather
  than two, rustfmt puts it ahead of `use aaa::bbb;` at both editions, and the
  existing comparison already agrees -- so that pair still answers to the
  alphabet. Standing down there would have lost a check that works.

  Found by `etheram-raft`, where it accounted for ten of twenty-nine remaining
  offences and no edit could clear them.

### Changed

- **`ImportPath`'s note on rustfmt's case handling was wrong for every
  edition.** It described 2021's rule for an uppercase crate and 2024's for an
  uppercase segment, so it matched neither as a whole. It now states both and
  gives the real argument for standing down: the editions disagree, this crate
  cannot know which one the code under inspection compiles with, and declining
  to judge is the only answer correct under either.

## [0.9.1] - 2026-08-20

### Changed

- **`tested-public-api` no longer counts the methods a `pub trait` declares.**
  Two shapes remain: a free `pub fn`, and a `pub fn` in an inherent impl.

  The rule already excused a method *implementing* a trait, because it carries no
  visibility of its own and is reached through the trait rather than named.
  Counting *declarations* while excusing every implementation of them was
  incoherent -- a trait method can only be called through an implementor.

  `etheram-core` measured the cost. It is a trait-definition crate, and the rule
  reported **15 offences, every one a trait method**. Satisfying them meant eight
  fake implementors whose only purpose was to be asserted against: a test of the
  compiler, not of the code. Across the family the rule goes from 72 offences to
  34, and every one of the 38 dropped is a declaration.

## [0.9.0] - 2026-08-20

### Changed

- **Three breaking changes, each a consequence of a rule this tool enforces on
  itself.**

  `Rule` gained `requirement`, so an external implementor must answer it -- the
  trait has no default bodies, by `pure-traits`.

  `src/finding/` module paths moved into `model/` and `parsing/`, as
  `src/rules/` did in `0.8.0` and for the same reason: `directory-file-count`
  refused a twenty-first file in one directory. Under
  [ADR-LibrarySurfaceIsNotAnApi](docs/ADRs/ADR-LibrarySurfaceIsNotAnApi.md)
  these are not breaking at all, since the library is not a public API -- but
  the note stays, because a reader whose build stopped compiling deserves to
  know why.

  **Baseline fingerprints changed, which invalidates every existing baseline
  once.** A checked-in baseline reports its entries as stale on the first run
  after upgrading; `--write-baseline` rewrites it, and the report already says
  so.

### Changed

- **A rule that could not run now says what it was waiting for.**
  `Rule::requirement` returns what a rule needs, and the report prints it:
  `header (needs --header-file)`,
  `spdx-matches-manifest (needs a `license` field in Cargo.toml)`.

  This is a breaking change for anyone implementing `Rule` outside this crate --
  the trait has no default bodies, so every implementor answers. Nineteen of the
  twenty-one answer `None`.

  The text was hardcoded in the printer until 0.8.0, when a second rule could go
  unconfigured and it began telling readers to pass `--header-file` for a rule
  waiting on a manifest field. Replacing it with a bare `(not configured)` fixed
  the inaccuracy and **lost the correction**: it said what was wrong without
  saying what to do, which Guiding Principle 1 requires of every finding. This
  restores it from the rule rather than from a hardcoded string.

### Changed

- **`src/finding/` is grouped into `model/` and `parsing/`**, with
  `tests/finding/` mirroring. Breaking for anyone importing a finder from the
  library, in the same way `0.8.0` was for the rules. The directory stood at 20
  files and `directory-file-count` would not allow a twenty-first, so the tool's
  own rule decided the split again.

- **`ItemNaming` replaces three copies of the same code.**
  `TestFileParser::name`, `RegistryParser::label` and `UnitTestFinder::describe`
  each carried the same match over `syn::Item` and an identical `source_line`
  fallback -- `OPEN_POINTS.md` recorded it as "three copies drift" and asked for
  the extraction to be deliberate rather than incidental to a bug fix.

  Only the shared half moved. The kind words stay with each caller, because one
  says `inline module` where another says `module`, and every one of those
  strings sits inside an offence description that baselines are keyed on.
  Sharing them would have been a silent breaking change for every baseline in
  the family.

### Added

- **`ADR-LibrarySurfaceIsNotAnApi`** — the library is **not** a public API.
  Depend on the binary. Module paths, type names and signatures may move in any
  release, including a patch.

  This was Phase 6 of the roadmap from the first release, and two restructures
  made it urgent: `0.8.0` moved every rule's module path and the release after
  moved every finder's, both because `directory-file-count` refused a
  twenty-first file in one directory. The tool's own rule reshaped its own
  library surface twice while no policy said whether that was allowed.

  The decision follows from a standard already in force: `CLAUDE.md` forbids
  unit tests, so every test lives in `tests/` and can only reach what is `pub`.
  **The surface is an artefact of the testing standard, not an offer.** It is
  exactly as wide as the test suite needs.

  Stated in `src/lib.rs` so it appears at the top of the `docs.rs` page, and in
  `README.md` beside the install instructions. Nothing about the behaviour of
  the tool changes.

### Fixed

- **A baseline no longer forgives by description.** `OffenceFingerprint` keys on
  file + rule + **subject**, not file + rule + description.

  The description was standing in for the subject, from before `Offence` carried
  one. That made **rule descriptions a published interface with nothing saying
  so**: a rule whose sentence gained a word reported its offences as new across
  every repository that had baselined them, all at once, with a spike in the
  stale-entry count as the only signal. The subject survives a rewrite of the
  sentence around it.

  Only `header` emits an offence with no subject, and it reports once per file,
  so the file and the rule already tell those apart; the description remains the
  fallback there.

  **This invalidates existing baselines, once.** Every fingerprint changes, so a
  checked-in baseline will report its entries as stale on the first run after
  upgrading. `--write-baseline` rewrites it, and the report already says so. It
  is the failure the old key made inevitable, triggered deliberately and once so
  that it stops recurring.

- **`tests-layout` no longer goes silent on a registry it cannot parse.** It
  reported nothing at all, which is indistinguishable from a registry it had read
  and found clean -- the silence Guiding Principle 2 refuses. It now states that
  the registry was not checked, once, with the syntax error as the correction.
  One offence rather than one per sibling file, since treating an unparseable
  registry as declaring nothing is the page of wrong answers R009 rejected.

## [0.8.0] - 2026-08-20

### Changed

- **Breaking, for anyone importing a rule from the library.** `src/rules/` is
  grouped into `source/`, `layout/`, `testing/` and `manifest/`, so every rule's
  module path moved: `stern4rust::rules::header_rule::HeaderRule` is now
  `stern4rust::rules::source::header_rule::HeaderRule`. Nothing was renamed or
  removed and no behaviour changed — only the path. The cargo subcommand is
  unaffected.

  The move was forced rather than chosen: `src/rules/` and `tests/rules/` both
  stood at exactly 20 files, and `directory-file-count` would not allow a
  twenty-first rule. The tool's own rule decided the layout.

  **Whether these paths were ever promised is exactly what has never been
  decided** — Phase 6 of [ROADMAP.md](docs/ROADMAP.md) is about committing to a
  library surface, and until it lands the answer is that they are not.

- **`Config` gained `manifest_license` and `workspace_dependencies`**, which is
  breaking for anyone constructing it without `..Config::default()`. Both are
  filled by `Runner` from the manifest; the command line has nothing to say
  about either.

- **Three new rules join the default set**, so a repository green on `0.7.0` may
  not be on `0.8.0`. Measured across the family they add: `workspace-dependencies`
  27, `ordered-imports` 10, `spdx-matches-manifest` 1 offence and one
  unconfigured package. `--skip <name>` turns any of them off.

### Added

- **`workspace-dependencies`**, the twenty-first rule: a workspace declares its
  dependencies once, in the root, and every member takes them from there with
  `.workspace` notation.
  [R021](docs/ADRs/R021-ADR-WorkspaceDependenciesRule.md)

  **Three requirements, one check.** The root holding all the references, each
  member using `.workspace`, and members declaring nothing new are the same
  requirement: `cargo` refuses to build a `foo = { workspace = true }` the root
  does not declare, so only the middle one needs code. That is the fourth time
  this repository has found the shape -- R009 (`E0583`), R014 (`E0046`), R016,
  and now this.

  Read from the TOML rather than from `cargo metadata`, because the question is
  *how* a dependency was written and resolution erases exactly that. All three
  tables count; a member pinning its own `proptest` splits the workspace as
  surely as a runtime dependency does.

  Intra-workspace path dependencies are included, decided on evidence:
  `etheram-ibft`'s root already centralises `execution-types` and `evm` as path
  dependencies while `node = { path = "../node" }` is spelled out in four
  members. Exempting them would have hidden all six of its findings.

  **27 offences across three workspaces.** `crap4rust` (11) and `grip4rust` (10)
  have no `[workspace.dependencies]` section at all; `etheram-ibft` (6) has one
  and leaks only on path dependencies. `braintax4rust` centralises all 65 of its
  dependencies and `etheram-raft` all 27, so the convention is demonstrably
  workable at that size. The five repositories that are not workspaces are
  silent -- not `(not configured)`, since the rule has everything it needs and
  simply has no subject.

### Changed

- **`src/rules/` is grouped into subfolders** -- `source/`, `layout/`,
  `testing/` and `manifest/` -- with `tests/rules/` mirroring exactly. Both
  directories stood at exactly 20 files, so `directory-file-count` would not
  allow a twenty-first rule. The tool's own rule decided the layout, which is
  self-gating working as intended.

- **Offences are deduplicated by content.** The workspace question is asked once
  per package root, so a rule whose subject is the *workspace* rather than the
  package stated each finding once per member: `etheram-ibft` reported 36 where
  there are 6. `Vec::dedup` was not enough -- two findings about one manifest
  interleave once sorted, so consecutive-only removal missed every copy after
  the first pair.

- **`ordered-imports`**, the nineteenth rule: imports in `src/` run in
  alphabetic order, on the pairs where the alphabet is the authority.
  [R019](docs/ADRs/R019-ADR-OrderedImportsRule.md)

  `test-file-structure` has asked this of `tests/` since 0.2.0 and nothing asked
  it of the source tree -- which is where `imported-paths` routinely *adds*
  lines, with nothing saying where a new one lands.

  **Verified consistent with `cargo fmt` before it was written**, since that is
  the only way it could be wrong in a way no user could fix. A controlled
  experiment on default settings showed rustfmt is not alphabetical: it hoists
  `crate::` ahead of everything, then sorts the rest. A naive rule would demand
  `aaa_crate` above `crate::` and `cargo fmt` would put it back -- the
  unsatisfiable file this repository hit once before. The rule reuses
  `ImportPath`, the same seam `test-file-structure` uses, rather than deciding
  again.

  The 10 findings were confirmed to survive formatting three ways:
  `cargo fmt --check` clean, standalone `rustfmt --check` clean, and the block
  extracted into a scratch crate came back byte-identical.

  **10 offences, all in `etheram-ibft`** -- 5 `node`, 4 `node-infra`, 1 `evm`.
  ROADMAP predicted "a wave of offences"; it is not one, and that entry was
  corrected. Recorded as the cost: **56% of adjacent import pairs in `src/` stand
  down**, because a source file leads with a `crate::` block where a test file
  never does. More than half of what the rule appears to check, it does not.

- **`spdx-matches-manifest`**, the twentieth: every file's
  `SPDX-License-Identifier` says what the manifest's `license` says.
  [R020](docs/ADRs/R020-ADR-SpdxMatchesManifestRule.md)

  `header` compares a header against a text file and nothing else and says so in
  its own "does not catch". A package states its licence twice -- manifest and
  every header -- and nothing held the two together.

  The expected value comes from **the package being judged** rather than a flag,
  which is the difference that matters: it needs no `--header-file`. That is how
  it found `braintax4rust/core/src/traits/mod.rs`, a three-line registry with no
  header at all -- `header` cannot catch it there, because `braintax4rust` has no
  header file to configure `header` with.

  A manifest naming no `license` leaves the rule **unconfigured** rather than
  offended, and the report names it. That was measured: reporting the silent
  manifest as an offence fired once per package root, and `braintax4rust` -- a
  workspace of twenty packages -- produced twenty identical `Cargo.toml` lines.
  `is_configured` is the mechanism this codebase already had.

  `etheram-core` is the other shape: no `license` field, no SPDX anywhere, and
  25 files asserting Apache-2.0 in prose that nothing machine-readable confirms.

  This is the first rule configured from the manifest, so `Config` gained
  `manifest_license`, `Runner` fills it once before building the registry, and
  `ManifestResolver` reads it.

### Changed

- **An unconfigured rule now reads `(not configured)`** rather than
  `(needs --header-file)`. Two rules can go unconfigured and the header rule is
  not what the other is waiting on; the old text was a hardcoded reason the
  registry's own doc warns against.

- **The rule set is complete at twenty**, the upper end of the range `CLAUDE.md`
  set. Phase 5 of the roadmap is closed, and a twenty-first rule is a decision to
  widen that range rather than to fill it.

## [0.7.0] - 2026-08-20

### Changed

- **Four new rules join the default set**, so a repository that was green on
  `0.6.0` may not be on `0.7.0`. Nothing was removed and no signature changed —
  the library surface is additive — but a linter gaining rules is a behaviour
  change for anyone gating on it, and it is called out here rather than left to
  be discovered. `--skip <name>` turns any of them off; a skipped rule is named
  as skipped in the report.

  Measured across the family, the four together add: `arrange-act-assert` 45,
  `paired-test-file` 34, and zero from `test-file-name-postfix` and
  `declared-by-name`.

### Added

- **`declared-by-name`**, the eighteenth rule: a module is declared by name, and
  `#[path = "..."]` on a `mod` is an offence.
  [R018](docs/ADRs/R018-ADR-DeclaredByNameRule.md)

  Not a rule about taste. `#[path]` is the one attribute that makes another rule
  here give a **confident wrong answer**: `registry-completeness` resolves a
  declaration to the file it names by convention, so a file reached through an
  explicit path is reported as **never compiled** when it compiles perfectly
  well. R009 accepted that gap on the grounds that the house standard forbids
  `#[path]` -- a convention `CLAUDE.md`, `OPEN_POINTS.md` and `RULES.md` all
  described and **no rule enforced**.

  Package-wide rather than registries-only. The standard names `all_tests.rs`
  because that is where the temptation is, but a `mod` in an ordinary source
  file is resolved by the same convention and misread the same way.

  `#[cfg_attr(unix, path = "...")]` is deliberately left alone: a platform-gated
  module is the one honest use, it cannot resolve by name on every platform
  anyway, and reporting it would accuse correct code.

  **Zero offences across ten repositories**, which confirms R009's assumption by
  measurement rather than assertion, and is the whole point -- the day it stops
  being true, the failure does not look like a `#[path]` attribute. It looks
  like `registry-completeness` accusing an innocent file, somewhere else
  entirely.

  Written as its own rule rather than folded into `module-registry` or
  `tests-layout`: those are scoped to one tree each, so the check would have
  been written twice, and their offence sentence -- "does not belong in a module
  registry, which holds ... `pub mod` declarations" -- would contradict itself,
  since `#[path = "x.rs"] pub mod x;` **is** a `pub mod` declaration.

- **`arrange-act-assert`**, the seventeenth rule: a test reads `Arrange`, then
  one or more `Act`/`Assert` pairs, with a blank line separating the sections.
  [R017](docs/ADRs/R017-ADR-ArrangeActAssertRule.md)

  **The original motivating example for this whole tool, shipped seventeenth.**
  `ROADMAP.md` carried it as "the oldest unbuilt one" since the first release.

  Every marker expands into the phases it names, and the expansion must read
  `Arrange` then one or more `Act`, `Assert` pairs. The merged forms expand
  identically to the separate ones, so a single check covers every legal shape
  and still rejects an Act with no Assert, an Assert with no Act, a test with no
  markers, and an Arrange dropped rather than merged.

  **`// Arrange & Act & Assert` is now part of the standard.** It was not in
  `CLAUDE.md`, and practice had outrun the standard: nine tests use it across
  the family, including this crate's own `tests/finding/section_tests.rs`. A
  rule written strictly to the standard as it stood would have failed its own
  repository on the first run.

  **The hard part was never the grammar.** The markers are comments, and `syn`
  discards comments, so the rule reads lines -- and a line scanner cannot tell
  code from a string containing code. This repository's tests are built from
  Rust source embedded in raw strings: a naive scanner reports **seven offences
  here that are every one of them a string literal**, plus ~156 across the
  family where a `}` at column zero inside a raw string ended a test early. The
  lines every literal occupies are taken from the token stream and skipped --
  comments are not tokens and literals are. Walking tokens rather than the
  syntax tree also reaches inside macros.

  Markers may carry trailing prose after `--`, `:` or `.`, all three observed in
  the family; a marker ends on a word boundary so `// Actually` is prose; and
  comment lines above a marker are folded into it, the same call
  `TestFileParser` already made.

  Measured across ~3,900 tests in nine repositories before the grammar was
  chosen: 68% canonical, 19% `Arrange & Act`, 6.6% `Act & Assert`, 9 fully
  merged, and **2 with multiple Act/Assert pairs**. Those two are why a stray or
  duplicated marker cannot be caught -- the permissiveness is recorded as the
  cost it is.

  **45 offences across the family** -- 23 `grip4rust`, 9 `braintax4rust`, 5
  `iceberg4rust`, 4 `crap4rust`, 3 `twin4rust`, 1 `etheram-core`, and **0 in
  `stern4rust`** over roughly 500 tests including the fixtures it had to learn
  to ignore.

- **`paired-test-file`**, the sixteenth rule: a `tests/a/b_tests.rs` names the
  source file it exercises, and `src/a/b.rs` exists.
  [R016](docs/ADRs/R016-ADR-PairedTestFileRule.md)

  The other side of the pairing from `twin4rust`, which starts at a source file
  and looks for its test. Nothing asked the reverse, and R015 recorded the gap
  in its own "does not catch". A test file outlives the module it was named for
  **silently**: it still compiles, still runs, still passes, and its name now
  points at nothing.

  Matched by path rather than by name alone, so a test file in the wrong
  directory is as unpaired as one whose source is gone.

  **Four findings in `etheram-ibft/node`, predicted by hand before the rule
  existed and reproduced exactly** -- 40 tests across four files named for
  source files that exist nowhere in the crate, all passing, in a crate whose
  gates are green. Across the family: 19 in `braintax4rust` (18 of them fixture
  crates), 8 in `grip4rust`, 2 in `crap4rust`, 1 in `slotgate`, and zero in
  `stern4rust`, `twin4rust`, `iceberg4rust` and `etheram-core`.

  `grip4rust`'s eight are **the same eight files `registry-completeness` found
  never compiled**. Two rules, approaching from unrelated directions, landing on
  the same files.

  `_proptest_tests.rs` is exempt, and that was measured: before the exemption
  the rule found 7 unpaired files in `node`, 3 of which were property-test
  suites whose real counterparts exist. Excluding them removed exactly those 3.
  `all_tests.rs` is exempt as a registry that would otherwise resolve to
  `src/all.rs`.

  The correction says "rename it after the source file it exercises, or delete
  it if that file is gone" -- deliberately **not** "create the missing file",
  since every unpaired file measured tested something real under a drifted name.

  **It assumes the package is mirrored**, and says so. A harness crate --
  `src/` apparatus, `tests/` named after behaviours -- is not:
  `etheram-ibft/validation` reports 53 and `system-tests` 28, correctly and
  uselessly. `--skip paired-test-file` is the answer, using rule selection as
  shipped; no new configuration was added.

- **`test-file-name-postfix`**, the fifteenth rule: a file under `tests/`
  holding at least one test is named `<X>_tests.rs`.
  [R015](docs/ADRs/R015-ADR-TestFileNamePostfixRule.md)

  The mirrored pairing -- `src/foo.rs` answering to `tests/foo_tests.rs` -- is
  the most load-bearing convention in the family, and it was enforced from one
  side only. `twin4rust` starts at a source file and looks for its test, so a
  `tests/rules/widget.rs` holding twenty tests was invisible to everything:
  `tests-layout` cared only that a registry existed, `registry-completeness`
  only that the file was declared, `test-file-structure` only about the order
  inside it, `test-naming` only about the function names. All of them passed it,
  while `twin4rust` separately reported the source file as untested.

  One direction only. Holding a test obliges the name; a `_tests.rs` file
  holding none is a different failure with a different correction, and is left
  out on purpose -- the same split R009 and R014 made.

  Two exemptions, both load-bearing rather than softening. `src/` is exempt
  because a `#[test]` there is already `test-free-source`'s offence and this
  rule's correction would be **wrong**: renaming `src/foo.rs` to
  `src/foo_tests.rs` leaves the test where it does not belong. Registries are
  exempt because a `#[test]` in a `mod.rs` is already `tests-layout`'s, and
  `mod.rs` cannot be renamed at all.

  **Zero offences across the family**, and the ADR says so plainly rather than
  burying it. The convention is already kept everywhere. The one real instance
  found anywhere is `validation_support.rs` in a `crap4rust` fixture tree -- a
  support file that grew a test, which is exactly how this drift happens -- and
  it sits outside the published package, so no ordinary run walks it.

  Does not catch an **orphan**: `banana_tests.rs` with no `banana.rs` behind it
  passes, since `<X>` is never resolved against the source tree. That gap stays
  open on `twin4rust`'s blind side.

- **`R012-ADR-TestNamingRule` and `R013-ADR-TestedPublicApiRule`**, written
  after the fact. Both rules shipped in `0.6.0` with only changelog prose behind
  them, against the standing obligation that every rule carries an ADR saying
  what it does **not** catch.

  R012 records the retreat as the decision: three attempts to verify a test's
  leading name part was the method under test, each abandoned on measurement,
  and why counting underscores is the only version that is never wrong.

  R013 records that the rule under-reports on purpose, and that gathering call
  sites from macro token streams is load-bearing rather than a refinement --
  assertions live in `assert!`, which never becomes syntax, so skipping them
  would report the best-tested code in a codebase as untested.

  `docs/RULES.md` gains the two matching sections, and the ADR index gains rows
  for R008 through R013, which had been missing since those rules shipped.

## [0.6.0] - 2026-08-19

### Added

- **`test-naming`**, the twelfth rule: a test is named
  `<method>_<conditions>_<result>`. The name and nothing else.

  It reads a name and counts underscores, and that retreat is the point. Three
  earlier versions tried to verify the leading part was the method actually
  under test -- looking in the body, then through the test file's helpers
  transitively, then against the mirrored source file. All three were measured
  across 1559 tests in eight repositories and all three accused correct code:
  tests of derived operators (`a < b` calls no named function), tests of derived
  methods (`from_str` on a `#[derive(ValueEnum)]` enum is a `fn` nowhere), and
  names reachable only through whatever a wide setup helper happened to touch.
  592 offences became 5.

- **`tested-public-api`**, the thirteenth: every public entry point is called by
  at least one test. The question `test-naming` gave up on, asked from the other
  end -- starting from declared entry points needs no guess about intent, and
  sidesteps derives entirely, since `Default::default` is not a `pub fn`
  declaration and can never be reported.

  Counted: a free `pub fn`, a `pub fn` in an inherent impl, and every method of
  a `pub trait`. Matched on **name and arity**; types and parameter order are
  not checked and cannot be without type inference, so the rule under-reports
  rather than accusing tested code.

  84 offences across the family. In this crate it found the six printer builders
  shipped untested in 0.4.0 -- `with_fixed`, `with_baseline`, `with_config_file`
  on both printers -- and `signature`, a method written minutes earlier in the
  rule's own supporting type.

- **`pure-traits`**, the fourteenth: a trait declares, it does not implement. No
  method in a `trait` declaration in `src/` may have a default body.
  [R014](docs/ADRs/R014-ADR-PureTraitsRule.md)

  Only one direction needed code. That every implementor implements every method
  is `rustc`'s `E0046` and needs no rule -- the same split
  [R009](docs/ADRs/R009-ADR-RegistryCompletenessRule.md) made, where the
  missing-file direction was likewise already a compile error.

  Four offences across the family, and three of them were here. `Rule` carried
  defaults on `check`, `check_workspace` and `is_configured`, so no rule's file
  said which question it answered -- an absent `check` meant "this rule is about
  the tree" or "this rule is half-written", indistinguishably. `is_configured`
  defaulted to `true`, which made every rule in this tool configured because
  nobody said otherwise: the silent pass this tool exists to catch, in the tool
  itself. The fourth is `Collection::is_empty` in `etheram-core`.

  The other six repositories hold 21 trait methods between them and not one
  default body. The standard was already being kept everywhere except in the
  tool that was going to enforce it.

  Removing the three defaults cost **27 bodies**: an empty `check_workspace` for
  the nine per-file rules, an empty `check` for the five whose subject is the
  tree, and `is_configured` returning `true` for the thirteen needing no
  configuration.

  Associated types and associated constants may still carry defaults -- neither
  is behaviour. Blanket impls are not caught, and are recorded as the rule's
  sharpest gap.

### Changed

- **Breaking, for anyone implementing `Rule` outside this crate.** `check`,
  `check_workspace` and `is_configured` no longer have default bodies, so an
  existing implementor stops compiling with `E0046` until it supplies all four
  methods. The fix is mechanical -- `Vec::new()` for whichever of the two check
  methods that rule does not answer, `true` for `is_configured` unless the rule
  needs configuration -- and the compiler names every one it is missing.

  This is the rule enforcing itself, and it is the shape of change the tool is
  meant to force: the trait now states what an implementor must decide instead
  of deciding it for them. Called out here rather than left to be discovered,
  the same way 0.2.0 called out an unreadable file moving from exit `1` to exit
  `2`. The library surface is still uncommitted (Phase 6 in
  [ROADMAP.md](docs/ROADMAP.md)); this is a pre-1.0 minor bump.

### Fixed

- **The summary line never carried `baselined=` or `fixed=`.** Both were
  documented in 0.4.0 and neither was implemented: the edits adding them failed
  silently against a `\` line continuation in the format string. Both are
  present now, after `offences`.

### Removed

- **`Args::parse_args`.** A thin wrapper over `parse_from(without_cargo_subcommand(env::args()))`
  that could not be tested without controlling the process argv, while both
  halves are public and tested. `main.rs` calls them directly.

## [0.5.0] - 2026-08-19

### Added

- **`directory-file-count`**, the tenth rule. A directory holds at most 20 `.rs`
  files, not counting its own index. `max-files-per-directory` in
  `stern4rust.toml` changes the limit -- this is the only rule whose number is a
  matter of taste rather than a fact about the code, and a rule pretending
  otherwise would be ignored rather than adjusted.

  20 rather than the 12-15 first considered: measured across eight repositories,
  12 puts all eight over the line. More to the point, a tighter limit fights the
  conventions this tool enforces elsewhere -- one struct per file, one
  implemented type per file, one test file per source file all manufacture files
  by design, and a limit punishing its own standards gets worked around.

  Registries do not count: a `mod.rs`, `lib.rs` or `all_tests.rs` is an index
  *of* the directory rather than something *in* it. `main.rs` does count, being
  an entry point holding real code.

  **Ten offences across five of eight repositories**, and the symmetry is exact:
  every repository over the line is over it on both sides -- 42/39, 30/37,
  28/28, 23/24, 23/22 -- because the mirrored-test-file convention means source
  and test directories grow together. Ten offences are five restructurings done
  twice.

  **It is the first rule that cannot be satisfied by editing a file**, and the
  first that `--fix` cannot help with: the correction is a `git mv`, a new
  `mod.rs`, a `pub mod` line in the parent, and a matching move under `tests/`.

- **`directory-subfolder-count`**, the eleventh. At most 5 subfolders per
  directory, checked at every level so sprawl cannot be pushed one directory
  down. `max-subfolders-per-directory` changes the limit.

  It is the counterweight: `directory-file-count` creates folders, and without
  this the cheapest way to satisfy it is a folder per file. **It finds nothing
  across the family today** -- the deepest tree is two levels and no directory
  has more than one subfolder -- and its ADR says so plainly rather than burying
  it. It is kept because `PackageTree` already models what it needs and because
  the folders it counts are about to be created.

  Two rules rather than one with two thresholds, so that a repository can adopt
  the file cap without the folder cap: `--rule` and `--skip` work by name, and
  here it matters, since one half has ten offences and the other has none.

### Changed

- **This crate's own `src/` and `tests/` were restructured** to satisfy
  `directory-file-count`, keeping the pattern that every rule so far has been
  satisfied by the repository that wrote it rather than baselined by it. `src/`
  now holds 8 files and 5 subfolders -- `finding/`, `reporting/`, `rules/`,
  `settings/`, `adoption/` -- and `tests/` mirrors it file for file. 33 moves on
  each side, eight new registries, and every `use` path rewritten.

  `settings/` rather than `config/`, because the folder holds `config.rs` and
  `crate::config::config::Config` is a path nobody should have to read. Five
  subfolders is exactly `directory-subfolder-count`'s limit, which is fair
  warning that the two rules together leave less room than they appear to.

  **This moves every public path in the library.** Anything importing
  `stern4rust::offence::Offence` now wants
  `stern4rust::reporting::offence::Offence`.


- **`stern4rust.toml` gained `max-files-per-directory` and
  `max-subfolders-per-directory`.** Both optional; the rules supply their own
  defaults. They are config-file-only rather than CLI flags, because a directory
  limit is a fact about a repository rather than a choice for one run.

### Fixed

- **`PackageTree` did not model a directory that held no files of its own.** A
  `src/` whose sources all live one level down was invisible to the walk, so a
  rule counting what a directory contains could never ask about it. Ancestors
  are now keys too, including the package root.

## [0.4.0] - 2026-08-19

### Added

- **`--fix`.** Repairs `test-file-structure` offences -- item order within a
  section, section order and blank lines -- then reports everything it could not
  repair, unchanged. The checks run against the repaired tree, so the report
  after `--fix` is the truth after fixing rather than before, and `fixed=N`
  joins the summary.

  It exists because doing this by hand four times in one day produced three
  separate string-handling bugs. A rewriter working from `syn` spans cannot make
  them: it moves whole line ranges without reading them, so a string literal
  containing something that looks like Rust travels like any other line.

  Three constraints, each of which the first version violated and each now
  pinned by a test:

  - **It never edits a file no rule governs.** The first version rewrote thirty
    `src/` files, merging their grouped imports into one alphabetical block --
    a convention no rule checks, so no rule would have restored it, and the run
    went green over a tree nobody had reviewed.
  - **It never writes an order `cargo fmt` will undo.** The second version
    sorted imports and put `use serde_json::Value` above
    `use serde_json::from_str`, which is precisely the pair
    `test-file-structure` stands down on. Imports are now moved as a block and
    never reordered.
  - **It never loses content.** The preamble and any trailing comment are kept.

  Both bugs were found by running it and reading the diff, not by the suite,
  which was green for both -- nothing tested the blast radius. See
  `docs/ADRs/ADR-FixOnlyWhatIsSafe.md`.

- **Baselines.** `--write-baseline` records the current offences and exits
  clean; later runs judge against the recorded set and fail only on what is new.
  `--baseline <PATH>` names one explicitly, `stern4rust-baseline.json` beside
  the manifest is discovered when nobody does, and `stern4rust.toml` can carry
  the path.

  `--rule` already let a codebase enforce one rule at a time. What it could not
  express is every rule against *new* code while tolerating what is already
  there -- which for a codebase with hundreds of existing offences is the
  difference between a gate that fails forever and no gate at all.

  **Keyed on file + rule + description, never the line.** An offence that moved
  because somebody added an import above it is the same offence; a baseline
  keyed on the line would go stale on the first unrelated edit, which is exactly
  when it is most needed. Counts are recorded rather than a set, so fixing one
  of two identical offences and introducing another still passes, while
  introducing a third does not.

  **Nothing is hidden quietly.** Every run that used a baseline names it and
  states how many offences it suppressed, `baselined=N` joins the summary, and
  an entry matching nothing is reported as stale so the file can be refreshed
  rather than trusted. A baseline that was asked for and is missing is an error,
  not an empty one.

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
- **The summary line gained `fixed=N`** as well, after `baselined`.
- **The summary line gained `baselined=N`** as well, after `offences`. Always
  present, including as `baselined=0`.
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
