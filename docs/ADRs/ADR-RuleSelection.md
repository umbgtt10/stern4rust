# ADR-RuleSelection

- **Status:** Accepted
- **Date:** 2026-08-19

## Context

Every registered rule ran, always. That is the right default and it stays the
default, but it made adoption all-or-nothing, and the survey against the six
sibling tools is what turned that from a theoretical objection into a measured
one: 717 offences across 338 files, and **not one of those repositories could
gate on this tool**. A repository facing 233 offences does not switch on a gate;
it closes the report.

The same survey showed what a usable first step looks like. `grip4rust` has 233
offences across five rules — but only **6** under `header`, and three of the
eight header offences in the whole family are genuine misses worth fixing today.
A gate on one rule is a gate somebody turns on this afternoon. A gate on five is
a gate nobody turns on at all.

## Decision

`--rule <NAME>` and `--skip <NAME>`, both repeatable. Naming any rule with
`--rule` makes the selection a whitelist; `--skip` subtracts from whatever is
left. The default is every rule and nothing excluded.

**Skipping wins over selecting.** Asking for a rule and excluding it in the same
breath is a contradiction, and the safer reading of a contradiction is the
narrower one.

**Every report names the rules it applied**, and a run that did not apply all of
them says *"All applied rules are satisfied"* rather than *"All rules are
satisfied"*, naming each absence with its reason.

**There are three states, not two.** A rule turned off with `--skip` is
*skipped*. A rule that could not run because it had nothing to work from — the
header rule without `--header-file` — is *unconfigured*. Calling the second one
skipped blames the reader for a choice they did not make; calling it nothing at
all lets the run check less than it appears to. Both the summary line and the
JSON carry all three: `rules_applied`, `rules_skipped`, `rules_unconfigured`.

Two conditions are errors — exit `1`, not silence:

- **An unknown rule name.** `--skip test-file-strucutre` that quietly skipped
  nothing would look exactly like a switch that worked. The error lists the five
  valid names.
- **`--rule header` without `--header-file`.** The registry's usual habit of
  leaving an unconfigurable rule out silently is right for an omission and wrong
  for a request: asking for a rule by name and getting an empty run is worse
  than not asking.

## Forcing constraints / Evidence

Measured on `grip4rust`, the worst case in the survey:

```
all five rules:  offences=233 rules_broken=3 rules_applied=5 rules_skipped=0
--rule header:   offences=6   rules_broken=1 rules_applied=1 rules_skipped=4
```

233 is a wall. 6 is an afternoon.

The whitelist/subtract semantic is not invented here — clippy, ruff and eslint
converge on it, so nobody has to learn this one. Allowlist alone cannot express
"everything except the noisy one"; denylist alone cannot express "just this one
while we clean up". Both switches exist because both workflows are real.

`RuleRegistry::known_names` is built by constructing each rule and asking its
own `name()`, so the list the switches validate against cannot drift from the
rules themselves.

## Rejected alternatives

**A baseline instead** — record the current offences, fail only on new ones.
Rejected *for now*, not on merit: it solves adoption more completely, since a
repository could gate on all five rules immediately. It is also much more
machinery — a checked-in state file, fingerprints stable across line moves, and
a story for when the baseline goes stale. Rule selection is the cheap 80% and
ships today; whether baselines are still wanted afterwards is a separate
question.

**Only `--skip`.** Rejected: it cannot express "enforce just this one", which is
the adoption path the survey argued for.

**Only `--rule`.** Rejected: as rules are added, a whitelist silently does not
pick them up. That is good for stability and bad for coverage, and a repository
that wants everything-but-one should not have to enumerate the rest.

**Let `--rule` win over `--skip`.** Rejected: between two readings of a
contradictory instruction, the one that checks less is the one that cannot
quietly claim more.

**Treat an unknown name as a warning.** Rejected for the reason the whole tool
exists: a switch that matches nothing and says nothing is indistinguishable from
a switch that worked.

**Severity levels, so noisy rules could be filtered by rank rather than by
name.** Rejected: it invites arguing about the ranking rather than fixing the
offence, and rule selection addresses the same need without the argument.

## Consequences

**A clean run is now conditional, and the report has to say so.** This is the
third place the same principle has been applied — after the omitted-offence note
and after leaving an unconfigured rule out of the registry — and it is becoming
the tool's defining habit: what was *not* looked at is part of the finding.

Applying it fully exposed a case that had been wrong since before these switches
existed. A run without `--header-file` printed *"All rules are satisfied"* with
`rules_skipped=0`, having never applied the header rule and never naming it —
and `README.md` claimed the report said which rules ran, which it did not. The
switches did not cause that; they made it visible, because once a report has to
justify a partial run it can no longer stay silent about the one partial run it
always allowed.

`rules_applied` and `rules_skipped` join the JSON document, which is a public
interface as of `0.2.0`. Adding keys is safe; these are additions.

A configuration file is still absent, so a repository wanting a fixed selection
must repeat the flags at every invocation. That is the natural home for these
settings and is deferred to the same decision that will carry `--exclude`.

The summary line grew two fields. It stays greppable and the existing prefix is
unchanged, so a script matching `offences=N` still matches.

## Enforcement

`tests/rule_selection_tests.rs` — 10 tests covering the default, the whitelist
behaviour, skip-wins-over-select, `selects_explicitly` as distinct from
`includes`, and unknown-name detection across both switches.

`tests/rule_registry_tests.rs` pins that a selection registers only what was
selected, that a skip removes exactly one rule, that `known_names` lists all
five, and that `skipped_names` reports what the switches turned off rather than
what went unregistered.

`tests/runner_tests.rs::run_with_an_unknown_rule_name_is_an_error` and
`run_with_the_header_rule_selected_but_no_header_file_is_an_error` pin both
error paths.

`tests/report_printer_tests.rs::render_names_the_rules_that_were_not_applied`
and `render_of_no_offences_with_a_skipped_rule_says_selected_rules_are_satisfied`
pin that a partial run cannot read as a complete one — the second asserts the
absence of the unqualified sentence, not only the presence of the qualified one.

## Related

- [ADR-MachineReadableReport](ADR-MachineReadableReport.md) — the same refusal to
  let a truncated report read as a complete one.
- [ADR-ExitCodeContract](ADR-ExitCodeContract.md) — why both new failure modes
  are `1` rather than `2`: neither is a finding about the code.
