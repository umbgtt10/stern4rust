# ADR-ExclusionsAreCounted

- **Status:** Accepted
- **Date:** 2026-08-19

## Context

Some trees cannot be moved. Vendored source, generated output, a fixture crate a
build script writes — a repository can be told to stop judging them, and until
now could not be.

The obvious shape is a `--exclude <glob>`, and the obvious implementation is to
prune the walk. That is exactly what this tool removed in `0.4.0` and for a
reason worth not repeating: the walker used to skip any directory holding its own
`Cargo.toml`, and a run reported `files_scanned=67` where the tree held 94 `.rs`
files with no line accounting for the other 27. Around 40 of the offences written
off as fixture noise turned out to be a repository's own integration tests.

A linter that quietly declines to look at part of a tree produces a clean report
that means nothing, and the reader has no way to tell. Adding a switch that
recreates that silence deliberately would be worse than not having the switch.

## Decision

`--exclude <glob>` is repeatable and matched against the **package-relative**
path, so a pattern can be written down in a repository and mean the same thing on
every machine that checks it out. Separators are normalised to `/` before
matching, because the walker hands back backslashes on Windows and an exclusion
that worked on one developer's machine and silently stopped working on another
would be the same failure in a new costume.

**Every pattern is named in the report with the number of files it removed**, and
the summary carries `files_excluded=N` beside `files_scanned=N`:

```text
  excluded: tests/** (31 files), nothing_here/** (0 files)
  matched nothing: nothing_here/** -- delete the pattern or correct it

summary: files_scanned=36 files_excluded=31 offences=8 ...
```

The JSON carries the same as `files_excluded` and an `exclusions` array.

**Exclusion happens after the walk, not by pruning it.** Pruning would be faster
on a large vendored tree and would cost the one thing that makes an exclusion
acceptable: a tree that is never entered cannot be counted. The walk is cheap
next to parsing every file it finds, so the trade is easy.

**A pattern matching nothing is called out by name.** This is the case a bare
total would hide. An exclusion naming a tree that has since moved or been deleted
goes on looking like it is doing work, and the run it silences is
indistinguishable from a run that had nothing to say. Naming it turns a dead
pattern into something a reader can delete.

**A path is attributed to the first pattern that covers it**, so two overlapping
patterns do not both claim the same file and inflate the total.

**An unusable pattern is an error**, exit `1`. Matching nothing is a legitimate
outcome and is reported as such; a glob that cannot be compiled is not, and a
gate whose exclude pattern has a typo must fail loudly rather than judge the tree
it was told to leave alone.

## Forcing constraints / Evidence

The case that originally motivated `--exclude` has since been closed by layout
instead. `grip4rust` and `crap4rust` both moved their fixture trees out of the
published package into a sibling `fixture/`, which is the better answer wherever
it is available — analysis input is input, and it does not belong inside the
package that ships.

That is why this arrived after the walker change rather than instead of it. Had
`--exclude` shipped first, both repositories would have papered over a layout
problem with a flag and the packages would still be shipping their fixtures.

## Rejected alternatives

**Prune the walk with `filter_entry`.** Rejected: the count is the point, and a
directory never entered cannot be counted. The performance argument does not
survive contact with the fact that this tool parses every file it keeps.

**A single `files_excluded` total without per-pattern counts.** Rejected: it
hides the stale pattern, which is the failure mode most likely to go unnoticed
for years.

**Silently ignore an uncompilable pattern.** Rejected for the same reason an
unknown `--rule` name is an error: a switch that quietly matches nothing looks
exactly like a switch that worked.

**Match against the absolute path.** Rejected: a pattern that has to know where
the repository sits on disk cannot be checked in, which defeats the purpose of
putting excludes in a config file.

## Consequences

**The summary line gained a field.** `files_excluded=N` sits between
`files_scanned` and `offences`, which is a change to text that gate scripts may
match on. It is always present, including as `files_excluded=0`.

**What this does not do.** It excludes files from being *judged*, not from being
*walked*, so an unreadable file inside an excluded tree is still never read —
correct, but it means exclusion cannot be used to work around a permissions
problem. It has no interaction with `--rule`: a file excluded from the run is
excluded from every rule, and there is no way to exclude a path from one rule
only.

**Excludes still have to be repeated at every invocation.** A `stern4rust.toml`
is their natural home and is the next thing to build.

## Related

- [ADR-WalkEveryFileInThePackage](ADR-WalkEveryFileInThePackage.md) — the skip
  this replaces, and why silence was the problem with it.
- [ADR-RuleSelection](ADR-RuleSelection.md) — the same principle applied to
  rules: a run that checked less than it appears to must say so.
