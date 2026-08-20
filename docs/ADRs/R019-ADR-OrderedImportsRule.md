# R019-ADR-OrderedImportsRule

- **Status:** Accepted
- **Date:** 2026-08-20

## Context

`test-file-structure` has required alphabetical imports since `0.2.0`, and only
in `tests/`. Nothing asked it of the source tree.

That gap widened as the tool grew. `imported-paths` routinely **adds** imports to
productive files — 201 of them across the sibling tools once its offences are
worked through — and nothing said where a new line should land. The result is the
shape found in `etheram-ibft`: a file whose first import block is properly sorted
and whose second is whatever order things were needed in, two crates interleaved
in three alternating runs.

`OPEN_POINTS.md` has carried this as "Nothing orders the imports of a productive
file" since `imported-paths` shipped, and `ROADMAP.md` describes it as "removing
a scope restriction rather than a new rule".

## Decision

**Imports in `src/` run in alphabetic order**, on the pairs where the alphabet is
the authority.

The rule reuses [`ImportPath`](../../src/finding/import_path.rs) rather than
deciding again. `cargo fmt` runs first in the gate and orders `self`, `super`,
`crate` and uppercase-initial paths by rules of its own, so a rule demanding the
alphabet there would write a file **no edit could make green** — each run undoing
the last. That failure was hit once already, while fixing `imported-paths`, and
is why the stand-down is decided per pair rather than per import.

A block ends where the lines stop being consecutive. A blank line or a comment
between two imports separates them, and the first import of a block is compared
with nothing. Item spans decide this, so a multi-line `use` is handled without a
special case.

`tests/` is left to `test-file-structure`, which asks the same question of a
stricter shape — one contiguous block, no blank lines — and would contradict this
rule if both applied.

| offence | correction |
|---|---|
| `` `use aaa_crate::Alpha;` is out of alphabetic order; it follows `use zzz_crate::Zed;` `` | move `use aaa_crate::Alpha;` above `use zzz_crate::Zed;` |

The wording deliberately matches `test-file-structure`'s, because it is the same
finding one directory over.

## Forcing constraints / Evidence

**Verified consistent with `cargo fmt` before the rule was written**, since that
is the only way it could be wrong in a way no user could fix.

A controlled experiment on default settings — neither this repository nor
`etheram-ibft` has a `rustfmt.toml` — established what rustfmt actually does:

```rust
// scrambled input, after cargo fmt:
use crate::alpha::A;   // crate:: hoisted first, as a block
use crate::beta::C;
use aaa_crate::B;      // then plain alphabetical
use std::fmt;
use zzz_crate::Z;
```

So rustfmt is **not** alphabetical, and a naive rule would demand `aaa_crate`
above `crate::` while `cargo fmt` put it back. With the stand-down, the two
agree on every pair the rule still judges.

The 10 findings were then confirmed to survive formatting, three ways:
`cargo fmt -p node-infra --check` clean, standalone `rustfmt --check` clean, and
the offending block extracted into a scratch crate and formatted returned
**byte-identical**. rustfmt abandons that group — the likely cause is a
158-character single-segment import it cannot wrap; a 116-character
reproduction *was* reordered.

Across the family the rule finds **10 offences, all in `etheram-ibft`**: 5 in
`node`, 4 in `node-infra`, 1 in `evm`. Every other repository, including this
one, is clean — `cargo fmt` sorts what it rewrites, and the stand-down covers
the rest.

`ROADMAP.md` predicted this would "arrive as a wave of offences". It does not,
and that entry was corrected.

## Rejected alternatives

**Demand plain alphabetical order.** Rejected on measurement: it contradicts
`cargo fmt`, which runs first in the gate, producing a file that cannot be
fixed. This is the one disagreement in the tool with no resolution.

**Write the comparison again rather than reuse `ImportPath`.** Rejected:
`OPEN_POINTS.md` already regrets the same logic living in three places, and the
case rules here are subtle enough that a second copy would drift within a
release.

**Extend `test-file-structure` to `src/` instead of adding a rule.** Rejected:
that rule requires one contiguous import block with no blank lines, which is the
`tests/` convention and not the `src/` one. Applying it whole would report every
source file in this crate.

**Require one contiguous import block in `src/` too.** Rejected for the same
reason from the other side — `std` / external / `crate` grouping is what
`src/` files here already do, and `cargo fmt` preserves those groups.

**Report the whole block rather than the pair.** Rejected: the correction is a
single line move, and naming the pair says exactly which.

## Consequences

**The stand-down does far more work here than in `tests/`, and that is the cost
worth stating.** A test file imports the crate under test, so `crate::` never
appears; a source file usually leads with a block of it. Measured on this crate:
**235 adjacent import pairs in `src/`, 131 of them (56%) stand down.**

So more than half of what the rule appears to check, it does not — silently. A
clean run means "no pair the alphabet governs is out of order", not "the imports
are sorted".

### What this rule does not catch

**Any pair `cargo fmt` decides**, which is the 56% above: `self`, `super`,
`crate`, and paths diverging at a segment of differing case.

**Grouping.** Whether `std`, external crates and `crate::` are separated into
blocks at all, and in what order those blocks appear, is not judged — only order
within a block.

**A missing or redundant import.** That is `imported-paths`' and the compiler's.

**Imports inside inline modules or macros.** Only top-level `use` items are
compared.

**Anything outside `src/`.** `tests/` belongs to `test-file-structure`, and
`benches/` or `examples/` are judged by neither.

## Enforcement

`tests/rules/ordered_imports_rule_tests.rs` — 12 tests covering the sorted
block, the unsorted block, several unordered pairs, the `crate::` stand-down,
the uppercase first segment, the case divergence inside a path, blocks separated
by a blank line, a test file, an unparseable file, `check_workspace`,
`is_configured`, and the rule's name.

`tests/finding/import_path_tests.rs` — the stand-down itself, unchanged and
already covered by `test-file-structure`.

Stage 2 runs the tool against this crate at zero offences.

## Related

- [R002-ADR-TestFileStructureRule](R002-ADR-TestFileStructureRule.md) — asks the
  same question of `tests/`, in a stricter shape, and owns `ImportPath`.
- [R008-ADR-ImportedPathsRule](R008-ADR-ImportedPathsRule.md) — the rule that
  adds the imports this one orders.
