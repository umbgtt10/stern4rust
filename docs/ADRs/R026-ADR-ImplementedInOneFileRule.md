# R026-ADR-ImplementedInOneFileRule

- **Status:** Accepted
- **Date:** 2026-10-20

## Context

[R007](R007-ADR-SingleImplementedTypeRule.md) bounds a file to one implemented
type, and [R023](R023-ADR-NamedSubjectRule.md) names the file after it. Nothing
bounds a type to one file. Rust accepts an inherent `impl` block for a type in
any file of its crate, so a type that grew can be continued file after file --
`impl RecoveryControl` in `timer_handling.rs`, `message_handling.rs`,
`query_handling.rs`, `verdict_handling.rs` and `request_issuing.rs` -- each file
passing R007 and R023, and the type as a whole a subject no file names.

The cost shows up as visibility. Rust's privacy is per module, so a private `fn`
in one of those files cannot be called from the others, and every function the
files share has to be `pub(crate)` or `pub(super)`: the third state
[R025](R025-ADR-PublicOrPrivateRule.md) removes. A spread impl is where most of
that state comes from, and no visibility rule can be satisfied honestly while
the spread remains.

## Decision

**A type's inherent impl lives in one file.** Every inherent `impl` block for a
type sits in the same file of its crate; a type with inherent impl blocks in two
or more files is an offence.

- **A type** is identified within its crate by the last segment of its path, the
  way R007 reads it: `impl<T> Store<T>`, `impl Store<u8>` and
  `impl crate::store::Store` are all `Store`. The tool resolves no types, so this
  is the syntax tree's view, below.
- **A name declared in two or more files of the crate** is read as
  alternatives -- `cfg`-selected platform types, or unrelated types that share
  a name -- and each declaration's impls must sit in its own file. Distinct
  types are not merged into one offence.
- **Trait impls are left alone.** `impl Display for Store` beside the trait or
  in an implementation area is the house pattern for a seam, not a spread type.
- **One offence per type**, at its declaration, naming every file that holds an
  inherent impl of it: *"`RecoveryControl` is implemented in 6 files: ..."*. A
  type with no declaration in the package -- an alias, a macro's -- is reported
  at its first impl. Alternatives are reported only when an impl sits in a file
  declaring none of them: *"`Flash` is declared in 2 files and implemented in 1
  that declares neither: ..."*.
- **The crate is the package** the run walks, all of its `src/` read at once.
- **The correction reads the impls:**
  - impls in two or more files a `#[cfg]` selects, one per platform: *"put the
    contract in a trait in `traits/` and give each platform its own implementing
    type"*. A file is selected by a `#[cfg]` on its `mod` declaration or one
    above it -- where the gate sits in practice, on the declaration in the
    registry, the selected file carrying nothing to say so -- by `#![cfg(...)]`
    at its top, or by a `#[cfg]` on the impl itself. `cfg_attr` selects nothing;
  - otherwise: *"move every `impl RecoveryControl` into
    `process/recovery_control.rs`, or split `RecoveryControl` into types of their
    own"*. The file offered is the one declaring the type. Which of the two
    applies is a judgement the tool cannot make, so both are named, the split
    second because it is the larger change.
- **Exempt:** `tests/`, whose fakes are this rule's business no more than R007's.
- **On by default.** It reports three types in two repositories, below.

## Forcing constraints / Evidence

A census on 2026-10-20 over the sixteen repositories of
[R025](R025-ADR-PublicOrPrivateRule.md), with the same heuristic scanner and the
same caveat: re-run it with this tool's finders before the rule ships. It found
1,518 types with inherent impls under `src/`, and three implemented in more than
one file.

| type | crate | files | restricted functions inside |
|---|---|---:|---:|
| `IbftProtocol` | `etheram-ibft` `node` | 13 | 12 |
| `RecoveryControl` | `etheram-ibft` `recovery/control` | 6 | 18 |
| `PlatformFlashAccess` | `etheram-embassy` `infra/storage/flash` | 5, one per platform | 8 |

Three more names are declared in two files each and implemented beside each
declaration -- `NodeInfraSlot` in `etheram-raft-embassy`, and `Processor` and
`Operation` in two `grip4rust` fixture crates -- and pass as alternatives.

**The census's first run is the reason for the cfg and alternatives clauses.** A
scanner that took every `impl` keyword reported thirteen spread "types" --
`Result`, `Future`, `Iterator`, `IntoIterator`, `Binding` and `CliResult` among
them -- all of them `-> impl Trait` in a return position or a name two files
each declared. A rule that cannot tell those apart accuses correct code.

The 38 restricted functions inside the three types are 30% of R025's 125, and
the ones R025 cannot clear by itself.

**Re-run with this tool's finders**, the built rule against the sixteen
repositories on 2026-10-21: **four types in three repositories**.

| type | repository | files | correction |
|---|---|---:|---|
| `IbftProtocol` | `etheram-ibft` | 13 | the declaring file, or a split |
| `RecoveryControl` | `etheram-ibft` | 6 | the declaring file, or a split |
| `PlatformFlashAccess` | `etheram-embassy` | 5 | a trait per platform: four of the files sit behind `#[cfg]` module declarations |
| `NormalizationContext` | `dry4rust` | 5 | the declaring file, or a split |

The heuristic missed the fourth. `NormalizationContext`'s five files are the
shape `dry4rust`'s adoption of [R024](R024-ADR-OwnedFunctionsRule.md) left the
day before: the normaliser's free functions became the context's associated
functions, each staying in its own file. The alternatives the census found --
`NodeInfraSlot`, and `Processor` and `Operation` in `grip4rust`'s fixtures -- pass
here too. `etheram-core`, `etheram-ibft-embassy`, `etheram-raft`,
`etheram-raft-embassy`, `etheram-tools`, the analysis workspace and the other
`*4rust` tools, this one among them, have none.

## Rejected alternatives

**Requiring the impl in the declaring file.** Stricter, and R023's naming would
then hold for the whole type. Rejected for now because it was not measured: the
census counted files holding impls, not their distance from the declaration.
Measure it, then decide.

**Exempting `cfg`-selected impls of one type.** Rejected: that shape forces
eight `pub(crate)` functions in `PlatformFlashAccess`, and the house standard
already names the alternative -- a contract in `traits/`, an implementation per
platform.

**Folding this into R007.** Rejected: R007 judges a file on its own, and this
rule needs every file of a crate at once, as
[R009](R009-ADR-RegistryCompletenessRule.md) does. Different scans, different
offences, different corrections.

## Consequences

**A type is one file's subject in both directions.** R007 keeps a file to one
type, this rule keeps a type to one file, and R025 then makes `fn` mean private
to that type.

**Four types have to move or split.** In `dry4rust`, `NormalizationContext`'s
methods share one mutable placeholder map and recurse into each other --
expressions hold patterns and patterns hold expressions -- so splitting it into
types would hand the map between them as a `&mut` parameter, which
[R022](R022-ADR-ReturnedMutationsRule.md) forbids; its fix is the move into the
declaring file. `PlatformFlashAccess` becomes a trait
with a type per platform. `RecoveryControl` and `IbftProtocol` are protocol
types continued across six and thirteen files; moving them back into one file
would make files of thousands of lines -- `IbftProtocol`'s `consensus.rs` alone
runs past 1,300 -- so the correction they need is the split: their handler
groups as types of their own. That is design work in `etheram-ibft`, and the 30 restricted
functions inside them go with it.

### What this rule does not catch

- **A type implemented in one file and declared in another.** Allowed, by the
  first rejected alternative.
- **A type reached through an alias** (`type Store = Inner; impl Store`), which
  the syntax tree sees as a second name.
- **Impls generated by a macro**, invisible to `syn`.

## Enforcement

`tests/rules/source/implemented_in_one_file_rule_tests.rs` -- fourteen tests: a
type implemented in one file left alone and in two reported at its declaration;
every implementing file named, generic, concrete and path-qualified impls as one
type; the correction offering the declaring file, a trait when the impls are
split by platform, one file when the type is declared nowhere, and the
declarations when alternatives have an impl astray; alternatives beside their
declarations, a trait impl elsewhere and `tests/` left alone; `check` answering
nothing per file; `is_configured`, the name and the requirement.

`tests/finding/parsing/type_homes_tests.rs` (twenty-one) pins the map the rule
reads: what is counted -- an impl inside an inline module, not a trait impl, not
`-> impl Trait`, not a file outside `src/` or one that does not parse -- which
files declare and implement a type, the ones outside its declarations, where an
offence is anchored, and a split by platform from gated modules, from gated impls
and not from one gated file. `tests/finding/parsing/modules/gated_modules_tests.rs` (ten)
pins which files a `#[cfg]` selects: by a `mod` declaration in a crate root, a
nested registry or a named file, everything inside a gated module, a file's own
`#![cfg]`, and not a module whose name a gated one begins, nor through
`cfg_attr`.

`tests/rule_registry_tests.rs`' four rule-name lists include
`implemented-in-one-file`, and `tests/runner_tests.rs`' `--rules` listing holds
twenty-eight. Stage 2 runs this tool, built from the checkout, against this
crate with every rule applied and zero offences.

## Related

- [R007-ADR-SingleImplementedTypeRule](R007-ADR-SingleImplementedTypeRule.md) --
  the converse: one implemented type per file.
- [R025-ADR-PublicOrPrivateRule](R025-ADR-PublicOrPrivateRule.md) -- the rule this
  one makes satisfiable: with the type in one file, a private `fn` reaches all of
  it.
- [R009-ADR-RegistryCompletenessRule](R009-ADR-RegistryCompletenessRule.md) --
  the other rule that reads a crate's files together.
