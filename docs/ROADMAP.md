# stern4rust Roadmap

Where this tool is going, and what it will not do.

## Product Direction

`stern4rust` holds a Rust workspace to its house coding rules and names every
file that breaks one, with what to do about each. It is the conventions member
of a family whose other members measure things: `crap4rust` scores complexity
against coverage, `iceberg4rust` scores private implementation risk,
`grip4rust` scores dependency grip, `twin4rust` pairs source files with test
files. This one has no score. A rule is satisfied or it is not.

The rules are the things a reviewer would otherwise have to say by hand, every
time, forever — and the ones that quietly stop being true across a codebase the
moment nobody is checking.

## Guiding Principles

1. **A finding is worth as much as what can be done with it.** Every offence
   names its subject and carries a correction. Required, not optional.
2. **Silence is never success.** A file the tool cannot read is reported, not
   skipped. A capped report says how much it withheld. A rule that was not
   configured is named as not having run.
3. **Fail the build, don't warn.** Only exit `2` is a finding, and it is
   distinct from `1` so a wrapper can tell a real failure from a broken step.
4. **Self-gating.** A rule that would fail this repository cannot be merged into
   it.
5. **AST-structural, never type resolution.** The tool runs on any
   syntactically valid source and needs no build.
6. **One ADR per rule**, saying what it does *not* catch.

## Current Baseline

Five rules, two output formats, 242 tests, both gates green. Surveyed against
six sibling repositories: 338 files, 717 offences, of which 101 are known
false positives from unmanifested fixture trees.

## Planned Phases

### Phase 1: Release 0.2.0

crates.io still carries the ruleless `0.1.0` scaffold. Everything below waits on
this. The changelog must call out that an unreadable source file moved from exit
`1` to exit `2`, which is a change to a published interface.

### Phase 2: Excludability

The largest source of wrong answers today. `--exclude <glob>`, repeatable,
matched against the package-relative path. Closes the 101 grip4rust fixture
offences that the nested-package skip cannot reach, and gives any repository a
way to keep generated or vendored trees out of the report.

Likely paired with a `stern4rust.toml` so the excludes live with the repository
rather than in every invocation.

### Phase 3: Adoption Paths

Two things a large existing codebase needs before it can gate on this tool at
all:

- **Rule selection** — `--rule` / `--skip`, so a repository can enforce one rule
  today and the rest next quarter. Today it is all-or-nothing.
- **A baseline** — record the current offences and fail only on new ones. This
  is what turns a 600-offence first run from a reason not to adopt into a
  starting point.

### Phase 4: The Completeness Rule

`tests-layout` verifies a registry exists; it does not verify the registry is
complete. Resolving each `pub mod` to the file it names, and each file to the
declaration that should point at it, in both directions — closing the last
version of "a test that is never compiled cannot fail".

### Phase 5: More Rules

The set is open. Candidates, roughly in order of how often they would have
caught something:

- **AAA structure** — `// Arrange`, `// Act`, `// Assert` present and in order
  inside a test body. This was the original motivating example and is still
  unbuilt; `test-file-structure` judges the file's shape, not the body's.
- **Test naming** — `<method>_<description>_<outcome>`, where `<method>` is the
  function called in the Act section.
- **One struct with an impl block per file** — the other original example.
- **Header against the manifest** — SPDX identifier agreeing with the `license`
  field, rather than only matching a text file.
- **No re-export shims** — a module whose only content is `pub use`.

### Phase 6: Library Surface

The crate already exposes everything as a library, but nothing about that
surface is committed to. Deciding what is public API, and what a consumer
embedding the rules in their own tool can rely on.

## Deferred Ideas

- **Autofix.** Every offence already carries a correction precise enough to
  apply mechanically, and the alphabetic-ordering ones are the bulk of any real
  run. Deferred rather than rejected: rewriting somebody's test files is a much
  bigger promise than reporting on them, and the reordering experiments in this
  repository's own history destroyed file-level comment blocks twice before the
  edge cases were understood.
- **Severity levels.** Every offence is currently equal. Ranking them would let
  a report be triaged, but it would also invite arguing about the ranking rather
  than fixing the offence.
- **Per-rule thresholds.** `--offence-threshold` is global. Per-rule caps would
  let one noisy rule stop drowning the others, but rule selection solves that
  more directly.
- **A score.** Deliberately not. The family already has three tools that
  measure; this one answers yes or no.

## Success Measure

A repository that runs `cargo stern4rust` in its gate and stays green, where the
conventions hold because the build enforces them rather than because somebody
remembered. The six sibling tools are the first test of that, and none of them
passes yet.

## Revision Policy

This file states where the tool is going, not where it has been. When a phase
ships it moves to [IMPLEMENTED-FEATURES.md](IMPLEMENTED-FEATURES.md) and leaves
here; when a gap is found it goes to [OPEN_POINTS.md](OPEN_POINTS.md) and, if it
is worth planning, becomes a phase.
