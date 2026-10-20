# Open Points

Known gaps in what is built. A checking tool is read as exhaustive by default,
so a gap nobody wrote down is indistinguishable from a gap nobody has yet hit —
which is why each rule ADR carries a "what this rule does not catch" section and
why the sharp ones are collected here.

Ordered by how likely they are to mislead somebody.

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

## `returned-mutations` is skipped outside the `*4rust` tools and `slotgate`

The rule this section used to argue for shipped in 0.14.0 as R022, and the
three decisions it owed were taken: constructors that store a `&mut` borrow are
reported like any other parameter, `tests/` is exempt, and the rule is on by
default ([R022](ADRs/R022-ADR-ReturnedMutationsRule.md)).

The `*4rust` tools and `slotgate` run it. Every other repository that installs
stern4rust skips it in its own `stern4rust.toml` for now -- the etheram
repositories and `faction` held most of the 238 offences the census counted.
What is open is when each removes its skip: the order is the owners' to set. The
campaign that held `etheram-ibft` and `etheram-ibft-embassy` back closed on
2026-10-17, so nothing now keeps either from starting.

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

The primitives were exactly that failure and are no longer guessed at. `u64`
opens lowercase, so `u64::from_le_bytes` was reported with the correction
`use u64::from_le_bytes;` -- which does not compile at all, since a primitive's
inherent function cannot be imported. An offence whose correction is not Rust is
worse than a missed one. Their names are fixed by the language rather than by a
convention, so the set can be written down: `u8` through `u128`, the signed and
pointer-sized forms, `f32`, `f64`, `bool`, `char` and `str`. Matched whole, so
`u64_helpers` stays a module. Found in `etheram-ibft`, five offences no edit
could clear.

What remains open is the rest of the trade: a lowercase type this crate cannot
name -- someone's `mod widget` holding `struct widget` -- is still reported, and
an uppercase-named module still passes.

## No rule sees a constructor that works a dependency before storing it

`RaftProtocol::new` in `etheram-raft` took its cluster membership as
`mut cluster_membership: Box<dyn ClusterMembership>` only to send it an event
before storing it -- a constructor operating a dependency it was handed, and the
one call that reached that bookkeeper around its owner. Nothing reported it, and
nothing here could: `returned-mutations` reads a parameter's type, and a `mut`
binding on a value the function owns changes nothing the caller can see. The
fix was structural -- the owner, `Bookkeepers`, now sends the event through its
own entry point -- not a different return type (`etheram-raft` `9b3ab05`).

A census on 2026-10-20 over sixteen repositories, 2,939 files under `src/`,
weighed three rules:

- **No `mut` by-value parameter anywhere:** 216 offences -- 126 builders
  (`with_x(mut self) -> Self`), 84 others, 6 `FnMut` callbacks -- and not one
  the smell. Sorting an owned `Vec`, folding an image through `apply_x(mut
  image)`, an Embassy task's `mut runner`: honest local mutation, which the rule
  would push into a rebinding or worse code.
- **No `mut` parameter in a constructor** (a function returning `Self` with no
  receiver): 4 offences, none the smell -- three `FnMut` observers a
  constructor must call, one `Vec` sorted before it is stored.
- **No `mut` dependency in a constructor**, a dependency being a `dyn Trait`
  anywhere in the parameter's type or an `impl Trait` other than the `Fn`
  family: 0 offences, and the one known case caught.

Only the third earns a rule: no false positive anywhere, and it says what the
house standard already says -- a constructor stores what it is given. It is
insurance against a rare smell rather than a cleanup, which is why it waits.
What it owes before it ships:

- the syntactic test for a dependency, and whether a generic parameter bounded
  by a non-`Fn` trait counts;
- the rebinding that would defeat it, `let mut dependency = dependency;` inside
  the constructor, reported as the same offence;
- whether a constructor that calls a dependency's `&self` methods before
  storing it is the same smell or a different rule.

## No rule sees an effect a function hides behind `()`

A function that takes neither `&mut self` nor a `&mut` parameter and returns
`()` can only act by side effect -- I/O, a log line, a static, a channel, a cell
behind `&self`. With [R022](ADRs/R022-ADR-ReturnedMutationsRule.md), a rule
saying such a function returns something would leave every effect in a
signature: `&mut self`, or a value handed back. That is the functional core the
newer rules lean towards, and what `grip4rust` scores as pure functions.

A census on 2026-10-20 over sixteen repositories found 602 such functions in
8,425 -- 348 `&self` methods, 158 associated or free functions, 9 consuming
`self`, and 87 that already take a `&mut` parameter and are R022's. Read through,
they are mostly where effects belong: log sinks, report printers, output
dispatchers, transports, test-harness commands, firmware `init` and `run`. The
rule would report the imperative shell for being one.

It earns its place only scoped to the code that should be pure -- a protocol's
core, not its adapters -- and this tool has no notion of which package is which.
What it owes before it ships:

- a way to name the core: a per-package opt-in in `stern4rust.toml`, the
  inverse of the usual skip, so the rule is off unless a package says it is pure;
- the census again, on the packages a repository would name, to see whether
  what is left is a defect or a log line;
- the same exemptions as [R024](ADRs/R024-ADR-OwnedFunctionsRule.md) for
  functions a framework requires, and `-> !`.

## No rule sees a literal repeated where a name belongs

`32` written twenty-five times across a crate is a hash length nobody named, and
the day it changes, the copy that was missed disagrees in silence. Clippy has no
lint for it: `unreadable_literal`, `decimal_literal_representation` and
`approx_constant` judge how a literal is written, not whether it wanted a name.

A census on 2026-10-20 over sixteen repositories weighed two rules:

- **No numeric literal outside a constant**, 0 and 1 excepted: 2,354 offences,
  led by `2`, `3`, `5`, `4`, `32`, `8`, `100`, `10`, `20`, `100.0` and `1000.0` --
  percentages, millisecond conversions, sizes, indices. The classic rule reports
  arithmetic for being arithmetic. Rejected.
- **The same literal twice or more in a crate**, outside constant initialisers,
  0, 1, 2 and -1 excepted, values compared after normalising (`1_000u64` is
  `1000`): 366 groups, 1,431 occurrences, 192 groups spanning more than one file.

The second has real signal at its top: `32` repeated 25 to 32 times in each of
`etheram-ibft`'s `evm`, `node`, `validation` and `system-tests` crates, a hash
length; `4` across both flash-storage crates, a flash word; `5` fourteen times
in one analysis crate, the number of terms in a model. Below that it is `3`, `4`
and `5` as cluster sizes in test-support crates, and fixtures.

What it owes before it ships:

- a threshold: repeats spanning files only, a floor on the value, or both --
  measured, not chosen;
- the per-package skip for test-support crates, as
  [R030](ADRs/R030-ADR-TypedErrorsRule.md) uses for harnesses;
- where the named constant goes, which
  [R032](ADRs/R032-ADR-SharedConstantsRule.md) answers: one file's constant is
  private to it, a shared one lives in a constants file.

