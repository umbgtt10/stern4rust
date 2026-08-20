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

Twenty-one rules, two output formats, autofix, baselines, exclusions and a config
file. 610 tests, both gates green, and the tool runs against its own tree on
every build with zero offences.

## Planned Phases

Phases 1 to 6 have shipped or been decided and have left this file, as the
revision policy below requires: releasing `0.2.0`, excludability, baselines, the completeness
rule, and the rule set itself. What they built is described in
[IMPLEMENTED-FEATURES.md](IMPLEMENTED-FEATURES.md); why each rule is shaped as
it is, in [the ADRs](ADRs/README.md). Their numbers are not reused.

Phase 6, the library surface, is decided rather than built: **the library is not
a public API, and consumers depend on the binary.** Everything is `pub` because
this repository forbids unit tests, so the surface is as wide as the test suite
needs and no wider a promise than that. See
[ADR-LibrarySurfaceIsNotAnApi](ADRs/ADR-LibrarySurfaceIsNotAnApi.md).

No phases remain. The rule set is at twenty-one, the original candidate list has
shipped in full, and what is left is in [OPEN_POINTS.md](OPEN_POINTS.md).

## Deferred Ideas

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
