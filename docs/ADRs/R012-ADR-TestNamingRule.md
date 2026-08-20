# R012-ADR-TestNamingRule

- **Status:** Accepted
- **Date:** 2026-08-20

## Context

A test's name is the only part of it anybody reads at the moment it matters.
`cargo test` prints names, not bodies, and a red line saying `scores_good`
tells the reader nothing about what broke — not what was called, not under what
conditions, not what was expected. The house standard is
`<method>_<conditions>_<result>`, and it exists so a failure list can be
triaged without opening a file.

The tension is that the standard has two halves and only one of them is
checkable. That a name has three parts is a fact about the text. That the first
part is *the method actually under test* is a claim about intent, and this tool
reads syntax with no type resolution behind it.

This ADR is written after the fact: the rule shipped in `0.6.0` with only
changelog prose behind it, and the reasoning below was recorded in commits and
measurements rather than here.

## Decision

**A test's name has at least three underscore-separated parts.** The rule reads
the name and nothing else.

That is a deliberate retreat from where the rule started, and the retreat is the
decision worth recording. Three earlier versions tried to verify that the
leading part named the method actually under test. All three were measured
across the family and all three produced confident wrong answers on correct
code, so all three were abandoned in favour of the one property that can be
checked without ever being wrong: **a name with fewer than three parts cannot
carry a method, a condition and a result**, whatever the words are.

Applies to `tests/` only, and not to `all_tests.rs` or `mod.rs`, which hold
declarations rather than tests. Only functions carrying a `#[test]` attribute
are judged — a helper beside them is not a test and has no naming obligation.

| offence | correction |
|---|---|
| `` `scores_good` has fewer than 3 parts, so it cannot say what it calls, under what conditions, and with what result `` | rename it `<method>_<conditions>_<result>`, starting with the method `scores_good` calls |

The correction names the method the test *calls* rather than asking for three
words, because a rename that satisfies the count without saying anything is the
failure mode a bare "add an underscore" would invite.

## Forcing constraints / Evidence

The three abandoned versions are the evidence, and each failed on a different
shape of correct code. Measured across **1559 tests in eight repositories**:

**Look for the leading part in the test body.** Fails on derived operators. A
test named `lt_with_a_smaller_left_is_true` exercising `a < b` calls no named
function at all — the comparison lowers to an operator, and there is no `lt` in
the syntax tree to find.

**Follow the test file's helpers transitively.** Fails on wide setup. A name is
"reachable" through whatever a broad `fn setup()` happened to touch, so the
check passed for names that had nothing to do with the test and failed for
narrow tests whose helper was specific.

**Check the name against the mirrored source file.** Fails on derives. `from_str`
on a `#[derive(ValueEnum)]` enum is a `fn` **nowhere** in the source — it is
generated — so every test of a derived method was accused.

Counting underscores instead took **592 offences down to 5**. That ratio is the
argument: 587 of the 592 were the rule being wrong, not the codebase.

Re-measured across the family for this ADR, `test-naming` still reports exactly
**5 offences, all in `grip4rust`** — `scores_good`, `scores_bad`,
`module_aggregation`, `perfect_grip` and `zero_grip`. Every one is a real
two-part name that says a subject and a verdict without saying what was called.
Every other repository in the family, including this one, is clean.

## Rejected alternatives

**Verify the leading part is the method under test**, in any of the three forms
above. Rejected on measurement, not on principle. The question is worth
answering and is answered from the other end by
[R013](R013-ADR-TestedPublicApiRule.md), which starts from the declared entry
points and so needs no guess about what a test intended.

**Require exactly three parts.** Rejected: `check_of_a_file_that_does_not_parse_reports_nothing`
is a better name than any three-word compression of it, and a rule capping
description length would be fought rather than followed.

**Require the parts to be meaningful words.** Rejected: `foo_bar_baz` passes and
there is no syntactic test that would catch it. A rule cannot check that a name
is honest.

**Apply it to helper functions too.** Rejected: a helper is not a test, is not
printed by a failing run, and `arrange_widget` is a perfectly good name for one.

**Apply it outside `tests/`.** Rejected: a `#[test]` in `src/` is already
[R005](R005-ADR-TestFreeSourceRule.md)'s offence, and reporting it twice under
two rules would double a single problem.

## Consequences

**The rule is deliberately shallow, and says so.** It buys a guarantee that
holds everywhere rather than a better guarantee that holds usually. Anyone
reading a clean `test-naming` run should understand it as "no name is
structurally incapable of describing a test", not "the names are good".

**A rename to satisfy it can be cosmetic.** `zero_grip` becomes
`zero_grip_works` and the rule falls silent while the name is no more useful.
Nothing catches that, and nothing can.

### What this rule does not catch

**Whether the first part is the method under test.** The entire point of the
retreat. `banana_when_split_returns_nothing` passes.

**Whether the name is true.** A test named `..._returns_nothing` that asserts
something was returned is invisible here.

**Whether the body has `// Arrange`, `// Act`, `// Assert`.** That is the oldest
unbuilt rule in [ROADMAP.md](../ROADMAP.md); this rule judges the name and
`test-file-structure` judges the file's shape, and neither looks inside a test
body.

**Tests generated by a macro.** `syn` does not descend into macro token streams,
so a test produced by one is neither named nor judged.

**A `#[test]` outside `tests/`**, which belongs to `test-free-source`, and
anything in `all_tests.rs` or `mod.rs`, which are registries.

**Whether the test asserts anything at all.** A three-part name on an empty body
satisfies this rule completely.

## Enforcement

`tests/rules/testing/test_naming_rule_tests.rs` — 8 tests covering the one-part name,
the two-part name, the three-part boundary, the long name, the helper function
without `#[test]`, a source file outside `tests/`, an unparseable file, and the
rule's name.

Stage 2 runs the tool against this crate, so a test added here with fewer than
three parts fails the build that introduces it.

## Related

- [R013-ADR-TestedPublicApiRule](R013-ADR-TestedPublicApiRule.md) — the question
  this rule gave up on, asked from the declared entry points instead. The two
  are halves of one concern.
- [R002-ADR-TestFileStructureRule](R002-ADR-TestFileStructureRule.md) — judges
  the shape of the file the tests sit in; this judges one test's name.
- [R005-ADR-TestFreeSourceRule](R005-ADR-TestFreeSourceRule.md) — owns the
  `#[test]` that is in the wrong tree entirely.
