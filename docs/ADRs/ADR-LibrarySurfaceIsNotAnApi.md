# ADR-LibrarySurfaceIsNotAnApi

- **Status:** Accepted
- **Date:** 2026-08-20

## Context

The crate ships a binary, `cargo-stern4rust`, and a library, `stern4rust`.
Everything in the library is `pub`: every rule, every finder, `Config`,
`Runner`, `Offence`. Nothing has ever said whether any of it is promised.

`ROADMAP.md` carried this as Phase 6 from the first release — "the crate already
exposes everything as a library, but nothing about that surface is committed
to". Two releases then made the question urgent rather than theoretical:

- **`0.8.0`** grouped `src/rules/` into subfolders, moving every rule's module
  path.
- The release after grouped `src/finding/` the same way, moving every finder's.

Both were breaking changes for anyone importing those paths, and both were
**forced** — `directory-file-count` refused a twenty-first file in one
directory. The tool's own rule reshaped its own library surface, twice, and no
policy said whether that was allowed.

## Decision

**The library surface is not a public API.** Depend on the binary.

Module paths, type names and function signatures may move in any release,
including a patch. Nothing in `stern4rust` is promised to a consumer, and no
change to it will be treated as breaking.

**The surface is `pub` for one reason, and it is not to offer an API.**
`CLAUDE.md` forbids unit tests — "unit tests are not allowed. Only integration
tests are" — so every test lives in `tests/` and can only reach what is `pub`.
The public surface is exactly as wide as the test suite needs it to be. It is a
consequence of the testing standard, not a design for consumers.

That also explains why `tested-public-api` finds so much to check: it holds
every `pub fn` to being called by a test, which is the correct question to ask of
a surface that exists *for* the tests.

## Forcing constraints / Evidence

The two restructures are the evidence. Neither was a design decision about the
library; both were `directory-file-count` refusing a twenty-first file, and both
moved public paths as a side effect. A crate that had promised its module paths
could not have taken either without a major version, and the rules would have
gone unwritten or the directories unsplit.

The alternative was tested by accident and failed: `0.8.0` shipped the first
restructure with the breaking change called out in the changelog and **no policy
behind the call-out**, which left a reader unable to tell whether it was a
mistake or a decision.

## Rejected alternatives

**Commit to the whole surface.** Rejected: it is every internal type in the
crate, `pub` only because unit tests are forbidden. Promising it would freeze
the internals of a tool whose own rules reshape them.

**Commit to a named subset** — `Rule`, `Offence`, `RuleRegistry`, `Runner`,
`Config`. Rejected for now, and this is the one worth revisiting. It is the
plausible shape of a real API, and nobody has asked for it. Committing to it
before a consumer exists would be guessing at what they need, and this
repository has rejected that shape of guess before.

**Make the surface private and test through the binary.** Rejected: it would
mean testing 21 rules through a process boundary and a printed report, when each
is currently testable against a string of source. The house standard that
produced the wide surface is also what makes the rules cheap to test, and that
trade is the right one.

**`#[doc(hidden)]` on everything.** Rejected as a lie of a different kind: the
items are documented and tested, and hiding them from `docs.rs` would make the
crate harder to work on without making the promise any clearer. The promise
belongs in words.

## Consequences

**A consumer embedding the rules has no supported way to do it today.** That is
the cost, stated plainly. If one appears, the named-subset alternative above is
where the conversation starts.

**Restructuring stays cheap**, which is what let `directory-file-count` be
obeyed rather than worked around, twice.

**The changelog still calls out moved paths.** Not because they are breaking
under this policy, but because a reader upgrading deserves to know why their
build stopped compiling — the same reason exit codes and offence descriptions
are documented.

## Enforcement

`src/lib.rs` carries the policy as a crate-level doc comment, so it appears at
the top of the `docs.rs` page rather than only in a file nobody reads before
depending on the crate. `README.md` states it beside the install instructions.

There is no test. Nothing about the behaviour of the tool changes, and a test
asserting that a policy exists would assert only that a comment is present.

## Related

- [ADR-ExitCodeContract](ADR-ExitCodeContract.md) — the interface that *is*
  promised: the exit codes, and the report the binary prints.
- [R010-ADR-DirectoryFileCountRule](R010-ADR-DirectoryFileCountRule.md) — the
  rule that forced both restructures.
