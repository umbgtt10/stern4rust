# Architecture Decision Records

Each ADR documents one load-bearing decision behind `cargo-stern4rust` —
succinct, self-contained, citable on its own. As in `crap4rust` and
`twin4rust`, and unlike the larger `etheram` ecosystem repositories, these
are not priority-tiered: `stern4rust` is a single-crate CLI tool with a
small enough decision surface that a flat list is sufficient.

There is one ADR per rule. A rule is the unit a reader argues with — "why
does this fail my build?" — so it is also the unit that has to justify
itself. Decisions that are not rules (the shape of the `Rule` trait, the
exit-code contract) are recorded inside the ADR of the rule that forced
them, next to the evidence, rather than in an abstract document of their
own.

Numbers are allocated in the order the rules were built and are never reused.
A superseded ADR keeps its number and says what replaced it, so a reference
to `002` in a commit message or a code comment stays resolvable forever.

## Index

| ADR | Decision |
|---|---|
| [001-ADR-HeaderRule](001-ADR-HeaderRule.md) | The expected header is data supplied by `--header-file`, not a constant; a file satisfies the rule when its first N lines match exactly after normalisation, and exactly one offence — the first divergence — is reported per file. |
| [002-ADR-TestFileStructureRule](002-ADR-TestFileStructureRule.md) | A test file is four sections in a fixed order, each alphabetical, with spacing as part of the shape — and `Helpers` is defined by exclusion so the set of item kinds stays closed. |
| [003-ADR-TestsLayoutRule](003-ADR-TestsLayoutRule.md) | A tests folder is reached through exactly one `all_tests.rs` and a `mod.rs` in every subfolder on the way down; this is the rule that forced `Rule::check_workspace`, because the offending file is usually the one that does not exist. |
| [004-ADR-ReadableSourceRule](004-ADR-ReadableSourceRule.md) | Every `.rs` file must be readable and must parse, and failing either is an offence rather than a reason to say nothing — a file the tool cannot read otherwise reads as a file with nothing wrong with it. |

## Template

```markdown
# NNN-ADR-<Name>

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

A rule ADR carries two obligations the template does not spell out.

It must say what the rule **does not** catch. A checking tool is read as
exhaustive by default, and a gap nobody wrote down is indistinguishable from
a gap nobody has yet hit.

It must say what each of the rule's offences tells the reader to **do**.
`Offence::correction` is a required field rather than an optional one, so
every offence a rule can emit has an answer to "and now what?", and the ADR
is where the wording of those answers is argued about rather than in a diff.
