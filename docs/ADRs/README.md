# Architecture Decision Records

Each ADR documents one load-bearing decision behind `cargo-stern4rust` —
succinct, self-contained, citable on its own.

There are two kinds, and the filename says which.

**`R<NNN>-ADR-<Name>.md` — one per rule.** A rule is the unit a reader argues
with when it fails their build, so it is the unit that has to justify itself.
The number is the rule's identity: it is allocated in the order the rules were
built, never reused, and a superseded ADR keeps its number and says what
replaced it, so a reference to `R002` in a commit message or a code comment
stays resolvable forever. A rule also has an identity *outside* these docs —
it prints its own name in every offence it emits — which is what the number
indexes onto.

**`ADR-<Name>.md` — everything else.** The exit-code contract, the shape of
the report, the seams in the `Rule` trait. These are unnumbered, as in
`crap4rust` and `twin4rust`, because they have no external referent to index
against and the name is a better identifier than a number would be:
`ADR-ExitCodeContract` says more than `A002` ever could.

The split is also legible in a directory listing. Anything with an `R` is a
rule; anything without is the machinery the rules run on.

## Rules

| ADR | Rule | Decision |
|---|---|---|
| [R001](R001-ADR-HeaderRule.md) | `header` | The expected header is data supplied by `--header-file`, not a constant; a file satisfies the rule when its first N lines match exactly after normalisation, and exactly one offence — the first divergence — is reported per file. |
| [R002](R002-ADR-TestFileStructureRule.md) | `test-file-structure` | A test file is four sections in a fixed order, each alphabetical, with spacing as part of the shape — and `Helpers` is defined by exclusion so the set of item kinds stays closed. |
| [R003](R003-ADR-TestsLayoutRule.md) | `tests-layout` | A tests folder is reached through exactly one `all_tests.rs` and a `mod.rs` in every subfolder on the way down. |
| [R004](R004-ADR-ReadableSourceRule.md) | `readable-source` | Every `.rs` file must be readable and must parse, and failing either is an offence rather than a reason to say nothing — a file the tool cannot read otherwise reads as a file with nothing wrong with it. |
| [R005](R005-ADR-TestFreeSourceRule.md) | `test-free-source` | Tests live in `tests/` and the production source tree carries none of them; `#[cfg(test)]`, `#[cfg_attr(...)]` in every form, and test-attributed functions are all offences outside `tests/`. |

## Everything else

| ADR | Decision |
|---|---|
| [ADR-ExitCodeContract](ADR-ExitCodeContract.md) | `0` clean, `1` could-not-run, `2` rule-broken, and the line between `1` and `2` is whether the work can still be enumerated — a bad manifest is a `1`, one unreadable file among fifty is a `2`. |
| [ADR-MachineReadableReport](ADR-MachineReadableReport.md) | The table stays the default and `--format json` renders the same run as a document; every offence carries a required `correction` alongside its description, because a report worth reading is not the same as a report worth acting on. |
| [ADR-WorkspaceRuleSeam](ADR-WorkspaceRuleSeam.md) | `Rule` carries `check_workspace` beside `check`, both defaulting to reporting nothing, because some offences are about a tree rather than a file — and the file that carries such an offence is usually the one that does not exist. |

## Template

```markdown
# R<NNN>-ADR-<Name>   (a rule)
# ADR-<Name>          (anything else)

- **Status:** Accepted | Proposed | Superseded by <ADR>
- **Date:** YYYY-MM-DD

## Context
The forces and tension this resolves.

## Decision
The choice, in one quotable sentence.

## Forcing constraints / Evidence
Why this was forced, not freely chosen — the real evidence. `N/A` if none.

## Rejected alternatives
What we did not do, and why.

## Consequences
What it commits us to; what it costs; obligations pushed onto consumers.

## Enforcement
The specific test, gate, or structural mechanism that keeps it true.
`N/A` if purely structural.

## Related
Links to other ADRs (this repo, `crap4rust`, `twin4rust` or `iceberg4rust`)
and architecture docs.
```

Fields that do not apply are marked `N/A` rather than padded. Each ADR is a
snapshot of the decision as it stands today, not a changelog — state the
current shape as fact, don't narrate what an earlier version of this
document used to say.

## Two obligations specific to rule ADRs

An `R` ADR must say what the rule **does not** catch. A checking tool is read
as exhaustive by default, and a gap nobody wrote down is indistinguishable
from a gap nobody has yet hit.

It must also say what each of the rule's offences tells the reader to **do**.
`Offence::correction` is a required field rather than an optional one, so
every offence a rule can emit has an answer to "and now what?" — and the ADR
is where the wording of those answers is argued about rather than in a diff.

Neither obligation applies to the unnumbered ADRs, which decide how the tool
behaves rather than what it requires of a codebase.
