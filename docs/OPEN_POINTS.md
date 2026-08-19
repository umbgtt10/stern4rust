# Open Points

Known gaps in what is built. A checking tool is read as exhaustive by default,
so a gap nobody wrote down is indistinguishable from a gap nobody has yet hit —
which is why each rule ADR carries a "what this rule does not catch" section and
why the sharp ones are collected here.

Ordered by how likely they are to mislead somebody.

## A fixture tree without a manifest is still walked

`SourceWalker` skips any directory holding its own `Cargo.toml`, which covers
the fixture crates in `crap4rust` — 31 offences against sample code, gone. It
does nothing for a fixture tree that is *not* a package.

Measured: `grip4rust` keeps eight fixture directories under `tests/fixtures/`,
each a plain `src/` and `tests/` tree with **no `Cargo.toml` at all**. 101 of its
233 offences come from them, and every one is the tool reporting on input data
as though it were the repository's own code — demanding a `mod.rs` per folder,
the repository header, and the test-file shape.

Only an `--exclude <glob>` reaches these. Until then, the number a run reports
for such a repository is inflated and the inflation is invisible.

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

## Import ordering agrees with rustfmt only for ordinary paths

The alphabetic check stands down on any import pair involving `self`, `super`,
`crate` or an uppercase-initial path, because rustfmt orders those by its own
rules and `cargo fmt` runs first. That is deliberate, and the cost is that a
genuinely scrambled import list containing such a path goes unreported at that
pair. Verified against the sibling tools' 168 distinct import lines, which
rustfmt leaves exactly as an alphabetic sort produces them.

## There is no configuration file

`--rule`, `--skip` and `--offence-threshold` all have to be repeated at every
invocation. A `stern4rust.toml` is the natural home for a fixed selection, and
is the same decision that will have to carry `--exclude`.

## No baseline

There is no way to record the current set of offences and fail only on new ones.
`--rule` now gives a repository a way in — enforce one rule today, add the next
when it is green — but a repository that wants all five enforced against new
code while tolerating the existing offences still cannot express that.
