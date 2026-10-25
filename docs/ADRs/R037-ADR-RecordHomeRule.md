# R037-ADR-RecordHomeRule

- **Status:** Accepted, 2026-10-25
- **Date:** 2026-10-25

## Context

[R007](R007-ADR-SingleImplementedTypeRule.md) keeps a file to one subject and leaves plain data free: *"Structs and
enums without impl blocks are unlimited -- plain data is not a subject, and a file's payload types belong beside the
subject that uses them."* Nothing decides where that is. A record shared by five files may sit in one of them, and a
record only one file uses may have a file of its own -- both pass every rule, and which one a codebase gets is taste.

[R035](R035-ADR-RecordsAndObjectsRule.md) makes a record's fields public, so the record's home and its visibility are
the last choices left about it. The owner's model, 2026-10-24: a record used by one consumer lives inside it,
private; a record used by several lives in a file of its own, `pub`.

Counting consumers alone has a blind spot this tool cannot see past: a consumer in *another repository*. A record a
library uses in one file and a second repository also names would be told to go private, the second repository would
stop compiling, and nothing in the library could satisfy both. What this tool can see is the crate's own public API,
and an external consumer can depend only on what that API names.

## Decision

**A record or a data enum lives where its consumers put it: private, inside its only consumer, or `pub`, in a file of
its own.**

- **A data enum** is an enum with no behaviour, by R035's definition -- no inherent method other than a getter, a
  plain constructor or a `with_` step, no impl of a trait outside the value traits. An enum with behaviour is a
  subject, under R007.
- **A consumer** is a file of the workspace, other than the declaring one, that names the type -- in code, in a
  macro's tokens, in a signature. A test file counts: a test can only name what is public, so a type a test names is
  one the crate shows outside the declaring file.
- **The public API** of a crate is what another crate can be handed: the signature of a `pub fn`, of a `pub` method
  and of a trait method; the type of a `pub` field of a `pub` struct; a variant of a `pub` enum; a `pub` type alias,
  `const` or `static`.

The home, first match wins:

1. **Named in its crate's public API** -- `pub`, in a file of its own. The API is the contract other repositories build
   on, whatever this workspace's consumers are.
2. **Used nowhere** -- named by no consumer, and in its own file by nothing but its declaration and its impls --
   `pub`, in a file of its own. Nothing in the workspace uses it, so it exists for consumers this tool cannot see.
3. **Named by two or more consumers, or by one in another crate** -- `pub`, in a file of its own. A record never moves
   across a crate boundary.
4. **Named by exactly one consumer, in its crate** -- private, inside that consumer's file. Its fields stay `pub`
   (R035); they are visible to the file that holds it.
5. **Used only by its declaring file's other code** -- private, where it is.

**A file of its own** is named after the type, as [R023](R023-ADR-NamedSubjectRule.md) reads "named after".

*As built, 2026-10-25:* the declaring file's other code counts as a user beside its consumers, so a record its
declaring file and one other file both use is used by two files -- home 3 -- rather than moved into the other, where
the declaring file could no longer name it. A type whose home is `pub` in a file of its own, sitting in that file but
not `pub`, is reported too: `` `LogLine` is used by 3 files but is not `pub` ``, `make it pub`. A binary target --
`src/main.rs`, `src/bin/` -- is no other crate's to reach, so its types have no consumer outside it and it has no
public API: the census of 2026-10-25 found six binaries' `Cli`s in one workspace counting each other as consumers.

One offence per type, at its declaration:

| offence | correction |
|---|---|
| `` `LogLine` is used by 3 files but declared in `parser.rs` `` | `move LogLine into log_line.rs and make it pub` |
| `` `Window` is used only by `scheduler.rs` but has a file of its own `` | `move Window into scheduler.rs and drop its pub` |
| `` `Window` lives in its only consumer, `scheduler.rs`, but is `pub` `` | `drop its pub: no other file uses it` |
| `` `Verdict` is named in the crate's public API but declared in `report.rs` `` | `move Verdict into verdict.rs and make it pub` |

Exempt: `tests/`, `benches/` and `examples/`, whose types are support, not API; generated code a package excludes.

## Forcing constraints / Evidence

The census of [R035](R035-ADR-RecordsAndObjectsRule.md), 2026-10-24 and 2026-10-25, sixteen repositories, consumers
counted in `src/` and tests left out -- so the "moves in" below are an upper bound, a record a test names staying
where it is.

**The 328 records by home:**

| | records | home |
|---|---:|---|
| named in the public API | 256 | `pub`, own file |
| internal, two or more consumers | 36 | `pub`, own file |
| internal, one consumer (one of them in another crate) | 20 | private in the consumer -- the one across crates, `pub` |
| internal, used only where declared | 16 | private, where it is |

**The API criterion is what makes the rule safe.** Counted by consumers alone, 40 single-consumer records that are part
of their crate's API would have been told to go private. With it, 35 records go private -- 19 moving into their only
consumer, 16 private where they already are -- and **none of the internal records with at most one consumer is named
by another repository**: the only two hits were two unrelated types that share the name `JsonReport`, one internal to
`dry4rust` and one to `iceberg4rust`.

**What moves.** 191 data enums join the 328 records; most already sit where the rule would put them:

| repository | records out | records in | enums out | enums in |
|---|---:|---:|---:|---:|
| `etheram-ibft` | 14 | 1 | 4 | 1 |
| `etheram-ibft-embassy` | 5 | 10 | 2 | 1 |
| `etheram-raft-embassy` | 7 | 0 | 1 | 0 |
| `etheram-embassy` | 4 | 2 | 4 | 1 |
| `etheram-raft` | 0 | 2 | 0 | 0 |
| `etheram-tools` | 1 | 1 | 1 | 1 |
| `dry4rust` | 0 | 0 | 5 | 0 |
| `stern4rust` | 0 | 1 | 1 | 0 |
| `crap4rust` | 0 | 1 | 0 | 0 |
| `twin4rust` | 0 | 1 | 0 | 0 |
| the rest | 0 | 0 | 0 | 0 |
| **all** | **31** | **19** | **18** | **4** |

*Out* is a shared or API type declared in another type's file, moving to its own; *in* is a single-consumer type
moving into its consumer.

## Rejected alternatives

**One way only** -- a shared record must move out, a single-consumer record may live either in its consumer or in a
file of its own. It removes every risk of telling a library to hide a type, and leaves two valid homes for the same
record: the anarchy this rule exists to end.

**Consumers alone, without the API criterion.** The 40 single-consumer API records above would have been made private,
breaking any repository that builds on them, with no change the library could make to satisfy the rule.

**An export list in `stern4rust.toml`** -- the library names the records it exports. Deterministic, and kept in
reserve: the census found no case that needs it, and a list is an exception someone has to maintain.

**Structs only.** A data enum used across files has the same question to answer; leaving it out keeps a pocket where
taste decides. Extended to enums by the owner on 2026-10-25.

## Consequences

**One home per record.** The house standard's *"one struct per file"* becomes *one object per file, and records where
this rule puts them* -- the repositories' `CLAUDE.md` files change with the rule.

**A residual, accepted.** A record internal to a library -- used by one of its files, in none of its public signatures
-- that another repository names directly would be told to go private, and that repository would stop compiling. It
fails loudly, never silently, and the fix is the library's: name the record in its API, and it becomes an API record.

**R007 and R023 count objects only** (their amendments), so a private record with a `Display` impl inside its
consumer's file is not a second subject there.

### What this rule does not catch

- **Consumers in other repositories** beyond what the public API names -- the residual above.
- **Two types sharing a name.** Consumers are counted by name across the workspace, so two `JsonReport`s count each
  other's consumers. The error runs toward a file of its own and `pub`, the safe direction.
- **Re-exports.** The house standard forbids re-export shims, so a type is counted where it is declared.

## Enforcement

A workspace rule ([ADR-WorkspaceRuleSeam](ADR-WorkspaceRuleSeam.md)): every file of every member read once, the
public-API identifiers gathered per crate, consumers counted per type. Test-first, in
`tests/rules/types/record_home_rule_tests.rs`: each of the five homes, for a struct and for a data enum; each offence
with its message and correction; an enum with behaviour left to R007; a consumer in another crate keeping a record
`pub`; a type named only by a test kept `pub`; a type named in a `pub fn`'s signature, a `pub` field, a trait method,
an enum variant and a `pub` alias each counted as API; `tests/` and an unparseable file left alone; the explanation,
the name, `is_configured` and the requirement.

## Related

- [R035-ADR-RecordsAndObjectsRule](R035-ADR-RecordsAndObjectsRule.md) -- what a record is.
- [R007-ADR-SingleImplementedTypeRule](R007-ADR-SingleImplementedTypeRule.md) -- one object per file; amended.
- [R023-ADR-NamedSubjectRule](R023-ADR-NamedSubjectRule.md) -- what "a file named after it" means; amended.
- [R032-ADR-SharedConstantsRule](R032-ADR-SharedConstantsRule.md) -- the same question for constants, counted by name
  across the workspace.
