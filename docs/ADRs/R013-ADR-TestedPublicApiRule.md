# R013-ADR-TestedPublicApiRule

- **Status:** Accepted
- **Date:** 2026-08-20

## Context

[R012](R012-ADR-TestNamingRule.md) retreated from proving that a test tests what
its name claims, because every version of that check accused correct code. The
question was still worth answering; it was the direction that was wrong.

Asked from the test, the question needs a guess about intent — what did the
author mean by this name, and does the body honour it. Asked from the **declared
entry point**, it needs no guess at all: a `pub fn` either appears at a call site
under `tests/` or it does not. That reframing is the whole reason this rule can
exist where the other one could not.

It also sidesteps the failure that killed the mirrored-source approach. Derived
methods are invisible here rather than misreported: `Default::default` and a
`ValueEnum`'s `from_str` are not `pub fn` declarations anywhere in the source, so
they never enter the count and can never be reported as untested.

This ADR is written after the fact: the rule shipped in `0.6.0` with only
changelog prose behind it.

## Decision

**Every public entry point declared in `src/` is called by at least one test.**

Two shapes count as an entry point: a free `pub fn`, and a `pub fn` in an
inherent impl block.

**Neither half of a trait counts.** A method *implementing* a trait carries no
visibility of its own and is reached through the trait rather than named, so
demanding a test call it by name asks for something the caller does not write. A
method *declared* by a trait is not an implementation at all — there is no
behaviour behind it to test.

**Matched on name and arity.** Types and parameter order are neither checked nor
checkable: at a call site `check(3, &paths)` offers two arguments and nothing
saying whether they fit `usize` and `&[&str]`. That is type inference and this
tool reads syntax. Arity is free and separates `new()` from `new(a, b)`, which is
most of what a bare name confuses.

**Call sites are gathered from macro token streams as well as from parsed
expressions**, and that is load-bearing rather than a refinement. A Rust test
puts its assertion inside `assert!` or `assert_eq!`, whose contents never become
syntax — `syn` does not descend into a macro's tokens. A collector that skipped
them would miss the one call most tests care about and report thoroughly tested
code as untested. Inside a macro the tokens are counted rather than parsed: an
identifier followed by a parenthesised group, arity taken from top-level commas.
That is looser than the parsed side and errs toward *finding* a call, which is
the safe direction here.

This is a workspace rule — the entry point is declared in one file and the call
that would exercise it lives in another — so it answers `check_workspace` and
reports against the declaring file at line 1. See
[ADR-WorkspaceRuleSeam](ADR-WorkspaceRuleSeam.md).

| offence | correction |
|---|---|
| `` `with_fixed` is public but no test calls it with 1 argument(s) `` | call `with_fixed` from a test, or stop exposing it if nothing outside needs it |

The correction offers two answers on purpose. An uncalled `pub fn` is as often
over-exposure as it is missing coverage, and a rule that only ever said "write a
test" would push people to write one for something that should have been private.

## Forcing constraints / Evidence

**84 offences across the family** when the rule was built. The two findings in
this crate are why it was kept:

**Six printer builders shipped untested in `0.4.0`** — `with_fixed`,
`with_baseline` and `with_config_file`, on both `ReportPrinter` and
`JsonPrinter`. Every one was public, every one was documented in
`IMPLEMENTED-FEATURES.md`, and no test called any of them. This is the same
class of failure as [R009](R009-ADR-RegistryCompletenessRule.md)'s
never-compiled test files: the feature looked covered and was not, and nothing
in any build output said so.

**`signature`, a method written minutes earlier**, in the rule's own supporting
type. The rule caught its own author, in the same sitting, on code that had just
been added. A rule that finds something during its own construction has
demonstrated its case better than any survey.

Re-measured for this ADR, the family stands at **72 offences**: `braintax4rust`
25, `crap4rust` 16, `etheram-core` 15, `twin4rust` 6, `grip4rust` 5,
`iceberg4rust` 4, `slotgate` 1, and **`stern4rust` 0** — this crate's twelve were
fixed before `0.6.0` shipped, which is what a self-gating tool is supposed to
force.

## Rejected alternatives

**Count the methods a `pub trait` declares.** This is what the rule did until
`etheram-core` measured it, and the reasoning was that a trait's methods are as
public as the trait. It was **incoherent**: a trait method can only be called
through an implementor, and implementors were already excused for a reason that
applies at least as strongly to declarations. The rule demanded a test for the
declaration while excusing every implementation of it.

What it produced in practice was worse than noise. `etheram-core` is a
trait-definition crate — its `src/` is nine `pub trait` declarations and almost
nothing else — and the rule reported **15 offences, every one a trait method**.
Satisfying them meant writing eight fake implementors whose only purpose was to
be asserted against: a test of the compiler, not of the code. Across the family
the change takes the rule from 72 offences to 34, and the 38 it drops are all
declarations.

**Match on name alone.** Rejected: `new()` and `new(a, b)` are different entry
points and a test of one would mark both. Arity costs nothing and removes most
of the confusion.

**Match on types or parameter order.** Rejected as impossible here — it needs
type inference, and the tool is committed to being AST-structural so it can run
on any syntactically valid source without a build.

**Skip macro token streams.** Rejected on measurement: assertions live in
macros, so this would report the most thoroughly tested functions in the
codebase as untested. It is the single largest false-positive source the rule
could have had.

**Count methods that implement a trait.** Rejected: they have no visibility of
their own and are called through the trait, so requiring a test to name one asks
for a call nobody writes.

**Report against the test tree** — "these tests do not cover X". Rejected: the
actionable file is the one declaring the entry point, and there is no particular
test file at fault for a call that does not exist anywhere.

**Require a test to *assert* something about the entry point.** Rejected: there
is no syntactic difference between a call that exercises and a call that merely
mentions, and demanding one would resurrect exactly the intent-guessing that
[R012](R012-ADR-TestNamingRule.md) abandoned.

## Consequences

**The rule under-reports, deliberately.** Two entry points sharing a name and an
arity are indistinguishable, so a test calling one marks both. It errs toward
silence rather than toward accusing tested code, the direction every rule here
leans.

**It makes `pub` cost something.** Widening visibility now obliges a test or a
justification, which is the intended pressure — but it will read as friction to
anyone who expected `pub` to be free.

### What this rule does not catch

**Whether the call tests anything.** A bare `let _ = thing.method();` in a test
file satisfies the rule completely. Calling is not testing, and nothing here can
tell the difference.

**Whether the call is on the right type.** Matching is by name and arity across
the whole test tree, so a `with_fixed` called on any type marks every
`with_fixed` in the package. This is the under-reporting above, stated as its
concrete failure.

**A call in unreachable test code.** The call site is found by scanning `tests/`,
not by following what a `#[test]` actually executes — a call inside a helper no
test ever invokes still counts.

**Anything a macro generates.** An entry point declared by a macro is not in the
syntax tree and is never counted, in either direction.

**Anything to do with a trait**, declared or implementing, by decision above.
A `pub trait` whose methods no implementor is tested through is invisible here.

**Derived methods.** `Default::default` and a `ValueEnum`'s `from_str` are not
`pub fn` declarations, so they never enter the count.

**Private code entirely.** A `fn` without `pub` is not judged here — that is
`crap4rust`'s and `iceberg4rust`'s question, not this one.

**Whether a public entry point should exist.** The rule accepts a test call as
sufficient justification for any `pub`, however unnecessary.

## Enforcement

`tests/rules/testing/tested_public_api_rule_tests.rs` — 7 tests covering the uncalled
entry point, the called one, the private function, the wrong arity, the call
made only from `src/`, the call inside an `assert!`, and the rule's name.

`tests/finding/parsing/public_entry_point_finder_tests.rs` — 7 tests on what counts as
an entry point, including the trait method and the trait *impl* method that does
not.

`tests/finding/parsing/call_site_finder_tests.rs` — 6 tests on call collection, macro
token streams included.

`tests/finding/model/public_entry_point_tests.rs` — 3 tests on the name-and-arity
identity.

Stage 2 runs the tool against this crate at zero offences, so a new `pub fn`
here cannot ship without a test calling it.

## Related

- [R012-ADR-TestNamingRule](R012-ADR-TestNamingRule.md) — the same concern from
  the other end. That rule gave up on proving a test tests what it claims; this
  one starts from the declaration instead of the name.
- [R009-ADR-RegistryCompletenessRule](R009-ADR-RegistryCompletenessRule.md) —
  the same class of silent failure: something that looks covered, is not, and
  produces no output saying so.
- [ADR-WorkspaceRuleSeam](ADR-WorkspaceRuleSeam.md) — why this rule answers
  `check_workspace` rather than `check`.
