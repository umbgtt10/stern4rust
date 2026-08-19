# R010-ADR-DirectoryFileCountRule

- **Status:** Accepted
- **Date:** 2026-08-19

## Context

A directory is a list a reader scans. Past some length it stops being a list and
becomes a wall, and the reader stops reading it and starts using search instead
— at which point the layout has told them nothing.

This tool's own `src/` holds 42 files.

The awkward part is that this is a rule about the consequences of the other
rules. One struct per file, one implemented type per file
([R007](R007-ADR-SingleImplementedTypeRule.md)), one test file per source file —
those conventions *manufacture* files by design, and they are right to. The
question is not whether to have many small files but whether to leave them all in
one heap.

## Decision

A directory holds at most **20** `.rs` files, not counting its own index.

**The limit is configuration, not a constant.** This is the only rule whose
number is a matter of taste rather than a fact — every other rule states
something that is either true or false about a file. `max-files-per-directory`
in `stern4rust.toml` sets it; 20 is the default. Somebody else's 20 is 30, and a
rule that pretended otherwise would be ignored rather than adjusted.

**20 rather than the 12–15 first proposed.** Measured across eight repositories,
12 puts all eight over the line and 15 puts six over. More importantly, a limit
that tight fights the conventions above: a subject with four payload types and a
mirrored test file is five files before anything else, and a limit punishing its
own standards gets worked around rather than kept. 20 is where the family's
directories actually separate — five repositories over, three comfortably under.

**Registries do not count.** A `mod.rs`, `lib.rs` or `all_tests.rs` is an index
*of* the directory rather than something *in* it, and counting the list against
the length of the list makes no sense. **`main.rs` does count**: it is an entry
point holding real code. That makes this list deliberately shorter than the one
[`PackageTree`](../../src/finding/package_tree.rs) uses to decide what may
declare a module, where `main.rs` belongs because it can.

At the measured threshold the exemption changes nothing — every offending
directory is 22 files or more, so one index never pushes one under the line. It
is decided on principle rather than on effect.

**Reported against the directory's index** where it has one, because that is the
file a split has to edit anyway: the new subfolders need `pub mod` lines there.

## Forcing constraints / Evidence

Ten offences across five of eight repositories:

```
stern4rust     src     42 -> 2 splits      grip4rust  tests 37 -> 1
stern4rust     tests   39 -> 1 split       grip4rust  src   30 -> 1
crap4rust      src     28 -> 1             crap4rust  tests 28 -> 1
braintax4rust  tests   24 -> 1             iceberg4rust src 23 -> 1
braintax4rust  src     23 -> 1             iceberg4rust tests 22 -> 1
```

`twin4rust`, `slotgate` and `etheram-core` are clean.

**The symmetry is exact and it is not a coincidence.** Every repository over the
line is over it on *both* sides — 42 ↔ 39, 30 ↔ 37, 28 ↔ 28, 23 ↔ 24, 23 ↔ 22 —
because `twin4rust` requires a mirrored test file per source file. Ten offences
are really five restructurings done twice.

## Consequences

**This is the first rule that cannot be satisfied by editing a file.** Every
other offence is fixed in place. This one needs `git mv`, a new `mod.rs` per
folder (which [R006](R006-ADR-ModuleRegistryRule.md) then governs), a `pub mod`
line in the parent (which [R009](R009-ADR-RegistryCompletenessRule.md) then
checks), and a matching move under `tests/`. `--fix` cannot help; moving files is
not a text rewrite.

**It does not say where to split.** A cap forces a division and is silent about
which one, and an arbitrary split — `a_to_m/`, `n_to_z/` — would be worse than
the flat directory it replaced. The rule can only say that 42 is too many; a
person has to say what the groups are. That is a real limitation and the reason
this rule is more of an invitation than an instruction.

**It counts files, not size.** Twenty files of two thousand lines each satisfy
it completely, which is `crap4rust`'s question rather than this one's.

**Non-`.rs` files are invisible**, because the walker only collects `.rs`. A
directory of forty `.json` fixtures beside four sources counts as four.

## Rejected alternatives

**12–15, as first proposed.** Rejected on measurement: it puts every repository
in the family over the line, and it fights the file-per-type conventions this
tool enforces elsewhere.

**A hardcoded constant.** Rejected: the number is taste. Every other rule's
threshold is a fact about Rust or about a stated standard; this one is a
judgement about how long a list can be.

**Counting registries.** Rejected on principle — an index is not content — though
it changes no count at 20.

**Exempting `tests/`.** Rejected: a 39-file test directory is exactly as
unscannable as a 42-file source directory, and the mirroring rule means the two
move together anyway.

## Applying it to this repository

The rule's first act was to report this tool's own `src/` at 42 files and
`tests/` at 39. Both were restructured rather than baselined, keeping the pattern
that every rule so far has been satisfied by the repository that wrote it:

```
src/               8 files, 5 subfolders   entry point, the Rule seam, the walk
├── finding/      15   parsers, finders, and the items they produce
├── reporting/     8   printers, offence, threshold, output formats
├── rules/        11   one file per rule
├── settings/      6   args, config, config file, manifest, selection
└── adoption/      5   baseline and exclusion -- the ways in
```

Five subfolders is exactly R011's limit, which is a fair warning that the two
rules together leave less room than they appear to.

`settings/` rather than `config/` because the folder would then hold
`config.rs`, and `crate::config::config::Config` is a path nobody should have to
read. That is the sort of decision a cap forces and cannot make for you.

`tests/` mirrors it file for file, as `twin4rust` requires — 33 moves on each
side, eight new registries, and every `use` path rewritten. The compiler and the
440-odd tests were the check that nothing was lost.

## Enforcement

`tests/rules/directory_file_count_rule_tests.rs` — 9 tests covering the
boundary, over the boundary, per-directory counting, the registry exemption for
all three index names, `main.rs` counting, reporting against the index, and
reporting against the path where there is no index.

## Related

- [R011-ADR-DirectorySubfolderCountRule](R011-ADR-DirectorySubfolderCountRule.md)
  — the counterweight, so that splitting is not the answer to everything.
- [R009-ADR-RegistryCompletenessRule](R009-ADR-RegistryCompletenessRule.md) — one
  of the three rules a split then has to satisfy.
