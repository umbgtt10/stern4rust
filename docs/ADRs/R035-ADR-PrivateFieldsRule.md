# R035-ADR-PrivateFieldsRule

- **Status:** Accepted
- **Date:** 2026-10-22, built 2026-10-23

## Context

A struct's fields are its state. A public field lets every file read it and
write it, so the type can keep no invariant of its own -- whatever must hold
between two fields holds only as long as every writer remembers -- and a change
of representation breaks every reader at once. Restricted visibility is the same
inside the crate: `pub(crate)` is a public field with a shorter list of readers.

The case that forced the question: `etheram-raft`'s gRPC transport and client
interface each hold their server's cancellation token and thread handle in
`pub(crate)` fields of their inner state. The adapter's constructor writes the
handle in from outside; the inner state's `Drop` stops the server from inside. A
lifecycle that starts in one file and ends in another, through fields neither
file owns.

[R025](R025-ADR-PublicOrPrivateRule.md) gave a method two honest states, `pub`
or private. A field gets one.

## Decision

**A struct's fields are private.** In `src/`, a struct field declared `pub`,
`pub(crate)`, `pub(super)` or `pub(in ...)` is an offence.

- **Every struct:** named fields and tuple fields alike, so a newtype's
  `struct Height(pub u64)` is one. Enum variants' fields are public by the
  language and are not this rule's.
- **Reading goes through a getter** -- a method that returns the field: by value
  where it is `Copy`, by reference otherwise.
  [R028](R028-ADR-ImmutableReturnsRule.md) already keeps `&mut` out of every
  return type, so a getter hands out nothing a caller can change.
- **Building goes through a constructor**, and a record too wide for one --
  clippy's `too_many_arguments` stops at seven -- through a builder, or a
  `Default` and `with_<field>(self, value) -> Self` steps.
- **No setters.** A change of state is a method that says what it does, or a
  `with_<field>` that returns the new value.
- **One offence per struct**, at its name, naming its non-private fields:
  *"`Config` exposes `baseline`, `config_file` and 15 more fields; a struct's
  fields are private"*. The correction: make them private, give readers a getter
  and builders a constructor.
- **Exempt:** `tests/`, `benches/` and `#[cfg(test)]` modules -- test support
  is not an API --, and generated code, which a package excludes by
  configuration as it does today.
- **On by default; adoption is by baseline**
  ([ADR-Baselines](ADR-Baselines.md)).

## Forcing constraints / Evidence

A census on 2026-10-22 over the sixteen repositories of
[R025](R025-ADR-PublicOrPrivateRule.md), with a heuristic scanner -- comments
and string literals stripped, struct bodies found by brace matching, `src/`
only, tests, generated code, fixtures and retired code left out -- to be re-run
with this tool's finders before the rule ships. A *literal site* is a place
outside the struct's own file that builds it by literal, `Type { .. }`, or
destructures it: each needs a constructor, a builder or a getter instead.

**1,876 public fields and 72 restricted ones, against 3,061 private**, in **459
structs**, 21 of them carrying restricted fields and 15 of them newtypes. **1,739
literal sites** build or destructure 280 of those structs.

| repository | `pub` fields | restricted | structs | literal sites (types) |
|---|---:|---:|---:|---:|
| `etheram-ibft` | 538 | 9 | 127 | 654 (110) |
| `etheram-ibft-embassy` | 185 | 18 | 53 | 31 (17) |
| `etheram-tools` | 194 | 0 | 28 | 274 (16) |
| `etheram-raft` | 95 | 19 | 38 | 165 (28) |
| `etheram-raft-embassy` | 95 | 11 | 26 | 35 (9) |
| `etheram-embassy` | 85 | 15 | 33 | 43 (17) |
| `stern4rust` | 151 | 0 | 36 | 98 (16) |
| `dry4rust` | 117 | 0 | 28 | 134 (18) |
| `grip4rust` | 94 | 0 | 20 | 51 (10) |
| `crap4rust` | 85 | 0 | 16 | 73 (9) |
| `braintax4rust` | 84 | 0 | 18 | 46 (9) |
| `iceberg4rust` | 59 | 0 | 12 | 46 (8) |
| `slotgate` | 48 | 0 | 10 | 50 (6) |
| `twin4rust` | 32 | 0 | 11 | 39 (7) |
| `etheram-core` | 14 | 0 | 3 | 0 |
| an analysis workspace | **0** | **0** | **0** | **0** |

- **It is livable: an analysis workspace already has it** -- 699 fields in 212
  structs, every one private, read through 405 getters.
- **The restricted fields are all in the protocol and firmware repositories**,
  none in the tools: state shared across files of one crate, as the gRPC
  adapters' is.
- **The cost is construction, not reading.** Of the getters the family already
  has, 725 are named after their field and return it; reading a private field
  is one short method. Building one is where the work is: the 1,739 literal
  sites, most of them in tests building the records they feed in, and wide
  records -- `stern4rust`'s `Config` (17 fields), `etheram-raft`'s
  `RaftContextSnapshot` (12) -- past what one constructor may take.

**Re-run with the rule on 2026-10-23**, read-only, `--rule private-fields` and no cap on the listing: **1,793
non-private fields in 421 structs**. Against the heuristic, every difference is accounted for: this repository's 36
structs and 151 fields, made private before the rule landed, and two structs and four fields `etheram-raft` has
since taken private.

| repository | structs | fields |
|---|---:|---:|
| `etheram-ibft` | 127 | 547 |
| `etheram-ibft-embassy`, with `demo` and `tools/cluster-console` | 53 | 203 |
| `etheram-tools` | 28 | 194 |
| `etheram-raft` | 36 | 110 |
| `etheram-raft-embassy` | 26 | 106 |
| `etheram-embassy` | 33 | 100 |
| `dry4rust` | 28 | 117 |
| `grip4rust` | 20 | 94 |
| `crap4rust` | 16 | 85 |
| `braintax4rust` | 18 | 84 |
| `iceberg4rust` | 12 | 59 |
| `slotgate` | 10 | 48 |
| `twin4rust` | 11 | 32 |
| `etheram-core` | 3 | 14 |
| this repository, an analysis workspace | **0** | **0** |

**What this repository's own adoption showed**, in eight commits: a wide record takes its `Default` and a
`with_<field>` step per field (`Config`, sixteen); a record with no functions gains a mirrored test file once its
getters exist; a getter returning a tuple of two strings is `distinct-tuple-elements`' offence, answered by holding a
named type (`PackageRoster` holds `UnconfiguredRule`); and every plain constructor -- `Self { a, b, c, d }` -- is a
duplicate of every other with as many fields, which dry4rust counted until it learned to leave them out
(ADR-PlainConstructorsAreTheStructsShape, in dry4rust; bridged here by an ignore list until its release).

## Rejected alternatives

**Fields `pub` or private, restricted visibility the only offence** --
[R025](R025-ADR-PublicOrPrivateRule.md)'s two states, for fields. It would take
the 72 restricted fields and leave the 1,876 public ones, which are the same
problem one step wider: any file of any crate writes the state. Rejected.

**Exempting plain records** -- structs with no methods, whose fields are their
contract. The analysis workspace's 212 structs show records with getters work,
and an exemption would make the rule ask, of every struct, whether it is "only
data" -- a question that changes answer with the next method added. Rejected for
one rule with no reading of intent.

**One offence per field.** 1,948 rows where 459 say the same thing, and the
correction is per struct anyway: the struct gains its constructor once.
Rejected.

## Consequences

**A struct owns its state.** An invariant between two fields is the type's to
keep, a representation can change behind its getters, and a lifecycle such as
the gRPC servers' lives in the one type that starts and stops it.

**Construction moves into the type.** Tests build records through constructors
and builders rather than literals; destructuring outside the struct's file
gives way to getters or an `into_parts`. That is most of the cost, and
`etheram-ibft` carries most of it.

**More public functions, so more tests.** Every getter and constructor is
public, and [R013](R013-ADR-TestedPublicApiRule.md) asks a test to call each --
more testing surface, at the price of the tests that pin it.

### What this rule does not catch

- **Shared mutability behind a getter**: a getter returning `&Mutex<T>` or
  `&RefCell<T>` hands out state a caller can change. R028 keeps out `&mut`, not
  interior mutability.
- **Enum variants' fields**, public by the language.
- **A getter for every field**, which makes private fields public in all but
  syntax. Whether a field should be read at all is a design question a syntax
  tree cannot answer.

## Open questions for the owner, answered

1. **Getter names** -- not this rule's, and on 2026-10-23 not any rule's: the owner dropped a getter-naming rule in
   either direction. A getter keeps the family's habit, named after its field.
2. **Builders** -- left to each crate. Where a record is past seven fields, this repository uses a `Default` and
   `with_<field>(self, value) -> Self` steps, the shape the family's experiment plans take as their default.

## Building it

Decided by the owner on 2026-10-23: the rule ships in the coming release and is enforced on the tools and on
`etheram-ibft`. The rule's own gate runs the development build over this repository first, so this repository goes
first, in commits that change no behaviour, and the rule lands on a tree it finds clean:

1. **Landed** (`7bb2007`..`1be25a3`). **This repository's 36 structs made private**, a module group a commit -- adoption, the finding models, the
   layout models, reporting, the settings, `Config` alone, then `xtask` -- each getter called by a test before it
   exists, and every literal site moved to a constructor or a `with_` step.
2. **Landed.** **The rule**, test-first, as *Enforcement* below says, with `RULES.md`, the README, the ADR index,
   `IMPLEMENTED-FEATURES.md`, the roadmap, the CLAUDE.md count and the changelog.
3. **Landed.** **The census re-run with the rule** over the sixteen repositories, read-only, and recorded here.
4. **The tools, `slotgate` and `etheram-core`**, a repository at a time and a module group a commit, each through its
   own gates and the development build. `etheram-ibft` takes the rule in its own overhaul, not here.

## Enforcement

Written test-first (`tests/rules/types/private_fields_rule_tests.rs`, twenty tests): a `pub`, a `pub(crate)`, a
`pub(super)` and a `pub(in ...)` field each reported at the struct, the fields named -- one, two, three in full, four
or more as the first two and a count -- and the correction in the singular or the plural; a newtype's field reported
by position; a struct in an inline module reported at its own line; a struct of private fields, an enum's variant
fields, a unit struct, `tests/`, a `#[cfg(test)]` module and an unparseable file left alone; `check_workspace` empty;
the explanation, `is_configured`, the name and the requirement. `tests/rule_registry_tests.rs` lists the rule, and
the runner's listing counts thirty-five.

The rule lives in `rules/types/`, the first of a group for how a type holds its state: `rules/source/` holds twenty
rules, `directory-file-count`'s limit.

## Related

- [R025-ADR-PublicOrPrivateRule](R025-ADR-PublicOrPrivateRule.md) -- the two
  honest states, for methods.
- [R028-ADR-ImmutableReturnsRule](R028-ADR-ImmutableReturnsRule.md) -- why a
  getter cannot hand out a field to change.
- [R013-ADR-TestedPublicApiRule](R013-ADR-TestedPublicApiRule.md) -- the tests
  every getter and constructor will ask for.
