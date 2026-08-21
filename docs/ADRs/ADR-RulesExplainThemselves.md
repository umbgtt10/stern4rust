# ADR: Rules explain themselves

**Status:** Accepted

## Context

A report names the rule that was broken and says what to do about that one
offence. It does not say what the rule is for, and it cannot be read at all
without a codebase to run against. So the question a first run raises — what
does this rule actually want, and what does satisfying it look like — has no
answer in the tool. It is answered in `README.md` and in twenty-one rule ADRs,
which is the right place for the reasoning and the wrong place for a person who
has just been told `pure-traits` is broken and wants to know what that means.

`--rules` answers it: every rule, a line of intent, a scrap of source that
breaks it, and the same scrap put right.

The decision is where that material lives.

## Decision

**Each rule answers for itself**, through an `explanation()` on the `Rule`
trait, alongside the `requirement()` it already answers.

The trait has no default bodies, by design, so adding a rule is a compile error
until it has said what it wants. That is the whole of the argument.

The alternative — a table in the printer mapping rule name to prose — is a
second idea of which rules exist, kept in step by hand. It would fall out of
date the first time a rule changed its mind without the table being told, and
nothing would fail: the listing would simply describe a rule that no longer
behaves that way, which is worse than not describing it.

`breaks` and `instead` are source rather than description because the question a
reader arrives with is *what does this look like*, and a sentence answering it
is longer and less exact than two lines of Rust.

### Both forms come from one list

`RuleListing` renders text and JSON from the same explanations, taking the
format rather than the caller choosing a printer. This is
[ADR-MachineReadableReport](ADR-MachineReadableReport.md) applied to the
listing: the two must not give different pictures, and the cheapest way to
guarantee that is for one type to read one list.

It does not live on either printer, though it began there. A printer holds what
one run found and renders that; a listing has no run behind it, so every field a
printer carries would be empty. The give-away was that both rendering functions
took no `self` and read no field.

### Every rule the registry can hold

The listing registers all twenty-one, including the two that stay out of an
ordinary run until something configures them: `header` until it is told what the
header says, and `spdx-matches-manifest` until a manifest declares a licence.
Both are handed a stand-in that nothing reads.

A listing missing a rule reads as a tool that does not have it. The first draft
supplied only the header and quietly listed twenty — caught by a test asserting
the count rather than by anyone reading the output, which is the argument for
asserting the count.

### It scans nothing

`--rules` returns before any walking, reading or judging. So it works in a
checkout with no manifest worth reading, and it cannot fail the way a run can —
the outcome is always `Clean`.

## Consequences

- Adding a rule means writing its explanation. There is no way to skip it.
- The listing and the rule cannot drift, because there is one statement of what
  the rule wants and the rule owns it.
- `--rules` sits one character from `--rule`, which selects rules to apply.
  clap's suggestion on a mistyped flag names the other, so the failure is
  visible rather than silent, but the pair is close and worth knowing about.

## Related

- [ADR-MachineReadableReport](ADR-MachineReadableReport.md) — the two forms must
  not give different pictures
- [ADR-RuleSelection](ADR-RuleSelection.md) — `--rule` and `--skip`, the flags
  `--rules` sits beside
- [ADR-LibrarySurfaceIsNotAnApi](ADR-LibrarySurfaceIsNotAnApi.md) — why the
  trait may grow a method without that being a breaking change
