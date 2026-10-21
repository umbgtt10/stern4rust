# R031-ADR-LeadingConstantsRule

- **Status:** Accepted
- **Date:** 2026-10-20

## Context

A test file already reads in one order: header, imports, constants, helpers,
tests ([R002](R002-ADR-TestFileStructureRule.md)). A source file has no order
at all for its constants. The common shape in the family is a type, then a
constant, then the impl that uses it -- the constant parked beside its first
reader -- and a reader looking for a file's numbers has to read the whole file
to find them.

Module-level items are order-independent in Rust: a constant can be read before
its declaration, a type can name a constant declared below it. So where a
constant sits is pure layout, which makes it a placement a tool can enforce and
repair, as [R027](R027-ADR-OrderedImplFunctionsRule.md) does inside an impl.

## Decision

**A source file reads header, imports, constants, then everything else.** In
`src/`, a module-level `const` or `static` declared after any other item -- a
struct, enum, trait, impl, function, type alias, union or macro -- is an offence.

- **The constants group** is every module-level `const` and `static`, in the
  order written. Sorting them is not asked: it was not measured, and a
  constant's place in the group is a smaller question than the group's place in
  the file.
- **Imports, `extern crate` and `mod` declarations come first** and are not
  judged here; their order is [R019](R019-ADR-OrderedImportsRule.md)'s and
  `rustfmt`'s.
- **Inline modules are not descended into.** A `mod x { ... }` is one item of
  the file; what it holds is a second file's worth of layout.
- **Exempt:** registries (`lib.rs`, `mod.rs`), which
  [R006](R006-ADR-ModuleRegistryRule.md) already keeps free of constants, and
  `tests/`, which is R002's.
- **One offence per file**, at the first constant out of place:
  *"`LIMIT` is declared after `Widget`; constants come straight after the
  imports"*.

### `--fix`

`--fix` moves every out-of-place constant to the end of the constants group --
straight after the imports when there is none -- in a package that applies this
rule, under the constraints of [ADR-FixOnlyWhatIsSafe](ADR-FixOnlyWhatIsSafe.md):

- **Whole line ranges from `syn` spans.** An item travels with its attributes,
  doc comments and the comment lines directly above it.
- **Nothing else moves.** Every other item keeps its place and its bytes; blank
  lines around the constants group become one.
- **A file it cannot repair without guessing is declined** and reported
  unchanged, its offence's correction ending *"by hand: `--fix` leaves alone"*
  and the reason:
  - a `macro_rules!` in the file, where textual order decides what a later item
    can call;
  - a misplaced constant sharing a line with another item, or the item the
    constants follow sharing one with the item above it, either of which would
    be split by a move made of lines;
  - nothing above the first item and a comment running from the top of the file
    into it, where the header cannot be told from that item's own comment.

  The proposal had the declined file named in the fix summary. The summary is a
  count, and the offence is where a file is already named, so the reason went
  there instead: a run with `--fix` and one without report it identically, as
  ADR-FixOnlyWhatIsSafe asks. The leader's shared line and the opening comment
  were found building the fixer; the census below met none of the three.

## Forcing constraints / Evidence

A census on 2026-10-20 over sixteen repositories -- the `etheram-*` six, the
seven `*4rust` tools, `slotgate`, `etheram-tools` and an analysis workspace --
with a heuristic scanner validated on a labelled crate, to be re-run with this
tool's finders before the rule ships. **491** source files declare module-level
constants or statics; **137** declare one after another item.

| repository | files with constants | out of place |
|---|---:|---:|
| an analysis workspace | 83 | 74 |
| `etheram-embassy` | 127 | 25 |
| `etheram-ibft-embassy` | 80 | 21 |
| `etheram-raft-embassy` | 37 | 9 |
| `etheram-tools` | 26 | 3 |
| `etheram-raft` | 19 | 2 |
| `etheram-ibft`, `braintax4rust`, `dry4rust` | 36, 11, 10 | 1 each |
| `etheram-core`, `crap4rust`, `grip4rust`, `iceberg4rust`, `slotgate`, `stern4rust`, `twin4rust` | | **0** |

No out-of-place file holds a `macro_rules!`. How many put a constant on one line
with another item was not measured -- a line count is defeated by the `;` in an
array type -- and is left to this tool's spans.

**Re-run with this tool's finders**, the built rule against the sixteen
repositories on 2026-10-21: **134 offences**, against the census's 137, and **not
one declined** -- no `macro_rules!`, no shared line, no opening comment running
into a first item.

| repository | offences |
|---|---:|
| an analysis workspace | 74 |
| `etheram-embassy` | 23 |
| `etheram-ibft-embassy` | 20 |
| `etheram-raft-embassy` | 9 |
| `etheram-tools` | 3 |
| `etheram-raft` | 2 |
| `etheram-ibft`, `braintax4rust`, `dry4rust` | 1 each |
| `etheram-core`, `crap4rust`, `grip4rust`, `iceberg4rust`, `slotgate`, `stern4rust`, `twin4rust` | **0** |

**The fixer, run on a copy of each of the nine**, repaired all 134: it touched
exactly the files reported and no other, every repaired tree checked clean,
`cargo fmt --check` passed on each, and `cargo check --workspace --all-targets`
passed on the three that build on the host without the rest of their tree:
`etheram-raft`, `etheram-tools` and `etheram-ibft`. The diffs were read as well as counted, the
way ADR-FixOnlyWhatIsSafe's two bugs were found: doc comments, attributes and
the comment above a constant travel with it, and nothing else moves.

## Rejected alternatives

**Leaving constants beside their first reader.** It is the habit the census
found, and it has a reason: the reader sees the number where it is used. It is
also the reason a file's numbers cannot be found without reading it, and a
second reader breaks it. Rejected.

**Sorting the constants group.** Deferred rather than rejected: R002 sorts a test
file's constants, and the same may be right here, but it was not measured.

**Shipping without `--fix`.** Rejected, as for R027: 137 files moved by hand is
the cost the fixer exists to remove.

## Consequences

**A file's numbers are at its top.** With
[R032](R032-ADR-SharedConstantsRule.md) beside it, a reader knows where a
constant is: at the top of the one file that uses it, or in a constants file.

**Seven repositories already comply, this one among them; adoption elsewhere is
one `--fix` commit.**

### What this rule does not catch

- **Associated constants**, which [R027](R027-ADR-OrderedImplFunctionsRule.md)
  places at the head of their impl.
- **Constants inside inline modules or function bodies.**
- **Items generated by a macro**, invisible to `syn`.

## Enforcement

`tests/rules/source/leading_constants_rule_tests.rs` -- twenty tests: a constant
after a struct, an impl, a function and a type alias, each reported; a static
like a constant; constants straight after the imports, a file of only constants
and an inline module's constants left alone; two misplaced constants reported
once, at the first; the offence naming the constant and the item it follows;
the correction offering `--fix`, or saying why the fixer declines; `lib.rs`,
`mod.rs` and `tests/` exempt; the unparseable file; `check_workspace`,
`is_configured`, the name and the requirement.

`tests/finding/parsing/constant_layout_parser_tests.rs` (eighteen) pins what is
found and where it would go: a `mod` declaration not an item and an inline
module one; a constant's block reaching over its attributes and comment; the
insertion point after the imports, after a constant already in place, and
before the first item's block when nothing precedes it; and each refusal -- a
`macro_rules!`, a constant on its neighbour's line either side, the leader on an
import's line, an opening comment running into the leader.

`tests/leading_constants_rewriter_tests.rs` (fifteen) pins the fixer: a constant
moved straight after the imports or appended to the group, carrying its
attributes and comment; the header kept above it; a trailing comment kept; every
other line as written; one blank line where a constant was; a multi-line string
moved whole; a repaired file the rule then finds clean; and `None` for a correct
file, an unparseable one, each refusal, a registry and a test file.

`tests/runner_tests.rs` runs `--fix` end to end -- a misplaced constant moved,
`fixed=1` and a clean run; a package that skips the rule left untouched -- and
its `--rules` listing holds twenty-seven rules.
`tests/finding/model/{misplaced_constant,fix_refusal,constant_layout}_tests.rs`
pin the model, `tests/source_file_tests.rs` the shared `block_start` and `is_under`,
`tests/finding/parsing/item_naming_tests.rs` the shared `implemented_type`, and
`tests/rule_registry_tests.rs`' four rule-name lists include
`leading-constants`. Stage 2 runs this tool, built from the checkout, against
this crate with every rule applied and zero offences.

## Related

- [R002-ADR-TestFileStructureRule](R002-ADR-TestFileStructureRule.md) -- the same
  order for test files, and the fixer this one extends.
- [R027-ADR-OrderedImplFunctionsRule](R027-ADR-OrderedImplFunctionsRule.md) -- the
  same argument one level down, inside an impl.
- [R032-ADR-SharedConstantsRule](R032-ADR-SharedConstantsRule.md) -- which file a
  constant belongs in at all.
