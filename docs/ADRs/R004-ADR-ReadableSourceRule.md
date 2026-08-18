# R004-ADR-ReadableSourceRule

- **Status:** Accepted
- **Date:** 2026-08-18

## Context

Two of the three rules that existed before this one parse the file they judge,
and both give up in silence when the parse fails. That was a deliberate
choice, recorded in [R002-ADR-TestFileStructureRule](R002-ADR-TestFileStructureRule.md):
`rustc` reports broken source far more clearly than this tool could, and
inferring a shape from source that does not parse piles noise on top of a
compile error.

The reasoning is sound for a file somebody is actively editing. It is wrong
for a file nobody is looking at. A file the tool cannot read produces no rows,
and a report with no rows for a file is indistinguishable from a report on a
file that is clean. The package looks *better* than it is, and it looks better
in exactly the situation where something has gone wrong.

This is not hypothetical. While rule 3 was being built, a probe file on disk
became a run of 41 NUL bytes. The tool reported one fewer offence than the tree
contained and said nothing about the file at all; the drop in the offence count
was noticed only because the same probe had been run minutes earlier and the
before-and-after happened to be on screen together. Nothing in the report
pointed at the file, and nothing would have.

The same blind spot had a second half. A file that could not be *read* — as
opposed to not parsed — aborted the entire run with exit 1, so one unreadable
file discarded every offence already found in every other file in the package.

## Decision

Every `.rs` file must be readable and must parse, and failing either is an
offence under a single rule named `readable-source` rather than a reason to
say nothing.

The rule is registered first, ahead of every other rule, because its failure
is the one that explains every other rule's silence about the same file.

`SourceReader` owns the read half. It returns `Result<SourceFile, Offence>`
rather than `Result<SourceFile>`, so a read failure becomes a finding in the
report instead of an early return out of the run. It reports under
`ReadableSourceRule::NAME`, which is the single place that string is written.

## Forcing constraints / Evidence

The NUL-byte file above is the whole argument, and it is now the rule's first
test: `check_a_file_of_nul_bytes_reports_it` uses exactly that input, 41 NUL
bytes, because that is what actually happened rather than what seemed likely to.

Verified end to end after the fact against a probe package whose only test file
was those 41 bytes. Before this rule the run reported the package's header
offences and nothing else; after it, the same run adds
`file does not parse as Rust: cannot parse string into token stream`.

## Rejected alternatives

**Leave both as they were and document them.** Rejected: this was the option
on the table, and the deciding argument against it is that the failure mode is
invisible rather than merely unreported. Every other gap in this tool produces
a report that is incomplete in a way a reader can eventually notice. This one
produces a report that is *wrong* in the direction of reassurance.

**Fix the abort but keep the parse skip.** Rejected for the same reason: the
NUL-byte file was readable. It was the parse that failed, so fixing only the
read half would not have caught the case that motivated the work.

**Report the parse failure from each parsing rule.** Rejected: three rules
would each report the same file, and the reader would be told three times that
one file does not parse. One rule owning the question means one row.

**Make the unreadable file keep its exit-1 status while still reporting.**
Rejected — see [ADR-ExitCodeContract](ADR-ExitCodeContract.md), which this rule
forced a change to and which argues the case properly.

## Consequences

**The exit-code contract moved because of this rule.** An unreadable source file
used to exit 1 and now exits 2. That is a change to a published interface rather
than an addition to it, and it is argued and recorded in
[ADR-ExitCodeContract](ADR-ExitCodeContract.md) rather than here — a consumer
checking what a code means should not have to read a rule ADR to find out.

The rule depends on `syn` accepting the file, so source using syntax newer than
this crate's `syn` would be reported as unparseable when `rustc` is perfectly
happy with it. `syn` with the `full` feature tracks stable Rust, so the window
is narrow, but it is a real false-positive class and the only one this rule has.

A parse error's span can be the call site rather than a position in the file, so
`line_of` reports line 1 rather than line 0 — a line no editor can navigate to
is worse than an approximate one.

**What this rule does not catch.** It answers "can this be read and parsed",
not "is this valid Rust". A file that parses but does not compile — an unknown
type, a borrow error, a missing import — is this rule's idea of fine, and
rightly so: that is `rustc`'s job, and duplicating it badly would be worse than
not doing it.

## Enforcement

`tests/rules/readable_source_rule_tests.rs` — 7 tests, including the NUL-byte
case that motivated the rule, an empty file (valid Rust, not an offence), and
the line the parse failed on.

`tests/source_reader_tests.rs` — 4 tests covering the read half, using a path
that does not exist so the failure branch is reachable without contriving a
genuinely unreadable file.

`tests/rule_registry_tests.rs` pins that the rule is registered first and needs
no configuration to hold.

## Related

- [R002-ADR-TestFileStructureRule](R002-ADR-TestFileStructureRule.md) — records
  the give-up-in-silence decision this rule is the counterweight to. Both still
  stand: the structure rule stays quiet about source it cannot parse, and this
  rule makes sure the file is named anyway.
- [R003-ADR-TestsLayoutRule](R003-ADR-TestsLayoutRule.md) — same shape of failure
  one level up: a test that is never compiled cannot fail, and a file that is
  never parsed cannot offend.
