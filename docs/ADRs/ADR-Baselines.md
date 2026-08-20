# ADR-Baselines

- **Status:** Accepted
- **Date:** 2026-08-20

## Context

`--rule` gives a codebase a way in: enforce one rule today, add the next when it
is green. Measured on `braintax4rust`, that was the difference between a
204-offence report and a 50-offence one.

What it cannot express is **every rule, against new code only**. A repository
facing several hundred findings has to choose between gating on a fraction of
the rules and not gating at all — and a 600-offence first run is a reason not to
adopt rather than a starting point.

This ADR was written after the fact: baselines shipped in `0.4.0` with only
changelog prose behind them, and the fingerprint was changed later without a
record of either the original decision or the new one.

## Decision

`--write-baseline` records the current offences and exits clean. `--baseline
<PATH>` judges against one. Discovered as `stern4rust-baseline.json` beside the
manifest when nobody names one, the same way `stern4rust.toml` is.

**The fingerprint is file + rule + subject.** Three decisions hold it up.

**The line is deliberately not part of it.** An offence that moved because
somebody added an import above it is the same offence, and a baseline keyed on
the line would go stale on the first unrelated edit — useless exactly when it is
most needed, on a codebase under active change.

**The subject, not the description.** The description was the key until it was
measured against its own consequence: it made rule descriptions a **published
interface with nothing saying so**. A rule whose sentence gained a word reported
its offences as new across every repository holding a baseline, all at once, and
the only signal was a spike in the stale count. The subject is the thing the
offence is about, named, and it survives a rewrite of the sentence around it.
The description remains the fallback for the one rule that emits no subject —
`header`, which reports once per file, so the file and the rule already tell
those apart.

**Counts, not a set.** Two identical offences in one file share a fingerprint,
so the baseline records how many there were. Fixing one of two and adding
another passes; adding a third does not. A set would forgive the third silently.

**A baseline that was asked for and is missing is an error**, not an empty one.
`--baseline typo.json` would otherwise suppress nothing, report clean, and look
exactly like a run that worked.

**Every run that used one says so**, names it, and states how many offences it
hid — `baselined=N` in the summary and the JSON. A run reporting nothing while a
baseline held back four hundred findings would be the most comfortable lie this
tool could tell.

**Entries matching nothing are reported as stale.** A baseline entry describing
an offence somebody has since fixed makes the file look like it is still holding
something back. The report says how many and tells the reader to rewrite it.

## Forcing constraints / Evidence

The file is checked in, so it is sorted on the way out: a diff that reorders
itself on every run is a diff nobody reviews.

The fingerprint change was itself the evidence for keying on the subject. It
invalidated every existing baseline once — the exact failure the description key
made inevitable, triggered deliberately so that it stops recurring. The
stale-entry report and `--write-baseline` are the migration, and they already
existed.

## Rejected alternatives

**Key on the line.** Rejected: the first unrelated edit above an offence would
make it new.

**Key on the description.** Rejected on its consequence, above. It was the
original design, from before `Offence` carried a subject.

**A set rather than counts.** Rejected: fixing one of two identical offences and
adding another would pass unnoticed.

**Suppress silently.** Rejected: every run that used a baseline names it and its
count. An invisible suppression is the failure mode the whole tool is built
against.

**Store the line for information.** Rejected: a field in a checked-in file that
nothing reads is a field that will drift and then mislead.

## Consequences

**A baseline is a state file somebody must maintain.** Stale entries accumulate
as offences are fixed, and only `--write-baseline` clears them. The report says
how many, so it cannot rot silently.

**Rule subjects are now a published interface**, where descriptions were before.
That is a narrower and more stable surface — a subject is an identifier the code
already carries — but it is a promise, and renaming what a rule reports as its
subject will invalidate baselines.

**A baseline hides real findings.** That is its purpose, and the reason every
run that uses one says so in the summary and the JSON.

## Enforcement

`tests/adoption/baseline_tests.rs` — the round trip, forgiving each recorded
offence once, forgiving an offence that moved lines, reporting an offence the
baseline never saw, counting occurrences, the missing file as an error, and a
file that is not a baseline as an error.

`tests/reporting/offence_fingerprint_tests.rs` — the key: unchanged across a
line move, unchanged across a reworded description, distinct across subjects,
distinct across files and rules, and the fallback for offences with no subject.

`tests/runner_tests.rs::run_with_a_written_baseline_forgives_the_old_and_still_fails_on_the_new`
— the property end to end.

## Related

- [ADR-RuleSelection](ADR-RuleSelection.md) — the other way in, one rule at a
  time. Baselines are what that could not express.
- [ADR-ExclusionsAreCounted](ADR-ExclusionsAreCounted.md) — the same refusal in
  a different place: what was removed from a report is named, not silent.
- [ADR-MachineReadableReport](ADR-MachineReadableReport.md) — `baselined=N` in
  the summary and the JSON.
