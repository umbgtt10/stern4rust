# R040-ADR-BehaviourTestsRule

- **Status:** Accepted, 2026-11-02
- **Date:** 2026-11-02

## Context

A record has no behaviour: its fields are its contract, and what it implements it derives
([R035](R035-ADR-RecordsAndObjectsRule.md)). A test whose subject is a record alone can only restate what the
compiler proves -- that a derived `Default` gives zeros, that a clone equals its original, that a plain constructor
keeps its arguments. The owner's rule is that such tests are not written. Nothing measures it: `tested-public-api`
asks that public functions are called ([R013](R013-ADR-TestedPublicApiRule.md)), and a record has none, so a test of
one passes every rule here.

## Decision

**A test that names a record of its package exercises some behaviour of the package.** A test under `tests/` that
names a record and calls nothing of its package but records' accessors and derived traits is reported.

- **A record** is a type `records-and-objects` calls one that also writes no trait method by hand. A derive is the
  compiler's; a hand-written `fmt`, `from_str` or `from` is code a test can pin, so its type is not a record here.
  Data enums count.
- **Behaviour** is every other method the package writes -- an object's or a builder's, a trait impl's, a trait's
  default -- and every free function in `src/`.
- **A test's calls** are its body's, the bodies of the helpers and local methods of its own file it calls, and the
  arguments of the standard macros -- `assert!`, `assert_eq!`, `vec!`, `format!` and the rest of their family --
  parsed as expressions, or scanned as tokens where they are not. A call qualified by an object is behaviour and one
  qualified by a record is not; a method call is behaviour when the package writes a method of that name.

## Forcing constraints / Evidence

On `etheram-ibft` at 2026-11-02, twin4rust's companion rule found 18 mirrored test files standing for records and
trait declarations, holding 73 tests; an unattended run had just added another, `receipt_summary_tests.rs`, for a
record it extracted. In this repository a derive that decides something is pinned by the behaviour relying on it:
the command line's format names by `Args`' tests, the offence order by `Offence`'s, the members' order by
`ordered-impl-functions`'.

## Rejected alternatives

- **Resolving types.** A method call's receiver has no type without inference; matching by name keeps the rule
  syntactic, as every rule here is, at the price of under-reporting.
- **Judging a macro the walk cannot see into.** A macro outside the standard family expands to code the walk never
  sees, so it counts as behaviour: the rule never accuses a test on a guess.
- **Judging a file rather than a test.** twin4rust's mirror rule asks per file; a test of a record can sit in the
  mirrored test file of an object, where only a per-test question finds it.

## Consequences

The rule under-reports by design: a record's method sharing its name with any method the package writes counts as
behaviour, and so does every macro outside the standard family. A test naming no record is never judged. The
correction moves the check to the test of the behaviour that uses the record, rather than deleting it.

## Enforcement

`PackageSurface` sorts a package's `src/` into records and behaviour, `TestSubjectFinder` and `TestBodyVisitor` judge
each test, and `BehaviourTestsRule::check_workspace` reports a test that names a record and reaches no behaviour.
Their tests pin each shape of call, each way into behaviour, and each way out of it.

## Related

- [R035](R035-ADR-RecordsAndObjectsRule.md) -- what a record is
- [R013](R013-ADR-TestedPublicApiRule.md) -- the converse: public behaviour no test calls
- twin4rust's `ADR-AMirrorMustBeEarned` -- the same rule per file: a mirrored test file for a source that earns none
