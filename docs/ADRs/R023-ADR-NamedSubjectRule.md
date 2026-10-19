# R023-ADR-NamedSubjectRule

- **Status:** Accepted
- **Date:** 2026-10-19

## Context

[R007](R007-ADR-SingleImplementedTypeRule.md) bounds a file to one subject, and
since 0.15.0 it knows which type the file is named after when it reports a second
one. It never asks whether the file is named after its subject at all. A file can
hold one implemented type and be named after something else entirely, and pass.

The sharp case was recorded as an open point. `etheram-ibft`'s `wal_record.rs`
holds `WalRecord`, a plain enum with no impl, beside `WalEntryReader`, which
carries the whole parsing impl. One implemented type, so R007 is satisfied, yet
the type the file is named for does nothing and the work is done by a type the
file name does not mention. The header, the module registry and the file name all
point a reader at the wrong type.

The milder cases are drift: `listener.rs` holding `UdpListenerTask`,
`cluster_query_provider.rs` holding the trait `ClusterStateProvider`. Each is
the shape a reviewer would mention by hand and that stops being true across a
codebase as soon as nobody checks.

## Decision

**A source file is named after its subject.**

- **Candidates.** The types the file implements, as R007 counts them -- a struct
  or enum declared in the file with at least one impl block there. With none, every
  trait and type the file declares, traits first, because a trait declares
  behaviour and a data type does not. A trait beside an implemented type is that
  type's companion and is not a candidate.
- **Named after.** The file's stem is the type's name, underscores and case
  aside, or the type's last whole words. `loopback/src/driver.rs` holding
  `LoopbackDriver` is named after it, and so is `listener.rs` holding
  `UdpEiListener`. Words are split at capitals, with a run of capitals one word,
  so `http_server.rs` is named after `HTTPServer`. Only the front may be left out:
  `protocol_observer.rs` holding `Observer` claims more than the type says.
- **One offence per file**, against the first candidate's declaration, listing
  every candidate, and the correction names the file the first would live in:
  *"`listener.rs` is named after none of the types it implements:
  `UdpListenerTask`"* -- *"rename the file to udp_listener_task.rs, or rename
  `UdpListenerTask` after the file"*.
- **Exempt.** `lib.rs`, `main.rs`, `mod.rs` and `build.rs`, named for their role in
  the package, and `tests/`, `benches/`, `examples/` and `src/bin/`, named for what
  they exercise or run. A file of functions and constants alone has nothing to be
  named after.

A file with two implemented types, one of them named for it, is R007's: it
reports the other, and this rule is silent. To keep the two rules agreeing on
which type a file is about, R007 now reads "named after" through the same
`FileStem::names`. The words come from `TypeName`, which also fixes R007's
suggested file for an acronym in capitals.

## Forcing constraints / Evidence

Measured before the rule was written, with a prototype built on
`ImplementedTypeFinder`, on the sixteen repositories in the family, each scanned
the way its own stern gate scans it. The prototype's count of two-subject files
matched every R007 baseline exactly. The shipped rule, run read-only on the same
trees, reports the same 57 offences, repository by repository.

| reading | offences |
|---|---:|
| strict: the stem is the type's name | 78 |
| strict, plus directories inside `src/` as a prefix | 77 |
| the type's last words (this rule, without data-only files) | 46 |
| **this rule** | **57** |
| the first type in the file | 121 |

| repository | offences |
|---|---:|
| `etheram-embassy` | 23 |
| `etheram-ibft` | 8 |
| `dry4rust` | 8 |
| `etheram-raft` | 7 |
| `etheram-ibft-embassy` | 5 |
| `etheram-raft-embassy` | 4 |
| `crap4rust` | 1 |
| `etheram-tools` | 1 |
| `stern4rust`, `etheram-core`, `beggar`, `braintax4rust`, `grip4rust`, `iceberg4rust`, `slotgate`, `twin4rust` | **0** |

By hand, the 57 fall into three shapes: 28 file and type names that differ by a
word or have drifted apart, fixed by a rename; 13 helper types in a file named
after its functions, such as `ThreadWaker` in `block_on.rs`, fixed by moving the
type out; and the rest data-only files or a behaving type beside a named data
type, the `wal_record.rs` case, fixed the way R007's offences are.

## Rejected alternatives

**The strict match alone.** Rejected: of its 78, 32 are files named after the
type's last words -- `driver.rs` holding `LoopbackDriver`, `state_machine.rs`
holding the trait `RaftStateMachine` -- where the crate supplies the qualifier
and the name reads right where it stands.

**Directories inside `src/` as the qualifier.** Rejected on the evidence: it
cleared one file in 78. In this family the qualifier comes from the crate, not
from a directory, and modelling the package path costs more than it buys.

**The type's name as the last words of the stem.** Rejected: `protocol_observer.rs`
holding `Observer` names a qualifier the type does not carry, so the file claims
more than it holds.

**Matching letters rather than words.** Rejected: `dex.rs` would be named after
`Index`. On the family's trees the two readings agree file for file; the rule has
to be right on the trees that are not measured.

**The first type in the file.** Rejected: declaration order is a style choice,
and the types a subject serves -- its config, its result, its error -- are often
declared above it. It reported 121, among them `default_scorer.rs`, asked to be
renamed after `BraintaxComponents`, and the six files whose named subject is
declared second, which is the defect 0.15.0 fixed in R007.

**Leaving data-only files unchecked.** Rejected: 273 files hold types and no
impl, 262 of them already comply, and the other 11 include two `variants.rs`
files of fourteen enums each, named after none of them -- the shape the rule is
for.

## Consequences

**A misnamed file fails the build.** On by default, with a correction naming the
file to rename to. Fifty-seven offences across eight repositories in the family
on the day it shipped; how each adopts it -- fixing, baselining, or skipping it
in `stern4rust.toml` first -- is decided repository by repository.

**R007 and this rule agree.** A file is about the type it is named after in the
same sense for both, so R007 never asks to move the type this rule accepts.

### What this rule does not catch

- **Whether a name is a good one.** `data.rs` holding `MetaData` is named after
  it.
- **The directories.** Only the stem is read: `udp/listener.rs` holding
  `TcpListener` is named after it.
- **A file of functions and constants**, which has no type to be named after.
- **A trait beside an implemented type**, by decision, above.
- **Macros.** `syn` does not descend into macro token streams, so a type declared
  by a macro is invisible, as everywhere else in this tool.

## Enforcement

`tests/rules/source/named_subject_rule_tests.rs` -- nineteen tests covering the
implemented type named and not named, the last words, the type named but declared
second, plain data beside an implemented type, a trait file named and not named,
data-only files named and not named, several implemented types reported once, a
trait beside an implemented type, the reserved names, the exempt roots, a file of
functions alone, the unparseable file, `check_workspace`, `is_configured`, the
name and the requirement.

`tests/finding/model/file_stem_tests.rs`, `type_name_tests.rs`,
`declared_item_tests.rs` and `tests/finding/parsing/declared_item_finder_tests.rs`
pin the match, the word split and the declarations on their own.

`tests/rule_registry_tests.rs` -- the four hardcoded rule-name lists include
`named-subject`.

`tests/runner_tests.rs::run_against_this_crate_with_its_own_header_is_clean` and
stage 2 both run the tool against this crate and require zero offences.

## Related

- [R007-ADR-SingleImplementedTypeRule](R007-ADR-SingleImplementedTypeRule.md) --
  one subject per file; this rule names it.
- [R016-ADR-PairedTestFileRule](R016-ADR-PairedTestFileRule.md) -- a mirrored
  test file says something only when its source file is named after what it
  holds.
