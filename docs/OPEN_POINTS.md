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

Where a tree genuinely cannot be moved — vendored code, generated output —
`--exclude <glob>` now covers it, and reports every pattern with the number of
files it removed so that an exclusion stays something the reader can see rather
than a silence. See
[ADR-ExclusionsAreCounted](ADRs/ADR-ExclusionsAreCounted.md).

## A file declared through `#[path = "..."]` reads as undeclared

`registry-completeness` resolves a declaration to the file it names by
convention -- `mod alpha;` reaches `alpha.rs` or `alpha/mod.rs` -- and does not
understand an explicit `#[path]` attribute. A file reached that way would be
reported as never compiled when it is compiled perfectly well.

`declared-by-name` now forbids the attribute outright, package-wide, so this
false positive can only be reached by a repository that has skipped that rule.
Nothing in ten repositories uses `#[path]`, which is what the rule was written
to keep true — the failure would otherwise arrive as a confident wrong answer
from `registry-completeness`, about a file with nothing wrong with it, with
nothing connecting it to the attribute that caused it.

The gap that remains is `#[cfg_attr(..., path = "...")]`, which
`declared-by-name` deliberately allows as the one honest use of the attribute.
On the platform where it applies, `registry-completeness` still misreads it.

A related limit: the rule resolves names, not the module graph. A `mod.rs` that
is itself never declared makes its whole subtree unreachable, and each level is
reported separately rather than as one root cause.

## `pure-traits` does not see a blanket impl

The rule reads `Item::Trait` and reports any method with a default body. A
blanket impl -- `impl<T: Base> Extended for T { ... }` -- is an `Item::Impl`, so
none of it is looked at. **A trait emptied of defaults can have every one of them
restored through a blanket impl and the rule will report nothing.**

That is the same shared body in the same crate reached through a different
syntax, and it defeats the rule completely wherever somebody reaches for it. It
is not caught because catching it means deciding when a generic impl is a
convenience and when it is the whole implementation, which needs the trait
resolution this tool does not do.

Nothing in the family uses one today, so this is an untested gap rather than an
observed one. A default inherited from a supertrait in another crate is invisible
for the same reason, one level further out.

Related: the rule verifies a default body is *gone*, never that it was moved into
the implementors rather than deleted. `rustc` guarantees each implementor has *a*
body; nothing guarantees it is the right one.

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

## A baseline forgives by description, so a reworded offence comes back

Baseline entries are keyed on file + rule + description, deliberately excluding
the line so an offence that moved is still the same offence. The cost is that
**changing an offence's wording invalidates every baseline entry for it**: a
rule whose description gains a word will report its offences as new across every
repository that had baselined them, all at once.

Nothing warns about this today beyond the stale-entry count, which would spike
at the same moment. Rule descriptions are effectively part of a published
interface once baselines exist, and changing one is a breaking change that this
tool does not currently call out as such.
