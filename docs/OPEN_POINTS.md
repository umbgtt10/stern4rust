# Open Points

Known gaps in what is built. A checking tool is read as exhaustive by default,
so a gap nobody wrote down is indistinguishable from a gap nobody has yet hit —
which is why each rule ADR carries a "what this rule does not catch" section and
why the sharp ones are collected here.

Ordered by how likely they are to mislead somebody.

## A fixture tree inside the package is judged like the package

The walker skips only `target/` and `.git/`. A tree of sample code that a tool
analyses — fixtures, vendored source, generated output — is judged as though
this repository had written it, and the report says so plainly rather than
quietly declining to look.

The fix is layout, not a skip, and both repositories that had the problem have
since taken it: `grip4rust` and `crap4rust` each moved their fixture trees out of
the published package into a sibling `fixture/`, leaving the analysis unchanged.
`grip4rust` is now at 53 offences over 75 files, `crap4rust` at 163 over 67.

Where a tree genuinely cannot be moved — vendored code, generated output —
`--exclude <glob>` now covers it, and reports every pattern with the number of
files it removed so that an exclusion stays something the reader can see rather
than a silence. See
[ADR-ExclusionsAreCounted](ADRs/ADR-ExclusionsAreCounted.md).

## A `cfg_attr`-gated path still misleads `registry-completeness`

`registry-completeness` resolves a declaration to the file it names by
convention -- `mod alpha;` reaches `alpha.rs` or `alpha/mod.rs` -- so a file
reached through an explicit path is reported as never compiled when it compiles
perfectly well.

`declared-by-name` closes the plain `#[path = "..."]` case by forbidding the
attribute package-wide. What remains is `#[cfg_attr(unix, path = "...")]`, which
that rule deliberately allows as the one honest use: a platform-gated module
cannot resolve by name on every platform anyway, and reporting it would accuse
correct code. On the platform where it applies, `registry-completeness` still
misreads it.

The wrong answer arrives somewhere else entirely -- an innocent file accused of
never being compiled, with nothing connecting it to the attribute that caused
it -- which is the hardest kind to trace and the reason this stays written
down.

A related limit: the rule resolves names, not the module graph. A `mod.rs` that
is itself never declared makes its whole subtree unreachable, and each level is
reported separately rather than as one root cause.

## `pure-traits` does not see a blanket impl

The rule reads `Item::Trait` and reports any method with a default body. A
blanket impl -- `impl<T: Base> Extended for T { ... }` -- is an `Item::Impl`, so
none of it is looked at. **A trait emptied of defaults can have every one of them
restored through a blanket impl and the rule will report nothing.**

That is the same shared body in the same crate reached through a different
syntax, and it defeats the rule completely wherever somebody reaches for it. It
is not caught because catching it means deciding when a generic impl is a
convenience and when it is the whole implementation, which needs the trait
resolution this tool does not do.

Nothing in the family uses one today, so this is an untested gap rather than an
observed one. A default inherited from a supertrait in another crate is invisible
for the same reason, one level further out.

Related: the rule verifies a default body is *gone*, never that it was moved into
the implementors rather than deleted. `rustc` guarantees each implementor has *a*
body; nothing guarantees it is the right one.

## A test-only helper in `src/` is invisible

`test-free-source` catches test attributes and `test` cfg predicates. An
ordinary `pub fn make_test_widget()` sitting in `src/` carries neither, and
nothing in the source distinguishes it from production code.

`crap4rust` has the same blind spot for the same reason. There may be no
structural answer to this one.

## The header rule cannot check the copyright holder

A file whose header is perfectly formatted but names the wrong copyright holder
still passes. The SPDX half is now held by `spdx-matches-manifest`, which reads
its expected value from the manifest and so needs no `--header-file` -- it holds
in a repository that has no header file at all, which is where it found
`braintax4rust`'s headerless registry.

The copyright holder has no second source of truth to check against: `authors` in
`Cargo.toml` is optional, frequently stale, and not the same claim. That half
stays open and may have no structural answer.

## `readable-source` depends on `syn` accepting the file

Source using syntax newer than this crate's `syn` would be reported as
unparseable when `rustc` is perfectly happy with it. `syn` with the `full`
feature tracks stable Rust, so the window is narrow — but it is the only
false-positive class the rule has, and it would arrive as a confident wrong
answer rather than as a missing one.

## `#[cfg(...)]`-gated `pub mod` declarations are treated as ordinary

`tests-layout` does not distinguish `pub mod alpha_tests;` from
`#[cfg(feature = "x")] pub mod alpha_tests;`. Neither shape has come up in a
real tests tree; if one does, the rule would need to decide whether a
conditionally-declared test file counts as reached.

## Import ordering agrees with rustfmt only for same-case pairs

`ordered-imports` extended the check to `src/`, so this stand-down now governs
both trees -- and it does far more work in `src/`. Measured on this crate, **56%
of adjacent import pairs there are ones `cargo fmt` decides**, because a source
file usually leads with a `crate::` block where a test file never does. More than
half of what the rule appears to check, it does not, and a clean run means only
that no pair the alphabet governs is out of order.

The alphabetic check stands down on any import pair involving `self`, `super` or
`crate`, and on any pair that first differs at a segment where the two sides are
of different *case shape* -- the initial, and whether the segment is all
capitals. rustfmt orders those by its own rules and
`cargo fmt` runs first, so demanding the alphabet would make the file
unsatisfiable rather than merely wrong. The cost is that a genuinely scrambled
import list goes unreported at such a pair.

The earlier version of this note claimed the stand-down was keyed on an import's
first segment and was verified against the sibling tools' 168 distinct import
lines. Both were true and neither was sufficient: `use serde_json::Value;` beside
`use serde_json::from_str;` share a first segment and part company at the second,
and no file in those 168 lines happened to contain such a pair. Adding one while
fixing `imported-paths` produced a file no edit could make green. The decision is
now made per pair.

A third was added after that, for two segments that both open with a capital.
The check read only the first character, so `WAL_V2_MAGIC` beside `WalRecord`
counted as same-case and the alphabet was demanded -- and the editions disagree
about that pair exactly as they do about `Value` against `from_str`:

|                                          | 2021          | 2024           |
|------------------------------------------|---------------|----------------|
| `WalRecord` against `WAL_V2_MAGIC`       | `WalRecord`   | `WAL_V2_MAGIC` |
| `Block` against `BLOCK_GAS_LIMIT`        | `Block`       | `BLOCK_GAS_LIMIT` |
| `TinyEvmEngine` against `OPCODE_PUSH1`   | `TinyEvmEngine` | `OPCODE_PUSH1` |

Found in `etheram-ibft`, where it cost sixteen offences no edit could clear
across `ordered-imports` and `test-file-structure`, and both rules had to be
stood down there to keep `cargo fmt --check` green. Case is now read as a shape
-- the initial, and whether the segment is all capitals -- and pairs sharing one
are still judged: `ALPHA_TWO` against `ZETA_ONE`, `Alpha` against `Zeta` and
`alpha` against `zeta` were each measured identical under 2021, 2024 and a plain
sort.

A second stand-down was added earlier, for a path that *extends* another rather
than diverging from it: `use alloc::vec;` beside `use alloc::vec::Vec;`. The
first version could not see it, because it looked for the first pair of segments
that differ and there is no such pair -- the difference is between a segment and
nothing at all. Compared as written, the shorter line ends in `;` (59) where the
longer carries on with `::` (58), so a plain sort demands the longer path first
whatever follows it, and rustfmt demands the shorter. Every extension disagreed,
not only the uppercase ones. Found in `etheram-raft`, where it accounted for ten
offences no edit could clear.

Worth recording from measuring rustfmt, because it is not what anyone would
guess: **which direction case leans depends on the style edition, and the two
editions disagree with each other.**

|                                        | 2021        | 2024        |
|----------------------------------------|-------------|-------------|
| `Bbb::gamma` against `zzz::last`        | sorts last  | sorts first |
| `serde_json::Value` against `from_str`  | `from_str`  | `Value`     |

An earlier version of this note stated 2021's rule for a crate and 2024's for a
segment, so it described neither edition as a whole. That is the real argument
for standing down rather than picking a side: this crate cannot know which
edition the code under inspection compiles with, and declining to judge is the
only answer correct under either. On an extended path they agree -- shorter
first -- which is why that case could be settled rather than merely reported.

Also, `cargo fmt` and a standalone `rustfmt <file>` disagree here -- only
`cargo fmt` matters, since that is what the gate runs.

## Imports are sorted but never packed

Nothing asks that two imports from the same module become one statement.
`raft_bootstrapper.rs` in `etheram-raft` opens with six consecutive
`use faction::...;` lines and seven consecutive `use crate::bootstrapping::...;`
lines, and every rule here is satisfied.

**Nothing packs them because on stable rustfmt nothing can.**
`imports_granularity` and `group_imports` are nightly-only; the stable default is
`Preserve`, which sorts what is written and never merges or splits an import
tree. Switching the gate to a nightly `cargo fmt` for this would make stage 1
depend on a nightly toolchain for something cosmetic, which is the wrong trade.

**A rule is viable precisely because of `Preserve`.** Measured on stable rustfmt
1.8.0: a hand-written `use faction::{command::Command, conclusion::Conclusion};`
comes back untouched. rustfmt will not undo packing, so unlike the ordering
disagreements above this one has a resolution -- and it is auto-fixable, which
`ordered-imports` deliberately is not.

Three things such a rule has to get right, all of them measured rather than
assumed:

- **`self` in a brace group does not carry the macro namespace.** Packing
  `use alloc::vec;` and `use alloc::vec::Vec;` into `use alloc::vec::{self, Vec};`
  compiles the import and then fails the build: `vec![]` is no longer in scope.
  This is exactly the merge a naive same-module packer produces, and it broke
  `etheram-raft` when tried. A module imported for its macro cannot be packed
  with its own children.
- **rustfmt sorts inside the braces, by the edition's rule.**
  `use crate::a::{B, c};` comes back as `{c, B}` under edition 2021 and stays
  `{B, c}` under 2024. So the rule may ask that the braces exist; it must not
  also ask what order they hold, or it reopens the disagreement above.
- **Packing dissolves the extension pairs.** `use alloc::vec;` beside
  `use alloc::vec::Vec;` is the shape that forced the second stand-down. Where
  packing is possible it removes the pair; where it is not -- the macro case --
  the stand-down is still what keeps the file satisfiable.

The gap this leaves today is a file whose import list is correct by every rule
and still reads as one line per symbol.

## `imported-paths` tells a module from a type by case

`Widget::new()` is left alone and `widget::new()` is reported, and the only thing
separating them is the case of the first segment. That is a naming convention,
not a resolution -- this tool has no type information and stays on the syntax
tree.

A lowercase-named type would be reported when it should not be, and an
uppercase-named module would pass when it should not. Both are conventions
violated rarely enough that the trade is worth it, but the failure would arrive
as a confident wrong answer rather than a missing one, which is the worse shape.
