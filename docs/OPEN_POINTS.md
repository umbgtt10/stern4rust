# Open Points

Known gaps in what is built. A checking tool is read as exhaustive by default,
so a gap nobody wrote down is indistinguishable from a gap nobody has yet hit —
which is why each rule ADR carries a "what this rule does not catch" section and
why the sharp ones are collected here.

Ordered by how likely they are to mislead somebody.

## A fixture tree inside the package is judged like the package

The walker skips only `target/` and `.git/`. A tree of sample code that a tool
analyses — fixtures, vendored source, generated output — is judged as though
this repository had written it, and the report says so plainly rather than
quietly declining to look.

The fix is layout, not a skip, and both repositories that had the problem have
since taken it: `grip4rust` and `crap4rust` each moved their fixture trees out of
the published package into a sibling `fixture/`, leaving the analysis unchanged.
`grip4rust` is now at 53 offences over 75 files, `crap4rust` at 163 over 67.

Where a tree genuinely cannot be moved — vendored code, generated output — an
`--exclude <glob>` is the answer, because an exclusion the reader can see in the
report is not the same as a rule that silently skips. `crap4rust` already
carries `--exclude-path` for exactly this.

## The registry rule checks existence, not completeness

`tests-layout` verifies that `tests/all_tests.rs` and every `mod.rs` exist and
hold only declarations. It does not verify that the declarations are *complete*.

A `tests/rules/mod.rs` that is present, valid, and simply fails to mention
`alpha_tests` leaves `tests/rules/alpha_tests.rs` uncompiled — the same silent
failure the rule exists for, one level down, and `rustc` says nothing because an
undeclared file is not an error.

Closing it means resolving each `pub mod` to the file it names and each file to
the declaration that should point at it, in both directions. It is the obvious
next rule.

## A test-only helper in `src/` is invisible

`test-free-source` catches test attributes and `test` cfg predicates. An
ordinary `pub fn make_test_widget()` sitting in `src/` carries neither, and
nothing in the source distinguishes it from production code.

`crap4rust` has the same blind spot for the same reason. There may be no
structural answer to this one.

## The header rule compares text and nothing else

A file whose header is perfectly formatted but names the wrong copyright holder
passes, as does an SPDX identifier that disagrees with the `license` field in
`Cargo.toml`. Cross-checking the header against the manifest is a plausible
future rule and is not this one.

## Item naming is written three times

`TestFileParser::name`, `RegistryParser::label` and `UnitTestFinder::describe`
each map a `syn::Item` to a kind and an identifier, with the same match arms and
the same source-line fallback. Three copies drift.

Not yet extracted because it means changing two working files whose behaviour is
already pinned, and the shared piece is small enough that the extraction should
be deliberate rather than incidental to a bug fix.

## `readable-source` depends on `syn` accepting the file

Source using syntax newer than this crate's `syn` would be reported as
unparseable when `rustc` is perfectly happy with it. `syn` with the `full`
feature tracks stable Rust, so the window is narrow — but it is the only
false-positive class the rule has, and it would arrive as a confident wrong
answer rather than as a missing one.

## `#[cfg(...)]`-gated `pub mod` declarations are treated as ordinary

`tests-layout` does not distinguish `pub mod alpha_tests;` from
`#[cfg(feature = "x")] pub mod alpha_tests;`. Neither shape has come up in a
real tests tree; if one does, the rule would need to decide whether a
conditionally-declared test file counts as reached.

## An unparseable registry is skipped in silence

`RegistryParser::strays` returns `None` when the file does not parse, so
`tests-layout` says nothing about it. `readable-source` reports the file, so the
run is not silent overall — but the registry-specific offences are simply
absent, and nothing says they were skipped rather than satisfied.

## Import ordering agrees with rustfmt only for same-case pairs

The alphabetic check stands down on any import pair involving `self`, `super` or
`crate`, and on any pair that first differs at a segment where one side is
uppercase-initial and the other is not. rustfmt orders those by its own rules and
`cargo fmt` runs first, so demanding the alphabet would make the file
unsatisfiable rather than merely wrong. The cost is that a genuinely scrambled
import list goes unreported at such a pair.

The earlier version of this note claimed the stand-down was keyed on an import's
first segment and was verified against the sibling tools' 168 distinct import
lines. Both were true and neither was sufficient: `use serde_json::Value;` beside
`use serde_json::from_str;` share a first segment and part company at the second,
and no file in those 168 lines happened to contain such a pair. Adding one while
fixing `imported-paths` produced a file no edit could make green. The decision is
now made per pair.

Worth recording from measuring rustfmt to fix it, because it is not what anyone
would guess: **case is significant in opposite directions at the two levels.** An
uppercase-initial crate sorts behind every lowercase one (`Bbb::gamma` after
`zzz::last`); an uppercase-initial segment later in a path sorts ahead of its
lowercase siblings (`serde_json::Value` before `serde_json::from_str`). Also,
`cargo fmt` and a standalone `rustfmt <file>` disagree here -- only `cargo fmt`
matters, since that is what the gate runs.

## Nothing orders the imports of a productive file

`test-file-structure` is scoped to `tests/`, so the alphabetic and grouping
checks never look at `src/`. `imported-paths` now routinely *adds* imports to
productive files -- 201 of them across the sibling tools once its offences are
worked through -- with no rule saying where the new line lands. `cargo fmt` is
the only authority there, and it reorders a group only when it decides to rewrite
it.

Extending the ordering check to `src/` is not a new rule so much as removing a
scope restriction, but it would need the same per-pair stand-down and would
arrive as a wave of offences in files nobody has touched.

## `imported-paths` tells a module from a type by case

`Widget::new()` is left alone and `widget::new()` is reported, and the only thing
separating them is the case of the first segment. That is a naming convention,
not a resolution -- this tool has no type information and stays on the syntax
tree.

A lowercase-named type would be reported when it should not be, and an
uppercase-named module would pass when it should not. Both are conventions
violated rarely enough that the trade is worth it, but the failure would arrive
as a confident wrong answer rather than a missing one, which is the worse shape.

## There is no configuration file

`--rule`, `--skip` and `--offence-threshold` all have to be repeated at every
invocation. A `stern4rust.toml` is the natural home for a fixed selection, and
is the same decision that will have to carry `--exclude`.

## No baseline

There is no way to record the current set of offences and fail only on new ones.
`--rule` now gives a repository a way in — enforce one rule today, add the next
when it is green — but a repository that wants all eight enforced against new
code while tolerating the existing offences still cannot express that.
