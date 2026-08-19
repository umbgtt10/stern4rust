# ADR-FixOnlyWhatIsSafe

- **Status:** Accepted
- **Date:** 2026-08-19

## Context

Reordering test files by hand was done four times in one day and produced three
separate string-handling bugs: a script tracking raw strings but not ordinary
ones, which corrupted a file; one that missed `\n` escapes; and one
desynchronised by a `'"'` char literal. Every one was caught only because the
tool re-checked the result afterwards.

That is the argument for a fixer, and it is a good one. A rewriter working from
`syn` spans never reads the text: it moves whole line ranges, so a string
literal containing something that looks like Rust travels along like any other
line. What a text-munging script gets wrong is structurally impossible here.

The argument against is that a fixer edits code nobody reviewed, and a bad one
does so at scale and in silence.

## Decision

`--fix` repairs what can be repaired mechanically and then **reports what is
left**, unchanged, exactly as a run without `--fix` would. The checks run against
the repaired tree, so the report is the truth after fixing rather than before.

Only `test-file-structure` is fixable today: item order within a section,
section order, and blank lines. Everything else is reported and not touched.

Three constraints, each of which exists because the first version violated it.

**A fixer must never edit a file no rule governs.** The first version rewrote
**thirty `src/` files**, merging their grouped imports into one alphabetical
block. No rule checks `src/` import grouping, so no rule would have restored it,
and the run went green over a tree nobody had reviewed — a linter silently
reformatting a codebase it was only asked to inspect. The rewriter now mirrors
`test-file-structure`'s scope exactly and refuses everything else.

**A fixer must not write an order the formatter will undo.** The second version
sorted imports, and reordered `use serde_json::Value` above
`use serde_json::from_str` — the exact pair where `cargo fmt` writes the opposite
order and `test-file-structure` deliberately stands down. That is the
unsatisfiable loop the stand-down exists to prevent, recreated by the tool that
knows about it. Imports are now **moved as a block and never reordered**:
grouping them is safe, ordering them is rustfmt's business.

**A fixer must not lose content.** Trailing comments after the last item belong
to nobody, and dropping them would be the rewriter destroying what it was asked
to tidy. The preamble — header and file-level commentary — is never reordered
either.

The count of repaired files is stated in the report and in the summary as
`fixed=N`, beside what could not be repaired. A fixer reporting only its
successes would leave the reader believing the file is done.

## Forcing constraints / Evidence

Both bugs above were found by running `--fix` on this repository and reading the
diff — not by the test suite, which was green for both. The suite was green
because nothing tested the *blast radius*: every test asked "does it reorder
correctly", none asked "does it decline to touch what it should not". Those tests
exist now and are the two that matter most.

`git status` after the first run showed 30 modified files where 1 was expected.
That number is the whole reason this ADR is not simply "a fixer is safe because
it uses spans".

## Rejected alternatives

**Fix every rule.** Rejected for now. `imported-paths` looks mechanical and is
not: the turbofish form `from_str::<Value>(...)` already defeated one hand-rolled
attempt this week, and adding an import correctly means knowing where it sorts,
which is rustfmt's decision rather than this tool's.

**Ask the rule whether a file may be edited.** Rejected: a rule answers questions
about a file, and giving it a second job — authorising edits — would put the
safety of every fixer inside the thing being fixed. The rewriter carries its own
scope, and a test pins that the two agree.

**Fix silently and report only what remains.** Rejected: the reader has to know
their working tree changed. `fixed=N` and a named line say so.

**Write a backup file before each edit.** Rejected: this is a tool for a
repository under version control, and `git diff` is a better backup than
anything it could write. This is also why `--fix` does not ask for confirmation.

## Consequences

**`--fix` changes your working tree.** Deliberately, and only under a flag. It
does not stage, commit or ask; reviewing the diff is the user's job and git is
the safety net.

**Import order is not fixed**, only import grouping. Those offences remain in the
report after `--fix`, which is honest — `cargo fmt` handles them for the pairs it
agrees with the alphabet on, and the ones it disagrees on are stood down anyway.

**What this does not fix.** Everything except `test-file-structure`: a missing
header, a path-qualified call, a second implemented type, an undeclared module.
All of them still appear in the report, and the report after `--fix` is exactly
the report you would get without it, minus what was repaired.

**It cannot fix a file that does not parse**, which is `readable-source`'s
finding and correctly stays one.

## Related

- [R002-ADR-TestFileStructureRule](R002-ADR-TestFileStructureRule.md) — the one
  rule `--fix` can repair, and the stand-down the second bug walked into.
- [ADR-ExclusionsAreCounted](ADR-ExclusionsAreCounted.md) — the same principle
  applied to what a run declines to look at.
