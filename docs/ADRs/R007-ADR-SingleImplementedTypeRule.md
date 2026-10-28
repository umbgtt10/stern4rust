# R007-ADR-SingleImplementedTypeRule

- **Status:** Accepted; amended 2026-10-25 (below)
- **Date:** 2026-08-19

## Context

A file is named after what it holds. That only works while it holds one thing.

The second type with behaviour is where it stops working, and it never arrives
announced: a small helper struct gets an `impl` because it needed one method,
and now `report_printer.rs` contains both a report printer and a column-width
calculator. The name still describes the file to whoever wrote it and no longer
describes it to anyone else. A reader opening it for one subject steps over the
other on the way.

It also decides whether a mirrored test file means anything. `src/foo.rs`
answering to `tests/foo_tests.rs` — the pairing `twin4rust` enforces — only says
something when `foo.rs` has one subject to test.

Plain data is a different case and must not be caught by the same net. A file
whose subject needs four payload structs with no behaviour holds one subject and
four descriptions of its inputs, which is exactly the shape that keeps the
subject readable.

## Decision

A source file outside `tests/` holds at most one type that carries behaviour: at
most one `struct` or `enum` that is both **declared in the file** and has **at
least one `impl` block** in it. Structs and enums without `impl` blocks are
unlimited.

Both halves of that conjunction do work.

**Declared here.** An `impl Display for SomeoneElsesType` does not make this
file that type's home, so it is not one of this file's subjects. A file may
implement foreign traits for foreign types without becoming their owner.

**At least one impl block**, inherent or trait. Both are behaviour, and a reader
looking for what a type does opens the file either way. `#[derive(...)]` is not
an impl block in the syntax tree and correctly does not count — a derived
`Debug` is not the file's subject.

The first implemented type is taken as the subject and every later one is
reported, so the offence names the type to move rather than announcing that the
file has too many. *(Amended 2026-10-19, below: the subject is the type the file
is named after.)* The correction names the file it would live in, PascalCase
mapped to snake_case: *"move `ColumnWidths` and its impl blocks into
column_widths.rs"*.

The walk descends into inline modules. A nested type with behaviour is still a
second subject in the same file.

`tests/` is exempt, and not as a concession: a test file legitimately holds
several fakes that each carry an impl block, and that is the shape
[R002](R002-ADR-TestFileStructureRule.md) asks for rather than something to be
split up.

## Forcing constraints / Evidence

Measured across eight repositories before the rule was written, exactly one file
breaks it — and it is this tool's own `report_printer.rs` (now under
`src/reporting/`), holding both
`ReportPrinter` and `ColumnWidths`. Every other repository in the family, and
`etheram-core`, was already clean.

That is the evidence for the rule being right rather than merely defensible. It
is not a new standard being imposed on eight codebases; it is a standard seven
of them already keep, written down and enforced, with the eighth being the one
that wrote it. `ColumnWidths` now lives in `column_widths.rs` with three tests
of its own that it never had while it was a private struct in somebody else's
file — which is the second thing this rule buys: an extracted subject gets a
mirrored test file, and `twin4rust` starts asking for one.

## Rejected alternatives

**Count any type with an `impl`, declared here or not.** Rejected: a file
implementing a foreign trait for a foreign type would become that type's home by
accident, and a single `impl Serialize for RemoteThing` beside a real subject
would be an offence with nowhere sensible to move to.

**Count only inherent impls, letting trait impls pass.** Rejected: a `Display`
impl is behaviour, and a type with only trait impls is still a subject. It would
also create an odd incentive to hide a second subject behind a trait.

**Count only `pub` types.** Rejected: `ColumnWidths` was private, and it was
still the second subject of its file. Visibility is about who may use a type,
not about whether the file has two things to be named after.

**Limit the file to one type outright.** Rejected: it would forbid the payload
structs a subject legitimately needs, and would push a file's own input and
output types somewhere less useful than beside it. The user's framing was
explicit that data is fine, and it is right.

**Ignore inline modules.** Rejected as an easy simplification that would leave
an obvious escape hatch: wrapping the second subject in `mod detail { ... }`
would silence the rule without changing anything a reader cares about.

## Consequences

**An extracted type gains a mirrored test file obligation.** Moving
`ColumnWidths` into its own file made `twin4rust` require its mirrored test
file, now `tests/reporting/column_widths_tests.rs`, which is a cost — and a fair
one, since the alternative was a piece of logic with no direct tests at all.

**A one-method helper now costs a file.** That is the intended trade and it will
occasionally feel heavy for a struct with a single `fn`. The rule's answer is
that a struct with a single `fn` and no file of its own is exactly how a second
subject sneaks in.

**What this rule does not catch.** It counts types, not size: a single type with
two thousand lines and forty methods satisfies it completely, and `crap4rust`
owns that question. It says nothing about free functions — a file with one
subject and a dozen unrelated `pub fn`s passes. And a `trait` with a default-
method body is behaviour that this rule does not count as a subject, because a
trait is a contract rather than a thing the file is named after; if that turns
out to be wrong it will be wrong visibly, in a file whose name stops matching.

## Enforcement

`tests/rules/source/single_implemented_type_rule_tests.rs` — 11 tests covering one
subject, one subject with plain data beside it, two subjects, an enum as the
second subject, a file whose named subject is declared second, one named after
by its last words, the `tests/` exemption, and that every type after the first is reported rather than only the
second.

`tests/finding/parsing/implemented_type_finder_tests.rs` — 10 tests pinning the two halves of
the conjunction independently: a trait impl counts, several impls of one type
count once, a type without an impl does not count, an impl for a type declared
elsewhere does not count, and the walk descends into inline modules.

`just stage2` runs the compiled binary against this crate's own tree, which
is what caught `report_printer.rs` the moment the rule was registered.

## Amendment, 2026-10-19: the subject is the type the file is named after

Taking the first implemented type as the subject reported the wrong one whenever a
companion was declared above it. `etheram-ibft-embassy`'s `wire_client_request.rs`
declares `WireTransaction`, with its two `From` impls, above `WireClientRequest`,
with its own: the rule said `WireClientRequest` was the second type and asked for
it to move into `wire_client_request.rs` -- the file it was already in -- while the
companion that should move went unnamed.

The subject is now the implemented type whose name, mapped to snake_case, is the
file's name, wherever it stands in the file, and every other implemented type is
reported. A file with no type named for it keeps the old reading: the first is the
subject. The count is unchanged -- the rule reports exactly as many offences as
before, on the same files -- but where the named subject was declared second, a
baselined offence moves from it to the companion. Since 0.15.0.

The narrower gap the count still misses -- a named subject that carries no impl at
all, its behaviour living in a companion -- stays open (`docs/OPEN_POINTS.md`).

## Amendment, 2026-10-19: named after, as named-subject reads it

[R023](R023-ADR-NamedSubjectRule.md) holds a file to the name of its subject,
and reads "named after" more widely than this rule did: the stem is the type's
name or its last words, so `listener.rs` is named after `UdpListener`. With two
readings, the two rules could disagree about one file -- named-subject satisfied
by `UdpListener`, this rule taking a companion declared above it as the subject
and asking for `UdpListener` to move. The subject is now found through the same
`FileStem::names`, so the two always agree on which type a file is about. No
baselined offence in the family moves beyond what 0.15.0 moved.

The suggested file now comes from `TypeName`, whose words keep an acronym in
capitals together: `HTTPServer` belongs in `http_server.rs`, where the old
mapping offered `h_t_t_p_server.rs`. The correction is not part of an offence's
fingerprint, so no baseline changes.

R023 also closes the gap above: a file named after a data type, with its
behaviour in a companion, is named after none of the types it implements. Since
0.16.0.

## Amendment, 2026-10-25: a subject is an object

**A subject is a struct or an enum declared in the file that has behaviour**, by
[R035](R035-ADR-RecordsAndObjectsRule.md)'s definition: an inherent method other than a getter, a plain constructor or
a `with_` step, or an impl of a trait outside the value traits. A record or a data enum whose only impls are value
traits -- `Display`, `From`, `Default` and the rest of R035's list -- is not a subject, however many of them it has.

[R037](R037-ADR-RecordHomeRule.md) puts a record with one consumer inside that consumer's file. With a `Display` impl,
the record would be the file's second implemented type under the reading above, and the two rules would contradict
each other. The classifier is R035's, shared, so every rule reads a type the same way. The correction and the
choice of subject -- the type the file is named after -- are unchanged.

## Related

- [R002-ADR-TestFileStructureRule](R002-ADR-TestFileStructureRule.md) — why
  `tests/` is exempt: several fakes with impl blocks is the shape that rule
  asks for.
- [R006-ADR-ModuleRegistryRule](R006-ADR-ModuleRegistryRule.md) — the other rule
  about what a source file may hold, one level up at the registry.
