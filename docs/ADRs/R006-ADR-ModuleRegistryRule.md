# R006-ADR-ModuleRegistryRule

- **Status:** Accepted
- **Date:** 2026-08-19

## Context

A `lib.rs` or `mod.rs` is an index. It is the file a reader opens to learn what
a crate or a module contains, and the only file whose job is to answer that
question rather than to do work.

It is also the easiest place for work to accumulate. A `use` arrives because
something in the file needed it; then a small `fn` because the crate root was
convenient; then a `pub use` because a path was long. None of it is wrong on the
day it lands, and each one makes the index slightly less of an index. The end
state is a crate root that has to be read rather than scanned, which defeats the
one thing it was for.

The `pub use` case is worse than the rest. This repository's standards forbid
re-export shims outright — *"do not create modules whose sole purpose is to
re-export symbols from other modules"* — because they hide where a symbol
actually lives and make import paths lie. A registry is where a shim naturally
forms, and nothing was checking.

## Decision

A `lib.rs` or `mod.rs` outside `tests/` holds the header, the crate's inner
attributes, `extern crate alloc;`, and `pub mod` declarations. Nothing else.

Three parts of that are worth stating separately.

**Inner attributes need no exception.** `syn` keeps `#![no_std]` and
`#![forbid(unsafe_code)]` on the file rather than among its items, so a `no_std`
crate root passes without the rule knowing which attributes exist. The rule
never has to be taught a list it would then have to maintain.

**`extern crate alloc;` is the one non-`mod` item allowed.** A `no_std` crate
has to say it somewhere and the crate root is where it belongs. No other
`extern crate` has that excuse, and in 2018-and-later Rust the form is otherwise
a relic.

**`pub` is required.** A private `mod name;` in a source registry hides part of
the crate's shape from the file whose job is to state it.

`tests/` is left to [R003](R003-ADR-TestsLayoutRule.md), which asks a different
question of the same filenames and rightly gives a different answer about a
private `mod`.

## Forcing constraints / Evidence

The two rules disagree about `mod` versus `pub mod`, and that disagreement is
the reason `RegistryPolicy` exists as a type rather than as a boolean threaded
through a parser. Under `tests/` being compiled is the whole concern, so a
private `mod name;` compiles that file as well as a public one and R003 accepts
it. Under `src/` the module tree *is* the crate's shape. Two trees, two
questions, one parser — the policy is what keeps the difference explicit instead
of implicit in a call site.

Measured across seven repositories, the rule finds **24 offences in two**:

- `slotgate/src/lib.rs` — 17 `pub use crate::…` lines, a re-export shim of
  exactly the kind the standards forbid, in exactly the place they form
- `crap4rust/core/src/lib.rs` — 5 imports plus `pub fn run()` and
  `pub fn run_from_args()`, the CLI entry points living in the crate root
  instead of a module of their own

`twin4rust`, `iceberg4rust`, `grip4rust`, `braintax4rust` and `etheram-core` are
already clean, which is the shape the rule was written from: `etheram-core`'s
`lib.rs` is a header, two inner attributes and eleven `pub mod` lines.

## Rejected alternatives

**Allow `use` in a registry.** Rejected: an import in an index exists to serve
code, and there should be no code. Permitting it concedes the point.

**Allow any `extern crate`.** Rejected: `alloc` earns its exception by being
unavoidable under `no_std`. Nothing else is unavoidable, and a general exception
would readmit the 2015-edition habit the rest of the ecosystem has left behind.

**Accept a private `mod`, matching R003.** Rejected: consistency between the two
rules is worth less than each being right about its own tree. The rules are
allowed to differ where the trees differ, and this ADR is where that is written
down so the difference reads as a decision rather than an oversight.

**Extend R003 to cover `src/` instead of adding a rule.** Rejected: R003's
subject is whether a test is compiled at all, and its offences are mostly about
files that do not exist. Bolting a contents check for a different tree onto it
would give one rule two unrelated jobs and one name that describes neither.

**Enumerate the permitted inner attributes.** Rejected as unnecessary — `syn`
already separates them from items — and as a maintenance burden that would fail
the first time somebody wrote a legitimate attribute nobody had listed.

## Consequences

**Crate roots that host `main`-adjacent glue must move it.** `crap4rust`'s
`run()` and `run_from_args()` need a module. That is a real edit to two working
repositories, and the tool reports it against them until it happens.

**A re-export shim becomes a build failure rather than a style note.** That is
the intent, and it is the sharpest thing this rule does: `slotgate`'s 17 lines
have been a standing violation of the written standard with nothing to enforce
it.

`RegistryPolicy` is now a shared seam between two rules. A third registry-shaped
question — a workspace manifest, an examples directory — would extend it rather
than fork the parser again.

**What this rule does not catch.** It judges what a registry *holds*, not
whether the declarations are *complete* or *ordered*. A `lib.rs` that omits a
module which exists on disk passes, exactly as R003's registries do; that gap is
recorded there and is the same gap. It also says nothing about a `main.rs`,
which is an entry point rather than an index and legitimately holds code.

## Enforcement

`tests/rules/layout/module_registry_rule_tests.rs` — 13 tests covering each permitted
form, each forbidden one, the `tests/` exclusion that keeps R003 and this rule
from reporting the same file twice, and that every stray is reported at its own
line.

`tests/finding/model/registry_policy_tests.rs` — 8 tests pinning the two policies against each
other, including the private-`mod` disagreement in both directions and that
`extern crate alloc` is permitted under `source()` and not under `tests()`.

`just stage2` runs the compiled binary against this crate's own tree, where
`src/lib.rs` and `src/rules/mod.rs` are pure `pub mod` lists.

## Related

- [R003-ADR-TestsLayoutRule](R003-ADR-TestsLayoutRule.md) — the same shape of
  check under `tests/`, with the opposite answer about a private `mod`.
- [ADR-WalkEveryFileInThePackage](ADR-WalkEveryFileInThePackage.md) — why a
  registry inside a nested package is judged rather than skipped.
