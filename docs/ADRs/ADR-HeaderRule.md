# ADR-HeaderRule

- **Status:** Accepted
- **Date:** 2026-08-18

## Context

Every `.rs` file is required to open with the repository's copyright and
licence header. Nothing in the Rust toolchain checks this: `rustfmt` does
not, `clippy` does not, and `cargo` has no opinion. The result is the
characteristic decay pattern — a young repository is at 100%, and every file
added under time pressure afterwards is a coin flip. The gap is invisible
until somebody audits the licensing of a published crate, which is exactly
when it is expensive.

Two forces pull against each other.

The expected text is **not the same twice**. This repository is MIT; the
`etheram` repositories are Apache 2.0; the year moves; another codebase will
have something else entirely, possibly with a company name or an SPDX
expression this author has never seen. A rule that hardcoded one header
would be correct for exactly one repository and useless everywhere else.

The comparison must be **exact and yet survive a checkout**. A wrong year or
a swapped licence line is precisely the failure worth failing a build over,
so the match cannot be fuzzy. But three things differ between one working
copy and another without anybody having edited anything: git rewrites line
endings on checkout, so a byte-for-byte comparison fails on every line of
every file on a Windows machine; a byte order mark is invisible in an editor
and sits in front of the first character of line 1; and editors add a
trailing newline to the header file itself.

## Decision

The expected header is **data**, not a constant: `HeaderRule::new(Vec<String>)`,
loaded from the file named by `--header-file`. A file satisfies the rule when
its first N lines equal those N lines exactly, after normalisation, and
exactly one offence — **the first divergence** — is reported per file.

Three sub-decisions fall out of that sentence:

**Normalisation happens once, in `SourceFile`, not in the rule.** A leading
BOM is stripped, `\r` is stripped from line endings, and the path is
forward-slashed, before any rule ever sees the file. `HeaderSource` does the
same for the header file and additionally drops trailing blank lines.
No rule has to remember to do this, and no future rule can forget.

**The overlap is compared before the length.** A file shorter than the header
is only reported as "too short" once its lines have all matched. Checking
length first would tell a file with no header at all that it was too short —
true, and useless, when the actionable fact is that line 1 is not the header.

**An empty expected header means no opinion.** Without `--header-file` the
rule is not registered at all, rather than registered and vacuously passing.

## Forcing constraints / Evidence

The normalisation cases are not hypothetical, and each is pinned:
`check_a_file_saved_with_windows_line_endings_reports_nothing`,
`check_a_file_carrying_a_byte_order_mark_reports_nothing`, and
`header_source_tests::parse_drops_the_trailing_newline_every_editor_adds`.
Without the first, the rule fails every file in this repository on the
machine it was written on.

The ordering of the overlap check against the length check is forced by a
pair of tests that differ only by a trailing newline and must produce
different reports.
`check_a_file_that_ends_before_the_header_does_reports_its_length` uses a
file with **no** trailing newline — the only way to reach the too-short
report — while
`check_a_file_whose_second_line_is_blank_reports_the_divergence_not_the_length`
adds one, which gives the file a second line that is blank where the licence
should be. That is a divergence, and is reported as one. Swapping the two
checks makes the second test report the wrong thing.

## Rejected alternatives

**Hardcode the header as a constant in the rule.** Rejected: correct for one
repository. The tool would have to be forked, or grow a `match` on
repository name, to be used anywhere else.

**Report every diverging line.** Rejected: a file with no header at all
produces one offence per header line — three rows for this repository's
header, more for a longer one — burying every other file in the workspace
behind a single offender. Pinned by `check_reports_only_the_first_divergence`.
This is the opposite of the choice made in `ADR-TestFileStructureRule`, and
deliberately so: there the offences are independent facts about different
items, and a reader fixing the file wants all of them at once.

**Find the header anywhere in the file rather than at the top.** Rejected:
the header is the *opening* of the file, and a licence buried below an
`impl` block does not do the job a licence header exists to do. Pinned by
`check_a_file_carrying_the_header_below_other_code_reports_the_first_line`.

**Compare with whitespace or case normalised away.** Rejected: the failure
this rule exists to catch — a wrong licence — differs from the correct text
by ordinary words in ordinary case. A comparison loose enough to be
forgiving is loose enough to miss the thing.

**Allow a prologue above the header** (an inner attribute, a `#![no_std]`,
a `#[rustfmt::skip]`). Rejected for now, as an unproven need — see
Consequences.

## Consequences

Anything that must syntactically precede the header is an offence. Inner
attributes such as `#![no_std]` are the realistic case: a crate root that
needs one must place it *below* the header, which is legal Rust and is what
this repository and the `etheram` repositories already do. No case has yet
arisen that cannot. If one does, it forces a revision here rather than a
quiet exception in the code.

The tool grows a required flag for one of its rules. A run without
`--header-file` still applies every rule that needs no configuration, and
the report says which rules were applied — so a clean run cannot be
mistaken for "the header rule passed" when the header rule never ran.

The header file is a per-repository artifact that has to be kept in step
with the headers themselves. This repository keeps it at `docs/header.txt`
and dogfoods it, so the two cannot drift apart without the build going red.

**What this rule does not catch.** It compares text and nothing else. A file
carrying a perfectly-formatted header naming the wrong copyright holder, or
an SPDX identifier that does not match the `license` field in `Cargo.toml`,
passes — the rule has no way to know which of the two is wrong. Cross-checking
the header against the manifest is a plausible future rule and is not this
one.

## Enforcement

`tests/rules/header_rule_tests.rs` — 15 tests covering exact match, wrong
year, wrong licence, header below other code, empty file, file shorter than
the header, both normalisation cases, and the empty-expected-header no-op.

Beyond the unit tests, the rule is dogfooded end to end:
`runner_tests::run_against_this_crate_with_its_own_header_is_clean` runs the
whole tool against this crate with `docs/header.txt` and requires a clean
report, and `run_stage_2.ps1` does the same through the compiled binary. A
new file added here without a header fails this crate's own build.

## Related

- [ADR-TestFileStructureRule](ADR-TestFileStructureRule.md) — takes the
  opposite decision on how many offences to report per file, for a reason
  spelled out there.
- [ADR-TestsLayoutRule](ADR-TestsLayoutRule.md) — the rule that widened the
  `Rule` trait this one is built on.
