# R002-ADR-TestFileStructureRule

- **Status:** Accepted
- **Date:** 2026-08-18

## Context

A test file grows by accretion. Each new test is appended wherever the last
one happened to end, each new helper is dropped next to the test that needed
it, and constants accumulate wherever they were first used. After a year the
file has no shape, and the practical cost is not aesthetic: answering "is
this case already covered?" requires reading the file rather than scanning
it, so cases get covered twice and gaps stay open.

The house standard already specifies the shape — header, then imports, then
constants, then helpers, then tests; each group alphabetical; imports run
together and everything else separated by one blank line. Nothing enforced
it, so it held in the files written on a good day.

Two forces made the implementation less obvious than the specification.

**"Helper" has no positive definition.** The user's own phrasing was
"whatever is neither header, nor consts, nor tests, nor use clauses". A
helper is usually a `fn`, but the real files contain `struct`s, `impl`
blocks, type aliases and occasionally a `trait`. Any rule that enumerates
the item kinds it knows about will silently mis-file the first kind it has
not learned.

**Plain `//` comments never reach the syntax tree.** A comment explaining
why a test exists is, to `syn`, simply absent. Naively, the blank line above
the comment is then the gap before the test, and the comment block itself is
unexplained dead space — so every documented test in the workspace becomes a
spacing offence, and the rule punishes precisely the tests somebody took
care over.

## Decision

A test file is four sections in a fixed order —
`Imports < Constants < Helpers < Tests` — each alphabetical, with spacing as
part of the shape; and **`Helpers` is defined by exclusion**, as whatever is
neither an import, nor a constant, nor a test, so that the set of item kinds
is closed by construction.

The rule compares adjacent items only, and each pair is judged on three
independent questions: section order (a section index may never descend),
alphabetic order (within a section, case-insensitively), and spacing (the
number of blank lines between two entries of the same section — zero for
imports, one for everything else).

`TestFileParser` folds the contiguous run of `//` and `#[...]` lines above an
item back into that item, so a block's first line is the first line of its
comment, not of its `fn`.

## Forcing constraints / Evidence

**Exclusion is what closes the set.** `check_a_struct_helper_sorts_among_the_helper_functions`
and `check_an_impl_block_sorts_under_the_type_it_implements` both pass
without `Section` knowing what a struct or an impl is, and
`parse_classifies_a_struct_an_impl_and_a_type_alias_as_helpers` pins all
three at once. A kind of item nobody has thought of yet lands in `Helpers`,
which is where a reader would put it.

**An `impl` block has no name of its own**, so it borrows the name of the
type it implements. Without this it sorts as the empty string and drifts to
the top of the helper section — an offence the author cannot fix by editing
the file, because there is no name to move. Pinned by
`parse_names_an_impl_block_after_the_type_it_implements`.

**`is_test` matches the last path segment of the attribute**, so
`#[tokio::test]` is a test without the rule enumerating test frameworks.
Pinned by `parse_classifies_a_function_carrying_a_qualified_test_attribute_as_a_test`.

**The comment-folding case was found by dogfooding, not by reasoning.** Most
tests in this crate carry an explanatory comment; the rule reported a
spacing offence on nearly all of them the first time it ran against its own
tree. `parse_folds_a_leading_comment_into_the_item_below_it` and
`check_a_comment_introducing_a_test_is_part_of_that_test` now hold the fix
in place.

**Adopting the rule is harder than it looks**, which is itself evidence for
the comment-folding decision. Reordering this crate's own test files with a
throwaway script destroyed two file-level comment blocks: they sit above the
imports with a blank line between, so — unlike a comment above an item — no
item claims them, and a naive "attach comments downwards" pass discards
them. Caught only because a backup was taken first.

## Rejected alternatives

**Enumerate helper item kinds positively.** Rejected: a new kind of item
lands in whichever `match` arm is the default, silently and wrongly. The
exclusion definition has no default arm to be wrong.

**Report only the first offence per file**, as `001-ADR-HeaderRule` does.
Rejected: these are independent facts about different items, and somebody
fixing the file wants the whole list in one pass. The header rule's reason
for the opposite choice — that one missing header would emit one row per
header line and bury the rest of the workspace — has no analogue here.
Pinned by `check_reports_every_offence_in_a_file_rather_than_only_the_first`.

**Apply the rule to `src/` as well.** Rejected: source files have a
different shape, and this ordering is specific to how a test file is read.
Pinned by `check_a_file_outside_the_tests_tree_reports_nothing`.

**Apply it to `all_tests.rs` and `mod.rs`.** Rejected: those are registries,
not test files. Demanding a blank line between each `pub mod` entry would
make the one file whose entire job is to be scannable the hardest one in the
tree to scan. Their shape is [R003-ADR-TestsLayoutRule](R003-ADR-TestsLayoutRule.md)'s
business instead — which is the reason that rule exists as a separate rule
rather than as a branch inside this one.

**Sort case-sensitively.** Rejected: ASCII orders every uppercase letter
before every lowercase one, so a `struct Recorder` would be required to sit
above a `fn assert_state` — an order no human would produce by hand and
nobody would recognise as alphabetical. Pinned by `sort_key_ignores_case`.

**Compare every item against every other** rather than adjacent pairs.
Rejected: the offence a reader can act on is "this one is in the wrong
place", which is a statement about neighbours. A global comparison reports
the same disorder many times over.

## Consequences

Adding a test means *placing* it, not appending it. That is the point, and
it is also a real ongoing cost: renaming a test can make the file fail, and
a rename that moves a test past its neighbour requires moving the body too.

A file that does not parse reports nothing. `rustc` will say so far more
clearly, and inferring a shape from broken source would pile noise on top of
a compile error. Pinned by `check_a_file_that_does_not_parse_reports_nothing`.

Adopting the rule on an existing suite is a mechanical but not trivial edit,
for the reason recorded above — comment blocks belong to the item below
them, and a file-level comment block belongs to nobody. Any bulk reordering
of a real test suite should be done against a backup and diffed, not trusted.

**What this rule does not catch.** It judges *shape*, not content. The AAA
convention — `// Arrange`, `// Act`, `// Assert` inside a test body, and the
naming pattern `<method>_<description>_<outcome>` — is not checked here at
all, despite being the standard that motivated the tool. Both are plausible
future rules and neither is this one. The rule is also silent about a file
in `tests/` that mirrors no source file, which is `twin4rust`'s subject
rather than `stern4rust`'s.

## Enforcement

`tests/rules/test_file_structure_rule_tests.rs` — 26 tests covering all
three questions, the registry and non-test-tree exclusions, the
comment-introducing-a-test case, and the reporting contract.

The parser is tested separately from the rule, because most of the ways this
rule can be wrong are ways the parser can be wrong:
`tests/test_file_parser_tests.rs` (17 tests), `tests/section_tests.rs`, and
`tests/test_file_item_tests.rs`.

Dogfooded through the compiled binary in `run_stage_2.ps1`, which is what
caught this crate's own new test file breaking the rule while the rule for
`003-ADR-TestsLayoutRule` was being written.

## Related

- [R001-ADR-HeaderRule](R001-ADR-HeaderRule.md) — takes the opposite decision on
  offences-per-file, for a reason that does not apply here.
- [R003-ADR-TestsLayoutRule](R003-ADR-TestsLayoutRule.md) — owns the shape of the
  registry files this rule deliberately skips.
