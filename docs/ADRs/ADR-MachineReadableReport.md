# ADR-MachineReadableReport

- **Status:** Accepted
- **Date:** 2026-08-18

## Context

The report started as one table, sized to its contents, meant for a person
reading a terminal. That is the right default and it stays the default, but it
is not consumable by anything else. Paths contain slashes and can contain
spaces; descriptions contain spaces, backticks, quotes and semicolons. Splitting
a row on whitespace is guesswork, and a consumer that guesses wrong fails
silently on exactly the offences whose text is most interesting.

There was a second, larger gap, and it was not about serialisation at all. The
report said what was wrong and never said what to do. `a registry holds nothing
but the header and pub mod declarations` is a true sentence about a file and an
instruction to nobody. Worse, the first version of the registry check discarded
the offending item entirely and emitted that one fixed sentence per stray, so a
file with four of them produced four byte-identical rows all pointing at line 1
— a report asserting a file had four problems while naming none of them.

## Decision

The table stays the default; `--format json` renders the same run as a
document; and **every offence carries a required `correction` beside its
description**, because a report worth reading is not the same as a report worth
acting on.

An offence carries four content fields beyond its location:

- `description` — what is wrong, as a sentence for a person
- `correction` — what to do about it. **Required, not optional**
- `subject` — the thing the offence is about, named
- `expected` — the correct text, where the rule knows it

`correction` is required and the other two are not, and that asymmetry is
deliberate. A rule that can say what is broken can always say how to fix it, so
making it an `Option` would only ever be used to skip the half of the report
worth acting on. `subject` and `expected` are genuinely absent for some
offences — there is no "expected text" for a missing folder — so an empty value
there is information rather than an omission.

In the table, the correction goes on its own line beneath the offence, indented
past the columns. Not a fifth column: the description column is already the
widest thing in the report and a correction is a sentence, not a field.

Offences are sorted by file, then line, then rule before rendering — sorting is
the report's business rather than any rule's. A rule states facts; their order
on the page is not one of them.

## Forcing constraints / Evidence

The four identical rows were real output, reproduced against a throwaway package
before anything was changed and again after. That is what turned "the report
could be richer" into "the report is wrong".

The escaping problem is not hypothetical either: the header rule builds its
description with `{:?}`, so quotes reach the report as a matter of course rather
than as an edge case. `render_escapes_a_description_containing_quotes` uses
exactly that shape.

The `expected` field earns its place through the header rule specifically.
Reporting only the first divergence — [R001](R001-ADR-HeaderRule.md)'s
decision, taken so that one headerless file does not bury the workspace behind
its own three rows — would otherwise force an iterative fix: correct line 1,
re-run, discover line 3, correct again. Carrying the whole expected header
collapses that to one pass without giving up the one-offence-per-file rule.

## Rejected alternatives

**Make the JSON the default, or the only format.** Rejected: it would break
every terminal and every gate script already reading the table, to serve a
consumer that has to opt in anyway.

**Enrich the text descriptions instead of adding a format.** Rejected as
insufficient rather than wrong — the descriptions did get richer, but no amount
of prose makes a whitespace-aligned table parseable.

**Make `correction` optional, matching `subject` and `expected`.** Rejected for
the reason above: the only use for the `None` case is the one that should not
exist.

**Put the correction in a fifth column.** Rejected: unreadable at any realistic
terminal width, and it would force the description column narrower to
compensate.

**Emit `subject` and `expected` only when present.** Rejected: a consumer would
have to test for the existence of a key rather than for its value. Every key is
always present so the shape never varies.

**Hand-roll the JSON.** Rejected: escaping is exactly the thing that looks
trivial and is not, and the descriptions this tool produces are full of the
characters that break naive escaping.

## Consequences

A dependency on `serde` and `serde_json`, which is the cost of not hand-rolling
escaping and is worth paying.

Every future rule must supply a correction for every offence it can emit. That
is enforced by `Offence::new`'s signature rather than by review, so it cannot be
forgotten — and it is now a standing obligation on rule ADRs, which have to
state the wording.

`ReportPrinter` and `JsonPrinter` both expose `render` returning a `String`
alongside `print`. That is what lets both formats be asserted on in tests rather
than checked for not panicking.

The JSON document's shape is now a public interface. Adding a key is safe;
removing or renaming one is not.

## Enforcement

`tests/json_printer_tests.rs` — 8 tests parsing the document back and asserting
on its structure, including the quote-escaping case and that both optional keys
are present as `null` when empty.

`tests/report_printer_tests.rs` — 5 `render` tests asserting what comes out,
including that every offence is followed by exactly one `fix:` line and that the
line is indented past the columns; plus 4 `print` tests pinning that the writing
path survives no offences, one, many, and an over-wide path.

`tests/offence_tests.rs` pins the sort key and both builders.

## Related

- [ADR-ExitCodeContract](ADR-ExitCodeContract.md) — the other half of what a
  consumer sees, for consumers that can only read a number.
- [R001-ADR-HeaderRule](R001-ADR-HeaderRule.md) — the one-offence-per-file
  decision that `expected` exists to make survivable.
- [R003-ADR-TestsLayoutRule](R003-ADR-TestsLayoutRule.md) — where the
  four-identical-rows defect lived.
