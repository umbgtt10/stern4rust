# R030-ADR-TypedErrorsRule

- **Status:** Accepted
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

**A library's public functions fail with an error type of their own.** A public
function of a library target returns no `Result` whose error is `String`, `&str`
or `Box<dyn Error>` (any `dyn ...Error`), wherever in its return type the
`Result` sits. Public is a `pub fn`, a `pub` method of an inherent impl, and
every method of a `pub` trait.

- **A library target** is `src/lib.rs` and the files its module declarations
  reach, as `name.rs` or `name/mod.rs`; `src/main.rs`, `src/bin/` and the
  modules only they declare are a binary's, whose errors end on a terminal, and
  are left alone.
- **A `Result`** is a path ending in `Result` with two type arguments. An alias
  with one, `io::Result<T>`, names its error elsewhere and is not judged.
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

**Re-run with this tool's finders**, the built rule against the sixteen
repositories on 2026-10-21: **250 offences**, against the census's 199 by
library target. The difference is trait declarations, which the census did not
count and the rule reports at the declaration.

| repository | offences |
|---|---:|
| an analysis workspace | 82 |
| `etheram-ibft` | 49 |
| `etheram-raft` | 20 |
| `etheram-tools` | 19 |
| `etheram-ibft-embassy` | 14 |
| `slotgate` | 11 |
| `dry4rust` | 8 |
| `etheram-raft-embassy`, `grip4rust` | 6 each |
| `etheram-core`, `etheram-embassy`, `braintax4rust`, `crap4rust`, `iceberg4rust`, `stern4rust`, `twin4rust` | 5 each |

**Five are in every `xtask` the family shares**: the `Gate` trait's `run`, the
`CommandRunner` trait's `run_streaming` and `run_capturing`, `Stage2::run` and
`CrapReportParser::parse`, each failing with a `String`. That is all of
`etheram-core`'s five -- its protocol library has none -- and all of this
crate's, and the census's line above holds for the libraries alone.

**This crate's five take typed errors** in the change that built the rule,
rather than the skip: the family's other repositories cannot skip a rule name
the released tool does not know -- it refuses the configuration -- so the skip
cannot reach them before the rule does, and the `xtask` they share is better
held to one shape. `GateFailure` has a variant per way a gate fails, beside
`StageFailure`, `LaunchFailure` and `ReportFailure`, each saying itself exactly
as the run reported before; the tests that pinned each message pin it still.

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
- **`anyhow::Error`**, and an alias such as `anyhow::Result<T>` that hides the
  error type: neither is written as a string or a boxed `dyn Error`. The rule
  judges what the signature says.

## Enforcement

`tests/rules/source/typed_errors_rule_tests.rs` -- thirteen tests: a `String`
error reported at the function with its description and correction, a `&'a str`
and a `Box<dyn std::error::Error + Send + Sync>` reported as written, a `pub`
trait's method reported at the declaration and its impl not; a typed error, a
private or `pub(crate)` function, a binary's `main.rs`, `bin/` and the modules
only it declares, `tests/` and an unparseable file left alone; `check` answering
nothing per file; `is_configured`, the name and the requirement.

`tests/finding/parsing/signatures/stringly_error_finder_tests.rs` (nine) pins
what fails with a sentence: `String`, `&str`, a boxed `dyn Error` with or without
bounds, a `Result` inside an `Option`; and not a typed error, a one-argument
alias, a boxed trait that is not an error, or a string outside any `Result`.
`tests/finding/parsing/modules/library_target_tests.rs` (five) pins the library
target's reach, and the shared pieces have tests of their own: `TypeText`,
`AuthoredSignatures::public` and `output`, `ModuleDeclarationFinder::directory_of`
and `SourceItems::in_returns`.

`xtask/tests` pins the typed errors' text -- every variant of `GateFailure`,
`StageFailure`, `LaunchFailure` and `ReportFailure` -- and every gate's failure
message as before. `tests/rule_registry_tests.rs`' four rule-name lists include
`typed-errors`, and `tests/runner_tests.rs`' `--rules` listing holds thirty-one.
Stage 2 runs this tool, built from the checkout, against this crate with every
rule applied and zero offences.

## Related

- [R013-ADR-TestedPublicApiRule](R013-ADR-TestedPublicApiRule.md) -- the
  contract a typed failure joins.
- [R028-ADR-ImmutableReturnsRule](R028-ADR-ImmutableReturnsRule.md),
  [R029-ADR-DistinctTupleElementsRule](R029-ADR-DistinctTupleElementsRule.md) --
  the other two rules about what a function returns.
- [ADR-PerPackageConfiguration](ADR-PerPackageConfiguration.md) -- how a
  harness says it is not a library.
