# R005-ADR-TestFreeSourceRule

- **Status:** Accepted
- **Date:** 2026-08-18

## Context

`#[cfg(test)] mod tests { ... }` at the bottom of a source file is the default
shape of a Rust unit test, and it is what this rule forbids.

The objection is not to unit testing. It is that a test living inside `src/` is
invisible to everything else this toolchain relies on. It does not appear in the
mirrored test file `twin4rust` checks for, so a file can be fully covered by
inline tests and still be reported as having no tests at all — or, worse, a file
with an empty mirrored test file passes `twin4rust` while its real coverage sits
somewhere nothing indexes. It is not declared from `tests/all_tests.rs`, so
[R003](R003-ADR-TestsLayoutRule.md) never sees it. It is not subject to
[R002](R002-ADR-TestFileStructureRule.md), so it has no required shape and
accretes in whatever order tests were added.

And it is compiled under a configuration the shipped build never uses. Code
behind `#[cfg(test)]` can drift out of step with the code it tests without any
build noticing, because the only build that compiles it is the one that also
compiles the tests that were written against the drift.

`#[cfg_attr(...)]` is the same door under a different name. Its entire purpose
is to make *which attribute is applied* depend on the build configuration, so
the type that ships and the type the tests see are not required to be the same
type.

## Decision

Tests live in `tests/`, and the production source tree carries none of them.
`#[cfg(test)]`, `#[cfg_attr(...)]` and test-attributed functions are all
offences under `test-free-source` wherever they appear outside `tests/`.

Three shapes are caught:

- a function carrying a test attribute, matched on the last path segment so
  `#[tokio::test]` counts without enumerating harnesses
- `#[cfg(...)]` whose predicate mentions `test`, recognised through the
  predicate rather than by matching literal text, so `any(test, ...)` and
  `not(test)` are caught too
- `#[cfg_attr(...)]`, **in every form**

The walk descends into inline modules, since nesting is where a gate is easiest
to miss by eye. An item that is itself an offence is not descended into: the
`#[cfg(test)]` module is the decision being reported, and listing the tests
inside it would report that one decision once per test.

`tests/` is exempt, and not as a concession. A `#[test]` under `tests/` is the
entire point of `tests/`, and a rule that reported it would report every test in
the workspace.

## Forcing constraints / Evidence

**`cfg_attr` is banned in every form, not only `cfg_attr(test, ...)`, and this
is the decision most worth arguing with.** The narrow version is defensible and
was rejected because the line it draws cannot be held: once
`cfg_attr(feature = "x", ...)` is permitted, the rule is committed to deciding
which conditional compilation is the acceptable kind, on a case-by-case basis,
forever. `#[cfg(feature = "...")]` remains allowed precisely because it gates
*whether code exists*, which is legible in the source; `cfg_attr` changes what
an item *means* while leaving it looking identical.

**The predicate is scanned for an identifier, not a substring.**
`#[cfg(feature = "test")]` is a feature named test and not a test gate — the
string literal never arrives as an `Ident`, so the distinction falls out of the
token walk rather than needing a special case. Pinned by
`sites_of_a_cfg_feature_named_test_reports_nothing`.

**Verified against a probe package** carrying all four shapes at once: a
non-test `cfg_attr`, a `#[cfg(test)]` module containing two `#[test]`
functions, a bare `#[test]` function, and a `#[cfg(any(test, feature = "..."))]`
module nested inside an inline `pub mod`. Four offences, one per decision, and
the nested one found two levels down.

This repository already satisfied the rule before it existed — there is no
`#[cfg(test)]` anywhere in `src/` — so it was written against probes rather than
against its own tree, and the dogfooding run confirms rather than drives it.

## Rejected alternatives

**Forbid only `cfg_attr(test, ...)`.** Rejected — see above. The narrow rule
cannot hold its own line.

**Allow `#[cfg(test)]` for a module that has a mirrored test file anyway.**
Rejected: it would make this rule depend on `twin4rust`'s answer, and it would
bless exactly the arrangement — coverage in two places — that makes a test file
hard to read.

**Report every test inside a `#[cfg(test)]` module.** Rejected: one decision,
one row. A module with thirty tests is one thing to move, and thirty rows would
bury every other file in the report.

**Match `#[cfg(test)]` by its source text.** Rejected: `any(test, ...)` and
`not(test)` gate on test just as effectively and would pass a textual match.

**Apply the rule to `tests/` as well, exempting only `#[test]`.** Rejected as
pointless complexity: nothing else this rule forbids has a reason to appear
under `tests/`, and the exemption would have to be argued for every future
shape.

## Consequences

**Unit testing a private function now requires a decision rather than a
`#[cfg(test)]` block.** That is the intended pressure — it pushes toward the
trait seams and injected dependencies this repository's own standards ask for —
but it is a real constraint and occasionally an inconvenient one. A genuinely
private helper that wants direct testing has to become `pub` within the crate,
or move behind a trait, or be tested through its caller.

**`#[cfg_attr]` is unavailable for legitimate non-test uses**, most obviously
`#[cfg_attr(feature = "serde", derive(Serialize))]`. A crate that needs optional
derives must find another shape — a feature-gated `impl`, or an unconditional
derive. This is the strictest thing this tool asks for and the most likely to be
revisited.

**Doctests are unaffected**, since they live in `///` comments and never reach
the syntax tree.

**What this rule does not catch.** A test-only helper in `src/` that carries no
test attribute and no `cfg` gate — an ordinary `pub fn make_test_widget()` — is
invisible to it, because nothing in the source distinguishes it from production
code. `crap4rust`'s cross-file test-module exclusion has the same blind spot for
the same reason. The rule also says nothing about a file in `tests/` that
mirrors nothing, which is `twin4rust`'s subject.

## Enforcement

`tests/unit_test_finder_tests.rs` — 17 tests covering all three shapes, the
`any(test, ...)` predicate, the feature-named-test non-case, both `cfg_attr`
spellings, nesting inside an inline module, and the mirrored path in the
correction.

`tests/rules/test_free_source_rule_tests.rs` — 8 tests covering the rule
itself, including that `tests/` is exempt and that the correction names the
mirrored file.

`tests/rule_registry_tests.rs` pins that the rule is registered and needs no
configuration. `run_stage_2.ps1` runs the compiled binary against this crate's
own tree, which has never contained a `#[cfg(test)]`.

## Related

- [R002-ADR-TestFileStructureRule](R002-ADR-TestFileStructureRule.md) — the
  shape a test file must have, which an inline test module escapes entirely.
- [R003-ADR-TestsLayoutRule](R003-ADR-TestsLayoutRule.md) — the other half of
  "a test that nothing compiles cannot fail", one level up.
