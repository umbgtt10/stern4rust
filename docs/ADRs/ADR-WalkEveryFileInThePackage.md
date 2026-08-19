# ADR-WalkEveryFileInThePackage

- **Status:** Accepted
- **Date:** 2026-08-19
- **Supersedes:** the nested-package skip introduced in `0.2.0`

## Context

`0.2.0` taught `SourceWalker` to skip any directory holding its own
`Cargo.toml`. The argument was clean: a nested package is a different package,
its files are that package's to answer for, and cargo would not compile them as
part of this one either. It worked — `crap4rust` went from 94 files scanned to
67 and shed 31 offences against fixture crates it had never written.

What it actually did was let a linter decline to look at part of a tree and say
nothing about it. A run reported `files_scanned=67` where the tree held 94
`.rs` files, and no line of the report accounted for the other 27. That is the
same shape of silence this tool refuses everywhere else — the omitted-offence
note, the rules-not-applied roster, `readable-source` reporting a file it cannot
parse. Every one of those exists because a report that quietly covers less than
it appears to is worse than one that covers less and says so.

The skip was also solving the wrong problem in the wrong place. Sample code that
a tool analyses is *input*, and input does not belong inside the package that
ships. `braintax4rust` already had it right: fixtures are siblings of the
published crate, not children of it, so nothing ever has to decide whether to
look at them.

## Decision

The walker skips `target/` and `.git/` and nothing else. Every other `.rs` file
under the package root is judged, including a nested package with its own
manifest.

A manifest is a fact about cargo, not a statement about whose conventions apply.
Where a tree genuinely should not be judged, the answer is layout — move it
beside the package rather than within it — or, where it cannot be moved, an
explicit exclusion the reader can see in the report.

## Forcing constraints / Evidence

`grip4rust` was restructured to test the layout answer rather than argue it: a
virtual workspace with `core/` as the published package and the eight fixture
directories moved to a sibling `fixture/`. Its offence count went from 233 to
171 with the skip removed, its 247 tests still pass, and `cargo package` still
produces a verifying 80-file crate under the same name and version.

The restructure also exposed that ~40 of the offences previously written off as
"fixture noise" were nothing of the kind. The eight `analysis_tests.rs` files
are `grip4rust`'s own integration tests — they use `grip::app::App` and assert
scores — and they merely *lived* under `tests/fixtures/`. Moving them into
`core/tests/` attributed them correctly. A skip keyed on location had been
hiding real files belonging to the package.

The cost is real and was accepted rather than discovered: `crap4rust` still
keeps ten fixture crates inside `tests/fixtures/` and its count returns from 130
to 161.

## Rejected alternatives

**Keep the skip.** Rejected on the grounds above: it is silence, and it hid 40
of `grip4rust`'s own test files along with the fixtures.

**Keep the skip but report what it skipped.** Rejected as the worst of both — it
concedes the silence argument, then spends a report line per skipped tree to
say the tool chose not to do its job. If the tree should be judged, judge it; if
it should not, it should not be in the package.

**Add a manifest to every fixture directory so the skip covers them.** Tried on
`grip4rust` and reverted: it is exactly what that repository's `0.5.0` removed,
because a nested manifest makes `cargo package` treat the fixture as a separate
package and silently drop its files from the published crate — 28 fixture files
to 0. The trick works and the consequence is unacceptable.

## Consequences

**`crap4rust` regresses by 31 offences** until it adopts the same layout. That
is the honest number: those files are inside the package it publishes, and the
tool now says so.

Repositories that keep analysis input inside the shipped package will see counts
rise. The remedy is the `grip4rust` restructure, which is mechanical and
preserves package name, version and published contents.

`--exclude <glob>` becomes more clearly necessary rather than less, for the
trees that cannot move — vendored source, generated output. It differs from the
removed skip in the way that matters: the reader asked for it and can see it.

## Enforcement

`tests/source_walker_tests.rs::walk_finds_a_fixture_package_nested_under_tests`
and `walk_finds_a_nested_package_holding_its_own_manifest` pin the new
behaviour. Both are the inverted forms of the tests that pinned the skip, kept
rather than deleted so the reversal is legible in the history.

`walk_keeps_the_root_package_even_though_it_holds_a_manifest` survives unchanged
in what it asserts: it once guarded an exemption from the skip, and now pins
that no such skip exists to need exempting.

## Related

- [ADR-MachineReadableReport](ADR-MachineReadableReport.md) — the same refusal,
  applied to a truncated report.
- [ADR-RuleSelection](ADR-RuleSelection.md) — the same refusal, applied to a run
  that did not apply every rule.
