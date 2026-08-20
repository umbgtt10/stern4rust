# R011-ADR-DirectorySubfolderCountRule

- **Status:** Accepted
- **Date:** 2026-08-19

## Context

[R010](R010-ADR-DirectoryFileCountRule.md) caps files per directory, and the only
way to satisfy it is to create folders. A rule that creates folders and says
nothing about how many is half a rule: a directory with twenty subfolders is
exactly as unreadable as one with a hundred files, and it looks tidier while
being worse.

## Decision

A directory holds at most **5** subfolders that contain source, checked at every
level rather than only at the root, so pushing the sprawl one directory down does
not escape it. `max-subfolders-per-directory` in `stern4rust.toml` sets the
limit, for the same reason R010's is configurable: the number is taste.

A folder holding no `.rs` file anywhere beneath it does not count. The walker
never sees it, and a folder of documentation is nobody's module.

## Forcing constraints / Evidence

**It finds nothing.** Across eight repositories the deepest tree is two levels
and no directory has more than one subfolder containing source. It found zero
offences on the day it was written and will keep finding zero until somebody
restructures something.

That is stated plainly rather than buried, because a rule that has never fired
has not earned the reader's trust, and this file is the honest place to say so.

Two things make it worth having anyway.

**It is nearly free.** `PackageTree` already models directories, their files and
their children for R009 and R010. This rule is a length check on a collection
that was already being built — about ten lines on a walk that already happens.

**It is a counterweight rather than a discovery.** R010 will cause folders to be
created across five repositories. The moment that work starts, the cheapest way
to satisfy R010 is a folder per file, and this is the rule that says no. Its
value is not in what it finds today but in the shape it forbids while the work
R010 forces is being done.

Both arguments are about cost and timing rather than evidence. A rule justified
this way should be held to a lower confidence than one justified by a finding,
and if it still reports nothing in a year it should be reconsidered rather than
kept out of habit.

## Rejected alternatives

**Cap the depth instead of the count.** Genuinely arguable, and the better metric
in isolation — deep nesting is what actually hurts navigation, and it is one
number rather than two. Rejected because it does not do the job asked of it: a
depth cap permits a hundred sibling folders at level two, which is the sprawl
R010 would otherwise encourage. Depth remains a candidate for a later rule
alongside this one rather than instead of it.

**Fold it into R010 as a second threshold.** Rejected: `--rule` and `--skip`
work by name, so two thresholds under one name cannot be adopted or skipped
independently — and here that matters, because one half has ten offences across
the family and the other has none. A repository should be able to take the file
cap without the folder cap.

**Count every folder, including those without source.** Rejected: the walker
cannot see them, and counting things the tool does not read would make the
offence unverifiable from the report.

**Not building it at all.** Considered seriously, and the honest position until
R010 existed. What changed is that R010 makes folder creation imminent across
five repositories, and a guard is cheaper to have in place before that work than
to add after it.

## Consequences

**A rule that reports nothing is indistinguishable from a rule that is broken.**
The tests are the only evidence it works; there is no repository in which its
correctness can be observed. That is why its test file covers the boundary, one
past the boundary, and nesting one level down rather than only the obvious case.

**It shares R010's blind spot.** It counts folders, not what is in them: five
subfolders of two hundred files each satisfy it, and R010 is what catches that.
The two are only meaningful together.

**It cannot be autofixed** for the same reason R010 cannot — the correction is a
`git mv` and a decision about grouping.

## Enforcement

`tests/rules/layout/directory_subfolder_count_rule_tests.rs` — 5 tests covering the
boundary, one past it, checking at a nested level rather than only the root, and
that a folder with no source does not count.

## Related

- [R010-ADR-DirectoryFileCountRule](R010-ADR-DirectoryFileCountRule.md) — the
  rule this one balances, and the one that will create the folders it counts.
