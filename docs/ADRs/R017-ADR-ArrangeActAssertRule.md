# R017-ADR-ArrangeActAssertRule

- **Status:** Accepted
- **Date:** 2026-08-20

## Context

This was the original motivating example for the whole tool, and it shipped
seventeenth. `ROADMAP.md` has carried it since the first release as "the oldest
unbuilt one".

Everything around it was built first. `test-file-structure` judges the shape of
the file a test sits in. `test-naming` judges the name a test carries.
`tested-public-api` judges whether a test exists at all. **Nothing judged the
body** — and the body is where a reader decides, in five seconds, whether they
can trust what the name claims.

It waited because of a problem that has nothing to do with AAA, described under
the Decision.

## Decision

**A test reads `Arrange`, then one or more `Act`/`Assert` pairs**, with a blank
line separating the sections.

Every marker expands into the phases it names, and the expanded sequence must be
`Arrange` followed by one or more `Act`, `Assert` pairs. That single check covers
every legal shape, because the merged forms expand identically to the separate
ones:

| written | expands to |
|---|---|
| `// Arrange` `// Act` `// Assert` | Arrange, Act, Assert |
| `// Arrange & Act` `// Assert` | Arrange, Act, Assert |
| `// Arrange` `// Act & Assert` | Arrange, Act, Assert |
| `// Arrange & Act & Assert` | Arrange, Act, Assert |

and it rejects, without a second rule, a test whose Act has no Assert, whose
Assert has no Act, whose markers are absent, or whose Arrange was **dropped
rather than merged**.

**`// Arrange & Act & Assert` is now part of the standard.** It was not in
`CLAUDE.md`, and practice had already outrun the standard: nine tests use it
across the family, including `tests/finding/model/section_tests.rs` in this crate. A
rule written strictly to the standard as it stood would have failed its own
repository on the first run. `CLAUDE.md` was amended rather than the nine tests.

**A marker may carry trailing prose**, after `--`, `:` or `.`. All three appear
in the family — `// Arrange -- four nodes, one split`, `// Act: heal the
partition`, `// Assert. every node commits`. Demanding a bare marker would report
precisely the tests that took the trouble to explain themselves.

**A marker ends on a word boundary**, so `// Actually this needs explaining` is
prose rather than an Act.

**Comment lines above a marker are folded into it**, the same way
`TestFileParser` folds them into the item they document, and for the same
reason its ADR gives: otherwise a marker that explains itself over two lines
reads as a spacing offence.

| offence | correction |
|---|---|
| `` `new_empty_collection_is_empty` reads Act, Assert; a test is `Arrange` followed by one or more `Act`/`Assert` pairs `` | label the sections `// Arrange`, `// Act` and `// Assert`, merging adjacent ones as `// Arrange & Act`, `// Act & Assert` or `// Arrange & Act & Assert` |
| `` `// Act & Assert` in `a_method_call_on_bare_self_…` is not preceded by a blank line `` | put a blank line before `// Act & Assert` |

### The hard part is not the grammar

**The markers are comments, and comments never reach the syntax tree.** `syn`
discards them, so this rule reads lines — and **a line scanner cannot tell code
from a string that contains code.**

That matters here more than anywhere, because this repository's own tests are
built from Rust source embedded in raw strings. Measured before the rule was
written, a naive line scanner reports **seven offences in this crate that are
every one of them a string literal** — the fixtures inside
`test_file_structure_rule_tests.rs` — and roughly **156 more across the family**,
where a `}` at column zero inside a raw string ended a test early and swallowed
its remaining markers.

So the lines every literal occupies are taken from the **token stream** and
skipped. Comments are not tokens and literals are, which is exactly the
distinction required. Walking tokens rather than visiting the syntax tree also
reaches inside macros, where `assert_eq!("// Act", x)` would otherwise hide one.

The rule reports **zero** offences against this crate, which is the measurement
that mattered.

## Forcing constraints / Evidence

The legal shapes were counted across **~3,900 tests in nine repositories** before
the grammar was chosen:

| shape | count |
|---|---|
| `Arrange` / `Act` / `Assert` | 2645 (68%) |
| `Arrange & Act` / `Assert` | 732 (19%) |
| `Arrange` / `Act & Assert` | 255 (6.6%) |
| `Arrange & Act & Assert` | 9 |
| several `Act`/`Assert` pairs | **2** |

**Multiple Act/Assert pairs are permitted, and they are almost theoretical** —
one test with two pairs and one with four, in 3,900. They are allowed because
they are the standard, but the cost is real and worth recording: once
`Arrange Act Assert Act Assert` is legal, a **stray or duplicated marker is
indistinguishable from a legitimate second pair**. The rule cannot catch a
copy-pasted `// Act` with no assertion behind it. Two tests bought that
permissiveness for the other 3,898.

Run across the family, the rule finds **45 offences**: 23 in `grip4rust`, 9 in
`braintax4rust`, 5 in `iceberg4rust`, 4 in `crap4rust`, 3 in `twin4rust`, 1 in
`etheram-core`, and **0 in `stern4rust` and `slotgate`**. `etheram-ibft` adds 10
in `node` and 14 in `validation`.

Both classes were verified by eye. `etheram-core`'s single finding is a bare
`// Act & Assert` with no Arrange, one word from correct. `grip4rust`'s spacing
findings are a two-line `// Arrange --` explanation with `// Act & Assert`
directly beneath it and no blank line between them.

45 in 3,900 is 1.2%. The convention is kept almost everywhere; this stops the
almost from drifting.

## Rejected alternatives

**Require the exact text `// Arrange`.** Rejected on measurement: it would report
hundreds of well-documented tests, since trailing prose is the established style
in the largest repository in the family.

**Scan lines without excluding string literals.** Rejected: it fails this
repository, on this repository's own fixtures, by seven offences. See the
Decision.

**Use `syn` to find the comments.** Not possible — `syn` discards them. This is
the constraint the whole design answers to.

**Require a blank line directly above every marker, with no folding.** Rejected:
a marker documented over two lines would be an offence, and the documentation is
the part worth keeping. `TestFileParser` had already made this call.

**Split the blank-line separation into its own rule.** Considered and rejected by
the maintainer: the sections and their separation are one standard, stated in one
line of `CLAUDE.md`, and two rules would let a repository adopt half a convention.

**Reject a bare `// Act & Assert` with no Arrange by requiring three separate
markers.** Rejected: `// Arrange & Act & Assert` is the merged form for exactly
that test, and now says so.

**Check what is *in* each section** — that the Act block calls something, that
the Assert block asserts. Rejected as the semantic guess that defeated three
versions of [R012](R012-ADR-TestNamingRule.md). This rule stays on the layout,
which is decidable.

## Consequences

**The rule is about layout, not truth.** A test labelled `// Assert` that
asserts nothing passes. That is the same retreat `test-naming` made, made
deliberately rather than after three failures.

**It is the first rule that reads lines and syntax together.** Every other rule
works from one or the other. This one needs `syn` for the function boundaries and
the literal spans, and the raw lines for the comments, and the correctness of the
whole rule rests on the join between them.

**A rewrite that moves a string literal can change the answer.** The literal's
span determines which lines are skipped, so reformatting a fixture is, in
principle, a semantic change to this rule. Nothing observed depends on it.

### What this rule does not catch

**Whether a section does what it says.** An `// Act` block that asserts, or an
`// Assert` block that mutates, is invisible.

**A stray or duplicated marker**, because multiple `Act`/`Assert` pairs are
legal. `Arrange Act Assert Act Assert` cannot be told from a copy-paste error.

**Tests generated by a macro.** They are not in the syntax tree, so their bodies
are never examined.

**A marker inside a string literal that *should* have been real** — the exclusion
is unconditional. A test whose entire body is built by a macro from a string
would read as having no markers, and would be reported rather than missed, which
is the safer direction.

**Anything outside `tests/`**, and anything in `all_tests.rs` or `mod.rs`. A
`#[test]` in `src/` belongs to [R005](R005-ADR-TestFreeSourceRule.md).

**Ordering within a section**, and whether `// Arrange` actually arranges
anything. A test may label an empty section.

## Enforcement

`tests/rules/testing/arrange_act_assert_rule_tests.rs` — 20 tests covering each legal
shape, several Act/Assert pairs, the missing Arrange, the Act with no Assert, the
Assert with no Act, the test with no markers, the spacing offence, the folded
comment block, trailing prose, the `Actually` word boundary, the helper function,
a source file, an unparseable file, and **the string literal holding markers**.

`tests/finding/model/test_marker_tests.rs` — 10 tests on marker parsing alone: each
merged form, trailing prose in all three punctuations, the word boundary, an
ordinary comment, and a line of code.

Stage 2 runs the tool against this crate at zero offences, over the whole test
suite including the string-literal fixtures this rule had to learn to ignore.

## Related

- [R012-ADR-TestNamingRule](R012-ADR-TestNamingRule.md) — judges the name a test
  carries; this judges the body beneath it. Both deliberately stop short of
  semantics.
- [R002-ADR-TestFileStructureRule](R002-ADR-TestFileStructureRule.md) — judges
  the file the test sits in, and established the comment-folding this rule
  reuses.
- [R005-ADR-TestFreeSourceRule](R005-ADR-TestFreeSourceRule.md) — owns the
  `#[test]` that is in the wrong tree entirely.
