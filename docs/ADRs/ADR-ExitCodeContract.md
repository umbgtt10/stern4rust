# ADR-ExitCodeContract

- **Status:** Accepted
- **Date:** 2026-08-18

## Context

A linter is invoked by a gate script far more often than by a person, and the
only thing a gate script reliably sees is the exit code. Everything else — the
table, the JSON, the summary line — has to be parsed to be useful.

The failure this contract exists to prevent is a gate that stops meaning
anything. A script written as `tool || exit 1` treats "your code broke a rule"
and "I could not read your code" identically, which sounds harmless until the
second one starts happening silently: a bad path in CI, a package that stopped
resolving, a manifest that moved. The gate goes red for a reason nobody
investigates because it looks like the reason it always goes red, or — worse,
if the script inverts the test — goes green because the tool never ran.

## Decision

`0` every rule satisfied, `1` the tool could not run, `2` at least one rule was
broken, and **only `2` is a finding**.

The line between `1` and `2` is whether the work can still be enumerated. A bad
manifest path or an unknown package is a `1`: without it there is no list of
files to judge, and any report would be a report about nothing. A single
unreadable file is a `2`, reported against `readable-source` like any other
offence — it is a fact about the tree, and forty-nine other files are still
worth reporting on.

`RunOutcome` carries the distinction and only `main` turns it into a number, so
the whole run is reachable from a test without a process boundary.

## Forcing constraints / Evidence

The `1`/`2` split is shared with `crap4rust`, `twin4rust` and `iceberg4rust`,
which is what lets one gate script drive all four the same way. Departing from
it in this tool alone would cost more than any local benefit.

The placement of the unreadable-file case was forced by
[R004](R004-ADR-ReadableSourceRule.md). Before that rule, one unreadable file
returned `Err` from `Runner::run` and aborted everything, discarding every
offence already found in every other file — the tool reported nothing about a
package it had almost finished judging.

## Rejected alternatives

**Collapse `1` into `2`.** Rejected: this is the whole contract. A script that
cannot distinguish them cannot tell a real failure from a broken CI step.

**Keep an unreadable file at `1`.** Rejected: it would mean a run that both
found offences and could-not-run, and an exit code can only say one thing.
Reporting forty-nine files and exiting `1` would be actively misleading —
the code would say "no result" while the report contained one.

**Exit `2` for a bad manifest as well, on the grounds that any non-zero code
fails the build.** Rejected: it is true that most gates fail either way, which
is exactly why the distinction has to be built in rather than left to the
script. The code is the only channel that survives a CI system swallowing
stdout.

**A separate code per rule.** Rejected: rules are open-ended, exit codes are
not, and the report already names the rule. A number that has to be looked up
is worse than a name that is printed.

## Consequences

**This changed once, and consumers need to know.** An unreadable source file
was a `1` and is now a `2`. That is a change to a published interface, not an
addition to it. A wrapper that treated `1` as "investigate the tooling" will
now see such a file as an ordinary finding — which is the intended reading, but
it is a behavioural change and it is why this ADR exists separately from the
rule that caused it.

A gate script gets the distinction for free and does not have to parse
anything. `run_stage_2.ps1` relies on exactly this.

An unreadable file no longer stops the run, so a package with a permissions
problem produces a report plus one offence rather than a bare error. The tool
does as much of its job as remains possible.

## Enforcement

`tests/run_outcome_tests.rs` pins each code, including that `2` is what a
single offence produces.

`tests/runner_tests.rs::run_against_an_unknown_package_is_an_error` and
`run_with_an_unreadable_header_file_is_an_error` pin the `1` side — both are
could-not-run conditions that leave nothing to enumerate.

`tests/source_reader_tests.rs` pins that a file that cannot be read becomes an
`Offence` rather than an `Err`, which is what keeps it on the `2` side.

## Related

- [R004-ADR-ReadableSourceRule](R004-ADR-ReadableSourceRule.md) — the rule that
  forced the unreadable-file case onto the `2` side of the line.
- [ADR-MachineReadableReport](ADR-MachineReadableReport.md) — the other half of
  the interface a consumer sees, for consumers that can read more than a number.
