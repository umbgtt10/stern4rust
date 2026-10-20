# R027-ADR-OrderedImplFunctionsRule

- **Status:** Proposed
- **Date:** 2026-10-20

## Context

Where a function sits in an impl block is decided today by whoever wrote it:
usually narrative order -- an entry point, then the helpers it calls, related
functions together -- sometimes the order they were added. Nothing is wrong
with either until the reader has to find something, or two authors disagree, or
an impl grows past a screen and its order stops telling a story at all.

The house already settled the same question twice. Imports run in alphabetic
order ([R019](R019-ADR-OrderedImportsRule.md)), and a test file's sections are
each alphabetical ([R002](R002-ADR-TestFileStructureRule.md)). An impl block is
the one ordered list in the source that no rule reads. And the rules beside this
one -- [R024](R024-ADR-OwnedFunctionsRule.md),
[R025](R025-ADR-PublicOrPrivateRule.md), [R026](R026-ADR-ImplementedInOneFileRule.md)
-- push towards small types in a file each, where narrative order buys least and
findability most.

**In a repository written mostly by agents, consistency outweighs authorial
order.** Narrative order lives in its author's sense of the story, and here the
author changes every session: each one reconstructs the story differently, so in
practice the order drifts, and a reviewer has to judge every placement. A fixed
order is the same in every session and on every machine. It takes the decision
out of every change, keeps a new function's diff to the lines it adds, and lets
two agents extend one type with fewer collisions. A decision held by a gate does
not have to be remembered or argued again; that is the case for every rule here,
and the strongest one for this.

Order within an impl means nothing to the compiler. That makes this the one
ordering rule a tool can repair with no semantic risk at all, which is why it
ships with `--fix` or not at all.

## Decision

**An inherent impl block lists its functions in a fixed order: constructors,
then `pub` functions, then every other function.**

- **Constructors first, by number of parameters, fewest first;** ties
  alphabetical. A constructor is an associated function with no receiver whose
  return type names `Self` or the impl's type, directly or wrapped:
  `Self`, `Result<Self, E>`, `Option<Self>`, `(Self, Vec<Outcome>)`. Its
  visibility does not matter -- a private constructor is still where a reader
  looks for construction.
- **Then every `pub fn`, alphabetical.** `pub const fn`, `pub async fn` and
  `pub unsafe fn` are `pub fn`. A `pub` associated function that is not a
  constructor sits here too.
- **Then every other function, alphabetical.** With
  [R025](R025-ADR-PublicOrPrivateRule.md) that is every private `fn`; a
  repository that has not adopted R025 keeps its restricted functions here.
- **Alphabetical** compares identifiers byte by byte, a raw identifier without
  its `r#`. Two functions of one name -- `cfg`-gated twins -- stay adjacent, in
  source order.
- **Associated constants and types come before the functions**, in the order
  written: they are not functions, and the rule only asks that they not be
  scattered among them.
- **Scope:** inherent impl blocks in `src/`. Trait impls are left alone: their
  natural order is the trait's, below. `tests/` is
  [R002](R002-ADR-TestFileStructureRule.md)'s.
- **One offence per impl block**, at the impl, naming the first function out of
  place and where it belongs: *"`impl Bookkeepers` is out of order: `catch_up`
  belongs before `cluster`"*. One per block rather than one per function,
  because a single insertion can displace every function after it.

### `--fix`

`--fix` reorders every offending block, under the three constraints of
[ADR-FixOnlyWhatIsSafe](ADR-FixOnlyWhatIsSafe.md):

- **It works from `syn` spans and moves whole line ranges.** An item's range
  runs from its first attribute, doc comment or ordinary comment line directly
  above it -- no blank line between -- to its closing brace. Comments go where
  their function goes.
- **It touches only the block it is fixing.** Nothing outside the impl's braces
  is rewritten; the file is otherwise byte-for-byte what it was. Blank lines
  between functions become exactly one, and constants and types are followed by
  one.
- **It loses nothing.** A block it cannot reorder without dropping or orphaning
  content is declined, reported unchanged, and named in the fix summary as
  declined rather than fixed:
  - a comment standing free between functions, separated from the next one by a
    blank line -- a section heading with no item to travel with;
  - a macro invocation among the items, whose expansion the tool cannot see;
  - two items on one line.

  Trailing comments after the last item stay at the end of the block, as for
  R002's fixer.

`rustfmt` does not reorder impl items (its `reorder_impl_items` is unstable and
off), so the fixed order is never undone by the formatter run in front of this
tool.

## Forcing constraints / Evidence

A census on 2026-10-20 over the sixteen repositories of
[R025](R025-ADR-PublicOrPrivateRule.md) -- the same heuristic scanner and the same
caveat, to be re-run with this tool's finders -- validated first on
`Bookkeepers` (`new`, `cluster`, `consensus`, `catch_up`: out of order, `catch_up`
first). It judged every inherent impl block with two or more functions; 349 with
fewer cannot be out of order.

| repository | blocks | in order | out of order |
|---|---:|---:|---:|
| an analysis workspace | 298 | 92 | 206 |
| `etheram-ibft` | 174 | 55 | 119 |
| `etheram-embassy` | 165 | 82 | 83 |
| `etheram-ibft-embassy` | 114 | 47 | 67 |
| `stern4rust` | 81 | 31 | 50 |
| `etheram-raft` | 80 | 31 | 49 |
| `etheram-tools` | 74 | 33 | 41 |
| `grip4rust` | 41 | 16 | 25 |
| `dry4rust` | 42 | 19 | 23 |
| `etheram-raft-embassy` | 38 | 18 | 20 |
| `crap4rust` | 23 | 11 | 12 |
| `iceberg4rust` | 18 | 7 | 11 |
| `braintax4rust` | 23 | 13 | 10 |
| `slotgate`, `twin4rust` | 21, 17 | 13, 9 | 8 each |
| `etheram-core` | 8 | 6 | 2 |
| **total** | **1,217** | **483** | **734** |

What breaks, a block counted once for each part it breaks:

| part | blocks |
|---|---:|
| `pub` functions not alphabetical | 506 |
| private functions not alphabetical | 246 |
| constructors not by parameter count | 87 |
| `pub` and private interleaved | 83 |
| a constructor not first | 44 |

The grouping is already the house habit -- 44 and 83 blocks break it -- and the
alphabet is not: authors write in narrative order. That is the cost of the rule,
and it is a taste adopted as policy rather than a defect found.

**The fixer can repair 732 of the 734.** Two blocks hold a free-standing comment
between functions, and none holds a macro invocation among its items. Without
`--fix` the rule would ask for 734 blocks reordered by hand, which is the trade
this ADR refuses.

## Rejected alternatives

**Narrative order** -- callers above callees, the step-down rule. It is what most
code here does and what most readers like, and it is not checkable: whether
`b` reads best after `a` is a judgement, and a rule built on one accuses correct
code.

**The groups without the alphabet.** It would fail at most 127 blocks rather
than 734 -- a block can break both parts -- and leave the question a reader actually has -- where is `x`? -- unanswered.
Rejected because the grouping is already mostly there; the rule would add little.

**Constructors alphabetical rather than by parameter count.** Rejected: fewest
parameters first puts the simplest way to build the type first, which is the one
a reader wants first, and `new` before `with_capacity` before `from_parts` reads
as construction does.

**Ordering trait impls by the trait's declaration.** A separate rule: it needs the
trait's declaration in view, and the standard library's traits are not in the
tree this tool walks.

**One offence per misplaced function.** Rejected: a block with one function added
at the top would report every function after it.

**Shipping the check now and the fixer later.** Rejected: 734 blocks by hand is
the adoption cost the fixer exists to remove.

## Consequences

**Every repository in the family fails it on the first run**, `etheram-core` with
two blocks and the analysis repository with 206. Adoption is `--fix`, one commit
per repository, reviewed as a diff and proved by an unchanged build and test
suite -- and, for a parser held to golden outputs, by those outputs. The two
declined blocks are reordered by hand or baselined.

**Where a function is stops being a decision.** New functions go where the rule
says; review stops arguing about it.

**Narrative order is given up.** A section comment directly above the first of a
group travels with that function and may end up heading the wrong ones; the diff
of the fixing commit is where that is caught.

### What this rule does not catch

- **Trait impls**, above.
- **Whether a long impl should be split.** Order makes a long block searchable,
  not short; R026 and review decide its size.
- **Functions generated by a macro**, invisible to `syn`.

## Enforcement

To be written test-first when the rule is built:

- **The check:** each group's order, constructor detection through `Result`,
  `Option` and tuples, a private constructor, constructor ties, `pub` variants,
  constants and types ahead of functions, `cfg`-gated twins, trait impls left
  alone, `tests/` exempt, one offence per block naming the first misplaced
  function, the unparseable file, `check_workspace`, `is_configured`, the name
  and the requirement.
- **The fixer:** a block reordered with its attributes, doc comments and attached
  comments; blank lines normalised; the bytes outside the impl unchanged; a
  free-standing comment, a macro invocation and two items on one line each
  declining the block; trailing comments kept; a fixed file re-checked clean; a
  file with no offence left untouched.
- `tests/rule_registry_tests.rs` updated, and stage 2 run against this crate after
  its own `--fix`.

## Related

- [R019-ADR-OrderedImportsRule](R019-ADR-OrderedImportsRule.md),
  [R002-ADR-TestFileStructureRule](R002-ADR-TestFileStructureRule.md) -- the two
  orders the house already keeps; R002's fixer is the machinery this one extends.
- [ADR-FixOnlyWhatIsSafe](ADR-FixOnlyWhatIsSafe.md) -- the constraints the fixer
  is held to.
- [R025-ADR-PublicOrPrivateRule](R025-ADR-PublicOrPrivateRule.md),
  [R026-ADR-ImplementedInOneFileRule](R026-ADR-ImplementedInOneFileRule.md) -- the
  rules that make "every other function" mean private, in one file.
