# R035-ADR-RecordsAndObjectsRule

- **Status:** Accepted, 2026-10-25 -- replaces [the private-fields decision](<../history/[SUPERSEDED] R035 PRIVATE FIELDS.md>)
  of 2026-10-22, built on 2026-10-23 and never released. The number stays R035; the rule's name becomes
  `records-and-objects`.
- **Date:** 2026-10-25

## Context

`private-fields` made every struct's fields private, whatever the struct does. Its adoption across the tools,
`slotgate` and `etheram-core` on 2026-10-23 showed where that lands: on plain data. A struct that only carries values
gained a getter per field, a constructor, past seven fields a `Default` and a `with_<field>` step per field, a test
calling each of them ([R013](R013-ADR-TestedPublicApiRule.md)), a mirrored test file in twin4rust's gate, and, now that
it had an impl block, a file of its own under [R007](R007-ADR-SingleImplementedTypeRule.md). `crap4rust`'s
`FunctionReport` -- eight fields -- came out of it with eight getters and eight `with_` steps that read and write
exactly what eight public fields did.

Privacy earns its keep where there is behaviour to protect: a type with methods can hold an invariant between its
fields, change their representation behind its methods, and own a lifecycle. A struct with no behaviour has no
invariant of its own to keep -- its fields are its contract -- and wrapping them in accessors adds code, tests and
files and protects nothing.

`private-fields` rejected exempting plain records because the rule would have to ask of every struct whether it is
"only data", *"a question that changes answer with the next method added"*. The question is syntactic, and the
answer changing with the next method is the point: the method that gives a struct behaviour is the moment its fields
become worth hiding, and the rule says so in the same change.

## Decision

**Every struct in `src/` is a record or an object, decided by its impl blocks, and each has one shape: a record's
fields are all `pub` and it has no accessors; an object's fields are all private.**

**Behaviour** is an inherent method that is neither a getter, a plain constructor nor a `with_` step, or an impl of
a trait outside the value traits. The terms, each a syntactic shape:

- **A getter** takes `&self` and nothing else, and its whole body reads one field: `self.f`, `&self.f`, `&*self.f`,
  or one of `.clone()`, `.as_ref()`, `.as_deref()`, `.as_str()`, `.as_slice()`, `.as_path()`, `.to_vec()`,
  `.to_string()`, `.to_owned()` on it, parentheses aside.
- **A plain constructor** has no receiver, returns `Self` or the type, and its whole body is `Self { a, b }`,
  `Type { a: a, b: b }` or `Self(a, b)` with every field a parameter -- dry4rust's `PlainConstructor`.
- **A `with_` step** takes `self` by value and one parameter, and its whole body is `Self { field, ..self }`.
- **The value traits** are `Clone`, `Copy`, `Debug`, `Display`, `Default`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`,
  `Hash`, `From`, `TryFrom`, `FromStr`, `AsRef`, `Error`, `Serialize`, `Deserialize`, `Send` and `Sync`. A fixed
  list in the rule, not configuration: a list a package could extend is a package deciding what counts as behaviour.

**A record** is a struct with no behaviour. **An object** is a struct with some. A constructor that does more than
store its parameters -- validates, computes, fills a default -- is behaviour, so its struct is an object.

The shapes:

- **A record:** every field `pub`; no getter, no plain constructor, no `with_` step. It is built by literal, a wide
  one with `..Default::default()`. Value-trait impls are allowed; they are how a record prints, compares and
  converts.
- **An object:** every field private, `pub(self)` counting as private; getters, constructors and `with_` steps as
  it needs them; no setters -- a change of state is a method that says what it does, as before.

Three offences, one per struct, at its name:

| offence | correction |
|---|---|
| `` `Export` is a record with private fields `data`; a record's fields are public `` | `make them pub: a struct with no behaviour has no invariant to keep` |
| `` `FunctionReport` is getter-only -- 8 getters, 8 `with_` steps and no behaviour `` | `delete the getters and steps, make the fields pub and build it by literal` |
| `` `Config` is an object exposing `threshold` and 2 more fields; an object's fields are private `` | `make them private: give a reader a getter and a builder a constructor` |

A record with both private fields and accessors is reported once, as getter-only. Exempt, as before: `tests/`,
`benches/`, `#[cfg(test)]` modules, and generated code a package excludes. Enums are not this rule's -- their variants'
fields are public by the language; [R037](R037-ADR-RecordHomeRule.md) decides where a data enum lives.

## Forcing constraints / Evidence

A census on 2026-10-24, read-only, with a `syn`-based program classifying every struct by the definitions above, over
the sixteen repositories of [R025](R025-ADR-PublicOrPrivateRule.md) -- `src/` only; tests, benches, examples,
fixtures, generated and retired code left out; `etheram-raft` at its pushed `phase-11`. To be re-run with the rule's
own finders before it ships.

**1,829 structs: 328 records and 1,501 objects.** Most structs carry behaviour; plain data is the minority the old rule
was paying for.

| repository | structs | records | objects | records with non-`pub` fields (fields) | getter-only (accessors) | objects exposing fields |
|---|---:|---:|---:|---:|---:|---:|
| `etheram-ibft` | 316 | 121 | 195 | 19 (76) | 11 (23) | 26 |
| `etheram-ibft-embassy` | 211 | 43 | 168 | 3 (7) | 2 (3) | 14 |
| `etheram-embassy` | 203 | 25 | 178 | 11 (33) | 13 (40) | 17 |
| `etheram-raft` | 166 | 36 | 130 | 10 (22) | 8 (20) | 11 |
| `etheram-tools` | 91 | 11 | 80 | 1 (4) | 2 (5) | 18 |
| `etheram-raft-embassy` | 81 | 16 | 65 | 0 | 0 | 10 |
| `stern4rust` | 142 | 8 | 134 | 8 (33) | 8 (56) | 0 |
| `dry4rust` | 87 | 14 | 73 | 14 (64) | 9 (49) | 0 |
| `grip4rust` | 62 | 10 | 52 | 10 (52) | 9 (85) | 0 |
| `crap4rust` | 44 | 10 | 34 | 10 (36) | 10 (50) | 0 |
| `braintax4rust` | 43 | 7 | 36 | 7 (43) | 6 (70) | 0 |
| `slotgate` | 40 | 5 | 35 | 5 (29) | 5 (50) | 0 |
| `iceberg4rust` | 38 | 9 | 29 | 9 (37) | 8 (48) | 0 |
| `twin4rust` | 32 | 6 | 26 | 6 (14) | 6 (20) | 0 |
| `etheram-core` | 15 | 0 | 15 | 0 | 0 | 0 |
| an analysis workspace | 258 | 7 | 251 | 7 (31) | 7 (38) | 0 |
| **all** | **1,829** | **328** | **1,501** | **120 (481)** | **104 (557)** | **96** |

- **The getter-only column is mostly the old rule's own adoption.** 428 of the 557 accessors are what it added on
  2026-10-23: 372 in the other tools and `slotgate`, 56 in this repository.
- **The objects exposing fields are the work that remains**, almost all in the protocol and firmware repositories --
  the cases `private-fields` was written for, unchanged here.
- **The definitions are not fragile.** Counting a constructor that fills defaults (`None`, `false`, `Vec::new()`) as
  boilerplate rather than behaviour moves three structs of 1,829; the strict definition stays.

## Rejected alternatives

**`private-fields` as built** -- every struct private. It costs the 557 accessors, their tests and their files, on
the 328 structs that have nothing to protect.

**An unread-getter offence** -- an object's getter that no `src/` code outside its file reads. A getter on a public
object is the library's API, and a consumer in another repository is invisible to this tool: the rule would ask a
library to delete what its consumers call, with nothing the library could do to satisfy it. Counting test reads
instead makes it redundant with [R013](R013-ADR-TestedPublicApiRule.md), which already demands a test call for every
public getter. Dropped on 2026-10-25; which getters an object needs stays a design question.

**One constructor per object, named `new`.** 154 objects have two or more constructors and 388 constructors are named
otherwise -- `read`, `parse`, `of`, `start`, `percent` beside `fraction` -- and the names carry what `new` would hide:
a constructor that reads a file, starts a process or takes a different unit. Rust's own guidelines and standard
library use named secondary constructors.

**Counting defaulting constructors as boilerplate.** Three structs differ; the shorter definition is the one a reader
can apply in their head.

## Consequences

**A record costs nothing beyond its fields.** [R013](R013-ADR-TestedPublicApiRule.md) asks nothing of a struct without
functions, and twin4rust asks nothing of a file that only declares. A record's value-trait impl with a body --
`Display::fmt`, `From::from` -- is logic, and twin4rust still asks for its mirrored test.

**A struct changes class with its first behaviour method**, and its fields go private in the same change. A record
whose fields came to need an invariant gains a constructor that checks it -- behaviour -- and becomes an object.

**Placement follows from the class.** [R007](R007-ADR-SingleImplementedTypeRule.md) and
[R023](R023-ADR-NamedSubjectRule.md) are amended to count objects only, so a record with a `Display` impl is not a
second subject in the file beside its consumer, and [R037](R037-ADR-RecordHomeRule.md) decides where a record lives.

**Reverting the 2026-10-23 adoption on records deletes tests** -- the tests that called the deleted getters,
constructors and steps. Each repository's list goes to the owner before anything is deleted.

### What this rule does not catch

- **An object's unneeded getters** -- dropped, above.
- **Interior mutability behind a getter**, `&Mutex<T>` or `&RefCell<T>`: [R028](R028-ADR-ImmutableReturnsRule.md)
  keeps out `&mut`, not cells.
- **A record that should have an invariant.** Whether `Range { start, end }` must keep `start <= end` is a design
  question; the rule only says that once a constructor checks it, the fields are private.
- **Enum variants' fields**, public by the language.

## Building it

Decided by the owner on 2026-10-25, with [R037](R037-ADR-RecordHomeRule.md),
[R038](R038-ADR-InjectedDependenciesRule.md) and the amendments to [R022](R022-ADR-ReturnedMutationsRule.md),
[R007](R007-ADR-SingleImplementedTypeRule.md) and [R023](R023-ADR-NamedSubjectRule.md). One commit per slice,
test-first, both gates green before each. A rule lands on a tree it finds clean: where this repository's own code has
to change in ways that delete tests, the rule lands with this repository's offences in a baseline, and the next slice
removes them -- the tests it deletes shown to the owner first.

| # | slice | status |
|---|---|---|
| 1 | These ADRs accepted; `private-fields`' ADR to `docs/history/` | **landed** |
| 2 | One classifier in `finding/parsing/`: method shapes, the value traits, record, object and builder | **landed** -- `finding/parsing/shapes/`, sharing `ImplBlocks::target` rather than copying it |
| 3 | `returned-mutations` leaves a plain constructor storing a `&mut` alone | **landed** |
| 4 | `single-implemented-type` and `named-subject` count objects only | **landed** -- through `ImplementedTypeFinder`, which `owned-functions` reads too |
| 5 | `records-and-objects` replaces `private-fields`; this repository's getter-only records in a baseline | **landed** -- a package rule, since a struct's impls may sit in other files; 8 getter-only records, baselined at the root and beside `core/Cargo.toml` for the runner's own test |
| 6 | This repository's records lose their accessors; their baseline entries go | |
| 7 | `record-home`; this repository's offences in the baseline | |
| 8 | This repository's records and enums moved; their entries go | |
| 9 | `injected-dependencies`; this repository's offences in the baseline | |
| 10 | The rule and rewriter registries take their parts; the baseline goes | |

`RULES.md`, the README, the ADR index, `IMPLEMENTED-FEATURES.md`, the roadmap, the changelog and the `CLAUDE.md`
counts change with the slice that changes the rule. The tools, `slotgate`, `etheram-core` and `etheram-raft`'s
`phase-11` follow, a repository at a time, at the owner's word.

## Enforcement

Test-first, in `tests/rules/types/records_and_objects_rule_tests.rs`: each getter shape, the plain constructor, the
`with_` step and a value-trait impl leave a struct a record; an inherent method of any other shape, a defaulting or
validating constructor and an impl of the package's own trait make it an object; a record with a private field, a
getter-only record and an object with a `pub`, `pub(crate)` or `pub(super)` field are each reported once, with the
message and correction above; a record with private fields and getters is reported as getter-only only; tuple structs
by position; a struct in an inline module; `tests/`, a `#[cfg(test)]` module, an enum and an unparseable file left
alone; the explanation, the name, `is_configured` and the requirement. `tests/rule_registry_tests.rs` lists the rule
under its new name.

## Related

- [`[SUPERSEDED] R035 PRIVATE FIELDS`](<../history/[SUPERSEDED] R035 PRIVATE FIELDS.md>) -- the decision this replaces.
- [R037-ADR-RecordHomeRule](R037-ADR-RecordHomeRule.md) -- where a record lives.
- [R038-ADR-InjectedDependenciesRule](R038-ADR-InjectedDependenciesRule.md) -- the builder, the third shape.
- [R013-ADR-TestedPublicApiRule](R013-ADR-TestedPublicApiRule.md) -- why a record needs no tests.
- [R007-ADR-SingleImplementedTypeRule](R007-ADR-SingleImplementedTypeRule.md),
  [R023-ADR-NamedSubjectRule](R023-ADR-NamedSubjectRule.md) -- amended to count objects only.
- `ADR-PlainConstructorsAreTheStructsShape`, in `dry4rust` -- the plain constructor this rule defines the same way.
