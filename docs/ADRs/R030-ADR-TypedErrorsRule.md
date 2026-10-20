# R030-ADR-TypedErrorsRule

- **Status:** Proposed
- **Date:** 2026-10-20

## Context

`Result<T, String>` makes a failure a sentence. A caller can print it and do
nothing else: telling one failure from another means comparing text, and a test
that pins a failure path pins its wording, so a reworded message breaks a test
that was right. `&str` and `Box<dyn Error>` are the same in different
clothes -- an error whose kind the caller cannot name.

That costs most in a library, whose callers are code, and where a failure path
is as much a part of the contract as the success path
[R013](R013-ADR-TestedPublicApiRule.md) holds to a test. An error type of the
crate's own -- an enum, one variant per way to fail -- lets a test match the
failure it means and lets a caller handle one and pass the rest.

## Decision

**A library's public functions fail with an error type of their own.** A `pub fn`
in a package with a library target returns no `Result` whose error is `String`,
`&str` or `Box<dyn Error>` (any `dyn ...Error`).

- **A library target** is `src/lib.rs` and the files it reaches; `src/main.rs`
  and `src/bin/` of the same package are a binary's, whose errors end on a
  terminal, and are left alone.
- **Packages that are libraries only in form** -- a system-test harness, an
  `xtask` with a `lib.rs` -- skip the rule in `stern4rust.toml`
  (`[package.<name>] skip = ["typed-errors"]`), which says so where a reader
  looks rather than in a heuristic of this tool's.
- **Trait impls are left alone.** `FromStr::Err = String` is the implementor's
  choice of an associated type, and a trait declared in the crate is reported at
  its declaration.
- **One offence per function**, at its name: *"`read` fails with a `String`;
  give the crate an error enum its callers and tests can match"*.
- **Adoption is by baseline** ([ADR-Baselines](ADR-Baselines.md)).
- **Exempt:** `tests/`.

## Forcing constraints / Evidence

The census of [R028](R028-ADR-ImmutableReturnsRule.md), the same sixteen
repositories: 388 `pub` functions return a `Result`, and 206 of them -- 53% --
fail with a string or a boxed error.

Where they are decided the rule's shape:

- **The protocol libraries already comply.** `etheram-core`, `etheram-ibft`'s and
  `etheram-raft`'s protocol crates have no string error in a public signature;
  theirs are in their system-test harnesses and `xtask`s.
- **Counted by library target alone**, 199 offences: 80 in an analysis
  workspace, 46 in `etheram-ibft`, 16 in `etheram-raft`, 15 in `etheram-tools`,
  11 in `etheram-ibft-embassy`, 8 in `slotgate`, 2 to 4 in each of the rest --
  most of the latter an `xtask`'s gate returning `Result<(), String>`.
- **Counted without harnesses and `xtask`s**, the packages the skip above is
  for, 72: 49 in the analysis workspace, 13 in `etheram-tools`, 6 in
  `slotgate`, 2 each in `etheram-ibft-embassy` and `dry4rust`.

**This crate has two**, both in its `xtask` -- `CrapReportParser::parse` and
`Stage2::run` -- which its own configuration skips or the building change fixes,
before the rule can be merged.

## Rejected alternatives

**Every `pub fn`, binaries included.** Rejected: a binary's errors end as a line
on a terminal, and a typed error there is ceremony without a caller.

**Detecting harnesses by path** -- `system-tests/`, `validation/`, `xtask/` --
as the census did. Rejected: a path is a convention of one family, and this
tool's configuration already names packages; the skip says it in the open.

**Requiring a particular error crate** (`thiserror`, `anyhow`). Rejected: the
rule asks for a type the caller can name, not for how it is written, and
`anyhow::Error` is the boxed error this rule exists to avoid.

## Consequences

**A failure becomes something a test can match.** Failure paths get the same
tested contract success paths have, and a reworded message breaks no test.

**The analysis workspace pays most**, its tools having grown on
`Result<_, String>`; it adopts by baseline and pays down per crate.

### What this rule does not catch

- **A typed error with one variant, `Error(String)`** -- a string in a costume.
  Whether variants are meaningful is review's.
- **`Option` used where a failure has more than one cause**, which is a
  judgement about the domain.
- **Panics**, which are not return types at all; `clippy::unwrap_used`,
  `expect_used` and `panic` are the lints for them.

## Enforcement

To be written test-first when the rule is built: `String`, `&str`, `&'a str`,
`Box<dyn Error>` and `Box<dyn std::error::Error + Send + Sync>` reported; a
crate's own error type left alone; `src/main.rs` and `src/bin/` left alone; a
package skipping the rule; trait impls left alone; `tests/` exempt; the
unparseable file; `check_workspace`, `is_configured`, the name and the
requirement; `tests/rule_registry_tests.rs` updated; this crate's two `xtask`
functions dealt with first.

## Related

- [R013-ADR-TestedPublicApiRule](R013-ADR-TestedPublicApiRule.md) -- the
  contract a typed failure joins.
- [R028-ADR-ImmutableReturnsRule](R028-ADR-ImmutableReturnsRule.md),
  [R029-ADR-DistinctTupleElementsRule](R029-ADR-DistinctTupleElementsRule.md) --
  the other two rules about what a function returns.
- [ADR-PerPackageConfiguration](ADR-PerPackageConfiguration.md) -- how a
  harness says it is not a library.
