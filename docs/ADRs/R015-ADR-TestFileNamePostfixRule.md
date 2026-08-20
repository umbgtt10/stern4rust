# R015-ADR-TestFileNamePostfixRule

- **Status:** Accepted
- **Date:** 2026-08-20

## Context

The whole test layout rests on one pairing: `src/foo.rs` answers to
`tests/foo_tests.rs`. `single-implemented-type` cites it as the reason a file
has one subject. `test-free-source` names the mirror file in its correction.
`twin4rust` enforces it in stage 2. It is the most load-bearing convention in
the family.

It was enforced from **one side only**. `twin4rust` starts at a source file and
looks for its test, so it reports a source file with no test. Nothing started
from a file full of tests and asked whether its name placed it in the pairing at
all. A `tests/rules/widget.rs` holding twenty tests was invisible to every tool
in the family: `tests-layout` cared only that a registry existed,
`registry-completeness` only that the file was declared, `test-file-structure`
only about the order of items inside it, and `test-naming` only about the names
of the functions. Every one of them would pass it, and `twin4rust` would
separately report `src/rules/widget_rule.rs` as untested while the tests for it
sat in a file none of them could connect to it.

## Decision

**A file under `tests/` holding at least one test is named `<X>_tests.rs`.**

One direction only, and stated as the deliberate half rather than an oversight:
holding a test obliges the name. A `_tests.rs` file holding **no** tests is a
different failure — tests deleted, tests commented out — with a different
correction, and it is not this rule. This mirrors
[R009](R009-ADR-RegistryCompletenessRule.md) and
[R014](R014-ADR-PureTraitsRule.md), where one direction of a two-directional
requirement was likewise left out on purpose.

A test is a function carrying an attribute whose **last path segment** is
`test`, so `#[tokio::test]` counts without this rule naming any runtime. The
walk descends into inline modules: a test does not stop being a test for sitting
one level down, and the file still holds it.

**Two exemptions, and both are load-bearing rather than softening.**

**`src/` is exempt** because a `#[test]` there is already
[R005](R005-ADR-TestFreeSourceRule.md)'s offence, and this rule's correction
would be actively **wrong**: renaming `src/foo.rs` to `src/foo_tests.rs` leaves
the test exactly where it does not belong. That file has to move, not be
renamed. Reporting it here would be a second offence for one problem, carrying
advice that does not fix it.

**Registries are exempt** for the same shape of reason. A `#[test]` in an
`all_tests.rs` or a `mod.rs` is already [R003](R003-ADR-TestsLayoutRule.md)'s
offence, and `mod.rs` **cannot be renamed at all** — the correction would be
impossible to follow.

| offence | correction |
|---|---|
| `` tests/rules/widget.rs holds 2 test(s) but its name does not end in `_tests.rs`, so nothing pairs it with the source file it exercises `` | rename it `tests/rules/widget_tests.rs` |

The correction names the exact path rather than restating the policy, and the
same string is carried in `expected` so a consumer of the JSON report can act on
it. The offence sits at line 1: the file's *name* is what is wrong, not any one
test in it. The count of tests is named as the evidence for calling it a test
file in the first place.

## Forcing constraints / Evidence

**Zero offences across the eight repositories**, and this ADR says so plainly
rather than burying it. The convention is kept everywhere, by everyone, without
being written down — the same finding [R014](R014-ADR-PureTraitsRule.md)
produced, and the same case for a rule: it guarantees the next person cannot
quietly stop keeping it.

The rule is not merely untested against real code, though. Searching the family
for test-bearing files outside the convention turned up exactly one:
`crap4rust/fixture/workspace_validation_fixture/app-validation/tests/validation_support.rs`,
and pointing the tool at that package reports it:

```
tests/validation_support.rs  1  test-file-name-postfix  tests/validation_support.rs holds 1 test(s) but its name does not end in `_tests.rs`
```

It is a **support file that grew a test** — precisely the shape the rule exists
for, and the way this drift actually happens in practice. It is not counted as a
family offence because it sits in a fixture tree deliberately moved outside the
published package (see [OPEN_POINTS.md](../OPEN_POINTS.md)), so no ordinary run
walks it. But it is real code, written by a person, doing the thing the rule
forbids.

[R011](R011-ADR-DirectorySubfolderCountRule.md) is the precedent for shipping a
rule at zero: it is kept because the machinery it needs already exists and
because the drift it prevents is cheap to fall into and expensive to notice.

## Rejected alternatives

**Check both directions**, reporting a `_tests.rs` file with no tests. Rejected
for this rule: it is a genuinely different failure with a different correction —
"restore the tests" rather than "rename the file" — and merging them would give
one rule name two meanings. Worth its own rule if it earns one.

**Require `<X>` to name a real source file.** Rejected as a different and much
harder rule. `tests/rules/banana_tests.rs` with no `src/rules/banana.rs` behind
it satisfies this rule completely, and closing that needs the name resolved
against the source tree — `twin4rust`'s pairing, approached from its blind side.
Recorded below as not caught rather than quietly attempted.

**Apply it to `src/` as well.** Rejected: see the Decision. It would double
`test-free-source` and carry a correction that does not fix the problem.

**Report at the line of the first test** rather than line 1. Rejected: the
subject is the file's name, and pointing at a test would suggest that test is
what needs changing.

**Reuse `UnitTestFinder`.** Rejected: it finds `#[cfg(test)]` and
`#[cfg_attr(test, ...)]` as well as tests, which is right for `test-free-source`
and wrong here. A file holding only test *machinery* is not a file holding
tests, and reusing the finder would have reported it.

**Match `_tests` anywhere in the name** rather than as a suffix before `.rs`.
Rejected: it would accept `widget_tests_helper.rs`, which is not a test file.

## Consequences

**It commits the repository to a filename convention that `cargo` also sees.**
For a package using the default `autotests` behaviour, every file directly under
`tests/` is its own test binary, so satisfying this rule renames test targets.
Harmless, but adopters should know the rename is not purely cosmetic there. This
crate is unaffected: `autotests = false` with a single `all_tests` target means
every test file is a module.

**A support file may not grow a test in place.** Adding one `#[test]` to
`tests/rules/support.rs` now fails the build until the file is split or renamed —
which is the intent, and is also the single most likely way anybody meets this
rule.

### What this rule does not catch

**An orphan test file.** `tests/rules/banana_tests.rs` full of tests with no
`src/rules/banana.rs` behind it passes completely here. The `<X>` is never
resolved against anything, so this rule closes the "test file not named as one"
gap and **not** the "test file pairing with nothing" gap. That second gap is
[R016](R016-ADR-PairedTestFileRule.md)'s, which resolves the name against the
source tree.

**A `_tests.rs` file with no tests**, by decision, above.

**Tests generated by a macro.** `syn` does not descend into macro token streams,
so a file whose tests all come from a macro reads as holding none and may be
named anything.

**Whether the name matches the *right* source file.** `tests/rules/widget_tests.rs`
containing tests for something else entirely satisfies the rule.

**A `#[test]` in `src/` or in a registry**, both by decision, and both already
reported by the rule that owns them.

**Anything outside `tests/`.** A test in `benches/` or `examples/` is not judged.

## Enforcement

`tests/rules/test_file_name_postfix_rule_tests.rs` — 12 tests covering the
misnamed file, the correctly named file, the file with no tests, the
`#[tokio::test]` attribute, the test inside an inline module, both registry
kinds, a source file, an unparseable file, `check_workspace`, `is_configured`,
and the rule's name. The misnamed case asserts the exact suggested path in both
`correction` and `expected`.

`tests/rule_registry_tests.rs` — the four hardcoded rule-name lists include
`test-file-name-postfix`.

Stage 2 runs the tool against this crate at zero offences, so a test added to a
file outside the convention fails the build that introduces it.

## Related

- [R005-ADR-TestFreeSourceRule](R005-ADR-TestFreeSourceRule.md) — owns the
  `#[test]` in `src/`, which is why this rule exempts that tree rather than
  reporting it twice with the wrong correction.
- [R003-ADR-TestsLayoutRule](R003-ADR-TestsLayoutRule.md) — owns the `#[test]`
  in a registry, for the same reason.
- [R012-ADR-TestNamingRule](R012-ADR-TestNamingRule.md) — names the test
  functions inside the file; this names the file holding them.
- [R002-ADR-TestFileStructureRule](R002-ADR-TestFileStructureRule.md) — governs
  the order of items within a test file, and says nothing about its name.
- [R016-ADR-PairedTestFileRule](R016-ADR-PairedTestFileRule.md) — requires the
  name to *resolve* to a source file. This rule requires a file holding tests to
  be named as one; together they are the two halves.
