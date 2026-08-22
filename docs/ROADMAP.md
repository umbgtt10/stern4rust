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

Twenty-one rules, two output formats, autofix, baselines, exclusions, a config
file, and `--rules` to say what each rule wants without a codebase to ask
against. Every planned phase has shipped or been decided, and what each built is
in [IMPLEMENTED-FEATURES.md](IMPLEMENTED-FEATURES.md). 740 tests, both gates
green, and the tool runs against its own tree on every build with zero
offences.

## What is left

Nothing is planned. The rule set is at twenty-one, the original candidate list
has shipped in full, the library surface is decided, and every remaining gap is
a limit with no structural answer -- collected in
[OPEN_POINTS.md](OPEN_POINTS.md).

What remains is not a feature but adoption, and that has started. Five
repositories carry a `stern4rust.toml` and gate on it: `etheram-core`,
`etheram-raft`, `etheram-ibft`, `etheram-embassy` and `etheram-raft-embassy`.
The last two are green against a recorded baseline rather than against zero,
which is what adoption looks like on a codebase that did not grow up under the
rules. `etheram-ibft` was the hard one -- roughly
1,950 offences taken down to a tail that needs decisions rather than edits --
and it is where both of the `0.10.0` rule fixes came from: a codebase large
enough to contain the shapes this crate's own tree never had.

Adoption is also how this crate finds its own bugs, and the pattern is
consistent enough to state plainly: **every release since `0.10.0` was a defect
that only a real codebase could surface.** `etheram-ibft` gave 0.10.0 an import
comparator that fought `cargo fmt` and a correction that did not compile.
`etheram-embassy` -- 31 members, mostly `no_std` -- gave 0.10.1 a predicate read
without its negation, 0.10.2 a workspace run that silently dropped 26 of 390
offences, and 0.10.3 an attributed import sorted by its `#[cfg(...)]` rather
than by its path. `etheram-raft-embassy` gave 0.10.4 the same rule sorting on
the raw line, semicolon and `pub` included. `etheram-ibft-embassy` gave 0.10.5 a
manifest finding stated once per member -- 20 of them reported as 580.

Four of those are worth separating from the rest, because they are the failures
this tool exists to refuse rather than ordinary bugs. Each got the *count* or
the *advice* wrong, which is the only two things it sells.

0.10.2 reported less than it found and said nothing about it. 0.10.5 reported
twenty-nine times more than there was, and said nothing about that either --
every copy identical, the number simply tracking the member count. 0.10.3 and
0.10.4 issued corrections that would have made a file worse: following them put
an import block in an order `cargo fmt` immediately undoes, so the advice could
not be taken at all. That is one rule wrong two different ways, and both times
the giveaway was a repository where `cargo fmt --check` was clean and this tool
disagreed.

0.10.5 is the one to keep in view, because it was **self-inflicted**: the fix
for 0.10.2 removed the only thing suppressing it. Reporting less and reporting
more are the same failure wearing different clothes, and correcting one walked
straight into the other. It then survived three releases, because a number that
is too large reads as a codebase with a lot of work to do.

None of the four could have been found by testing this crate against its own
single-package, `std`, unattributed tree. A checking tool is only as honest as
the codebases it has been pointed at.

Deliberately no count here. The previous revision of this file named one, and
it was wrong within a release -- a snapshot of somebody else's tree goes stale
the moment they touch it, and a roadmap that has to be re-measured to stay true
is not stating direction. `--rule` exists so a repository can take one rule at
a time, and `--baseline` so it can take none of them today and no more of them
tomorrow.

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
remembered. Five repositories now do, two of them against a baseline. The
sibling tools -- `crap4rust`, `grip4rust`, `iceberg4rust`, `twin4rust`,
`slotgate` -- are the remaining test of it, and none of them passes yet.

## Revision Policy

This file states where the tool is going, not where it has been. When a phase
ships it moves to [IMPLEMENTED-FEATURES.md](IMPLEMENTED-FEATURES.md) and leaves
here; when a gap is found it goes to [OPEN_POINTS.md](OPEN_POINTS.md) and, if it
is worth planning, becomes a phase.
