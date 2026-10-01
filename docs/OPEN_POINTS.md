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

## No rule enforces "no `&mut` input parameters"

The standard is written down in sixteen repositories' `CLAUDE.md`, this one
included (line 191: "no `&mut` input parameters; prefer return values"), and none
of the twenty-one rules checks it. Seven of the sixteen add "`&mut self` is allowed
in traits and impl blocks"; `faction` words it "no `&mut` parameters -- prefer
return values over mutation". `etheram-ibft-embassy-demo` does not state it at all.
**A clean run in a repository that states this standard says nothing about whether
the standard holds**, and a reader who assumes the tool covers the house standards
will read it as holding.

Evaluated on 2026-10-01 and parked rather than built. Everything below is the
evaluation, kept so the decision does not have to be re-derived.

**A rule is viable, because the question is syntactic.** Whether a parameter's
type is `&mut T` is visible in the signature; no type resolution is needed, which
is what defeated the three attempts recorded in
[R012](ADRs/R012-ADR-TestNamingRule.md). It fits the existing shape exactly: one
file, `core/src/rules/source/read_only_parameters_rule.rs`, built like
`pure_traits_rule.rs` -- `syn::parse_file`, walk the items, an `Offence` with a
subject so it can be baselined -- plus one line in `RuleRegistry::all`, an ADR
(`R022`), a row in `docs/RULES.md` and `README.md`, and tests. The name follows
`Rule::name`'s guidance, reading as what is required: `read-only-parameters`.

### What was measured

A throwaway `syn` scanner, not stern itself, run read-only over 17 repositories:
`braintax4rust`, `crap4rust`, `dry4rust`, `etheram-core`, `etheram-embassy`,
`etheram-ibft`, `etheram-ibft-embassy`, `etheram-ibft-embassy-demo`,
`etheram-raft`, `etheram-raft-embassy`, `etheram-tools`, `faction`, `grip4rust`,
`iceberg4rust`, `slotgate`, `stern4rust`, `twin4rust`. 4,267 files, 22,263
function signatures, zero parse failures. A parameter counts when its type is
`&mut T` at the top level; receivers are counted apart.

| where the `&mut` parameter sits | count |
|---|---|
| `src/`: free functions | 127 |
| `src/`: inherent methods | 113 |
| `src/`: trait declarations | 8 |
| `src/`: trait impls, local / foreign | 17 / 26 |
| `tests/` | 88 |
| nested in a callback type -- `impl FnMut(&mut T)`, `fn(&mut T)` | 3 |
| closures, `extern` functions, nested in data -- `Option<&mut T>` | 0 |

The ADR should re-measure with the rule itself, as
[R017](ADRs/R017-ADR-ArrangeActAssertRule.md) did, since the scanner and the rule
may draw the edges slightly differently.

### Exemptions the measurement supports

Each is decidable from the syntax:

1. **Receivers** -- `&mut self`, `self: Pin<&mut Self>`. The standard allows them.
2. **Trait impls.** The trait owns the signature. The 26 foreign ones --
   `Display::fmt`, embassy's `Driver`, eframe's `App`, tonic's `Service` -- cannot
   be changed by the implementor at all. A local trait is judged once, at its
   declaration (8), rather than once per impl (17).
3. **`&'static mut T`** (3). A permanent hand-over rather than an in-out
   parameter, and embassy-net's stack requires one.
4. **Callback types** (3). The parameter is a function, not a mutable input.
5. **Constructors that store the borrow** -- *a decision, not yet taken.*
   `new(protocol: &'a mut IbftProtocol) -> Self`, where `'a` is a lifetime of
   `Self`, keeps the borrow and mutates through `&mut self` methods. 7 of them, 6
   in `etheram-ibft`'s protocol helpers. Flagging them pushes against the small
   types the other rules encourage; the alternative is extension `impl` blocks for
   the borrowed type in their own files, which `single-implemented-type` permits
   since the type is not declared there.
6. **`tests/`** -- *a decision, not yet taken.* Exempt as `pure-traits` exempts it?
   88 occurrences, 47 of them in `etheram-ibft`'s harness.

### What it would report

With all six exemptions, **238 offences in 13 of the 17 repositories**:

| repository | offences |
|---|---|
| `etheram-ibft` | 114 -- 57 in the EVM interpreter, 41 in the node's runtime and protocol |
| `dry4rust` | 31 |
| `etheram-tools` | 19 |
| `etheram-raft` | 16 |
| `etheram-embassy` | 13 |
| `etheram-ibft-embassy-demo` | 11 |
| `faction` | 7 |
| `grip4rust` | 6 -- 5 of them in the deliberately sloppy fixture, which should be excluded |
| `twin4rust`, `etheram-raft-embassy`, `etheram-ibft-embassy` | 5 each |
| `iceberg4rust` | 4 |
| `crap4rust` | 2 |

`stern4rust`, `slotgate`, `braintax4rust` and `etheram-core` are already clean, so
the rule would pass its own repository on its first run.

**Every remaining case is fixable without changing allocation behaviour.** Move the
state into a struct and make the function a `&mut self` method. That matters on the
boards: the 31 `&mut Vec<u8>` buffers in `etheram-ibft` do not have to become fresh
allocations -- an encoder that owns its buffer keeps the single allocation, so the
rule does not trade a style finding for heap churn on a target whose heap has
already been exhausted once.

### What it cannot catch

For the ADR's "does not catch" section:

- **Interior mutability** -- mutation through a shared reference: `&RefCell<T>`,
  `&Cell<T>`, `&Mutex<T>`.
- **A `&mut` laundered through a generic.** `fn f<W: Write>(w: W)` called with
  `&mut buf` is the same mutation, and the signature shows nothing.
- **A type alias that hides `&mut`**, a raw `*mut` pointer, and any function a macro
  generates.

### Rollout

Every repository in the family installs stern4rust without pinning a version and
enforces every rule, so a twenty-second rule that is on by default turns stage 2
red in the 13 repositories above at their next install. Each then fixes, baselines
with a recorded reason ([ADR-Baselines](ADRs/ADR-Baselines.md)), or skips the rule.
`etheram-ibft` and `etheram-ibft-embassy` are frozen while a campaign runs and
should be left alone until it ends; their gates cannot run during the freeze anyway,
because both stages begin with `cargo update`. The documents that say "twenty-one
rules" -- [ADR-RulesExplainThemselves](ADRs/ADR-RulesExplainThemselves.md),
`etheram-core`'s `CLAUDE.md` -- move with it.

**Effort is small**, about the size of `pure-traits`: a rule file of 150--200
lines, about 25 tests, the ADR -- most of whose evidence is above -- and the
registry line, the `RULES.md` and README rows, and a version bump.

### Decisions owed before building it

1. Exempt constructors that store the borrow?
2. Exempt `tests/`?
3. On by default, or opt-in for one release while the family adopts it?

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
