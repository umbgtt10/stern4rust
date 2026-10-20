# R029-ADR-DistinctTupleElementsRule

- **Status:** Proposed
- **Date:** 2026-10-20

## Context

`-> (u64, u64)` returns two numbers and says nothing about which is which. The
compiler cannot help: swapping them type-checks. The caller has to know the
order from a comment, a test or a guess, and an agent writing a caller in a
session that has never seen the function guesses.

Most tuples in a return type are not like that. A map keyed by
`(Height, u64, PeerId)`, a `Vec<(PeerId, Message)>`, an `into_parts` returning
`(Storage, u64)`: the types themselves say which element is which, and a swap
does not compile. The danger is narrower than "tuples": it is two elements of
the same type, where position is the only thing that tells them apart.

## Decision

**A tuple in a return type repeats no element type.** Any function in `src/`
whose return type holds a tuple with two elements of the same type is an
offence, wherever the tuple sits: `-> (u64, u64)`, `-> Option<(Instant,
Instant)>`, `-> Vec<(usize, usize)>`, and in a private function as much as a
public one.

- **Same type** is compared as written, whitespace and lifetimes ignored:
  `&'a str` and `&'static str` are the same; `Height` and `u64` are not, even
  where one aliases the other -- an alias that names a meaning is the cure, not
  a loophole.
- **Tuples are found by their parentheses with a comma at top level**, so
  `impl Fn(u8, u8) -> u8` is a callback's parameter list, not a tuple, and `()`
  is not one.
- **Trait impls are left alone**, their signatures being the trait's.
- **One offence per function**, at its name: *"`(u64, u64)` names neither of
  its `u64`s; return a struct whose fields say which is which"*.
- **Exempt:** `tests/`.

## Forcing constraints / Evidence

The census of [R028](R028-ADR-ImmutableReturnsRule.md), the same sixteen
repositories and 8,425 functions, first counted every tuple in a `pub` return
type: 221 -- 87 at top level, 129 wrapped, 5 of the constructor shape
`(Self, ...)`. Read one by one they are mostly the safe kind above. Narrowed to
tuples that repeat a type, the count is **86**:

| | top level | wrapped |
|---|---:|---:|
| `pub` | 34 | 24 |
| private | 16 | 12 |

And they are the ambiguous kind:

- `IbftProtocol::execute_and_compute_commitments -> (Hash, Hash)` -- which one
  is the state root;
- `summarize_receipts -> (usize, usize, usize, usize)`;
- `wait_until_leader_elected -> (u64, u64)`;
- `CivilDate::from_days -> (i64, u64, u64)`;
- `LedgerRecording::write -> (PathBuf, PathBuf)`;
- about twenty count accessors in an analysis workspace returning
  `(u64, u64)` or `(usize, usize)`, a part and its whole.

A handful are positional by design -- two sort or hash keys
(`Offence::sort_key` in this crate, `SimilarityPair::key` in `dry4rust`) and a
fixture's `center -> (f64, f64)` -- and are offences anyway: a named key that
derives `Ord` orders the same way, and `x` and `y` are better as fields than as
a convention.

**This crate has one offence**, `Offence::sort_key -> (&str, usize,
&'static str)`, which the change that builds the rule fixes first: a rule that
would fail this repository cannot be merged into it.

## Rejected alternatives

**No tuples in a `pub` return type at all.** 221 offences, most of them the safe
kind; the rule would be a style preference about tuples rather than a defence
against a swap. Rejected.

**Top-level tuples only.** It would miss `Option<(Instant, Instant)>`, which is
the same two positions behind one `Some`. Rejected: 36 of the 86 are wrapped.

**`pub` functions only.** Rejected: a private function's callers are in its own
file and can swap its results as easily. 28 of the 86 are private.

**Exempting functions named `key` or `sort_key`.** A name is not a type, and an
exemption by name is the first one somebody borrows. Rejected.

## Consequences

**A function that returns two values of one type names them.** The fix is a
small struct with two fields, and the struct's name usually turns out to be a
concept the code was missing: a tally, a pair of roots, a range.

**Adoption is by baseline** where a repository cannot clear its share at once;
the analysis workspace holds a third of the offences.

### What this rule does not catch

- **Two different types that convert into each other silently** -- `u32` and
  `u64` behind `into()` -- which is a type question.
- **Parameter lists with two arguments of one type**, which have the same
  problem on the way in. A separate question, and a larger census.
- **Tuples built by a macro**, invisible to `syn`.

## Enforcement

To be written test-first when the rule is built: a repeated type at top level,
wrapped in `Option`, `Result` and `Vec`, in a map key; distinct types left alone;
lifetimes ignored, aliases not resolved; `impl Fn(A, A)` and `()` not tuples;
trait impls left alone; private functions reported; `tests/` exempt; the
unparseable file; `check_workspace`, `is_configured`, the name and the
requirement; `tests/rule_registry_tests.rs` updated; this crate's
`Offence::sort_key` replaced by a named key first.

## Related

- [R028-ADR-ImmutableReturnsRule](R028-ADR-ImmutableReturnsRule.md),
  [R030-ADR-TypedErrorsRule](R030-ADR-TypedErrorsRule.md) -- the other two rules
  about what a function returns.
- [R027-ADR-OrderedImplFunctionsRule](R027-ADR-OrderedImplFunctionsRule.md) --
  the same argument from agents: what a session can get wrong without noticing,
  a rule should make impossible.
