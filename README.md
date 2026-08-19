# stern4rust

A cargo subcommand that holds a Rust workspace to its house coding rules and
names every file that breaks one.

It is in the same family as
[`cargo-crap4rust`](https://crates.io/crates/cargo-crap4rust),
[`cargo-twin4rust`](https://crates.io/crates/cargo-twin4rust) and
[`cargo-iceberg4rust`](https://crates.io/crates/cargo-iceberg4rust), and shares
their character: it measures what it can state precisely, it is opinionated
about what good looks like, and it fails the build rather than warning.

The rules it checks are conventions, not correctness. `rustc` and `clippy`
already own correctness. These are the things a reviewer would otherwise have to
say by hand, every time, forever — and the ones that quietly stop being true
across a codebase the moment nobody is checking.

> **Status: fourteen rules, more coming.** The set below is what is implemented and
> gated. `stern4rust` runs against its own tree on every build, so a rule that
> would fail this repository cannot be merged into it.

## Install

```bash
cargo install cargo-stern4rust
```

## Use

```bash
cargo stern4rust
cargo stern4rust --header-file docs/header.txt
cargo stern4rust --manifest-path path/to/Cargo.toml --package a --package b
cargo stern4rust --format json
```

| flag | meaning |
|---|---|
| `--manifest-path <PATH>` | workspace manifest to analyse; defaults to the one in the current directory |
| `--package <NAME>` | restrict to these packages; repeatable. Omit to take the manifest's own package |
| `--header-file <PATH>` | the header every `.rs` file must open with. Without it the header rule does not run, and the report names it as not applied |
| `--format <text\|json>` | `text` (default) is the table below; `json` is the same run as a document |
| `--offence-threshold <N>` | how many offences the report prints. Default `100`, `0` for all. The cap is on what is *shown*, never on what is counted |
| `--rule <NAME>` | apply only these rules; repeatable. Omit to apply every rule |
| `--skip <NAME>` | do not apply these rules; repeatable. Subtracted from whatever `--rule` selected |
| `--fix` | repair what can be repaired mechanically, then report what is left. Only `test-file-structure` is fixable today |
| `--baseline <PATH>` | offences recorded here are not reported and do not fail the run; the suppressed count is always in the summary |
| `--write-baseline` | record the current offences and exit clean, instead of judging against them |
| `--exclude <GLOB>` | keep these paths out of the run; repeatable, matched against the package-relative path. Every pattern is named in the report with how many files it removed, including zero |

Only `target/` and `.git/` are skipped by default. Everything else under the
package is judged, including a nested package with its own `Cargo.toml` — a
linter that quietly declines to look at part of a tree is the silence this tool
exists to refuse. Sample code a tool analyses belongs beside the package rather
than inside it; see
[ADR-WalkEveryFileInThePackage](docs/ADRs/ADR-WalkEveryFileInThePackage.md).

`--exclude` covers the tree that genuinely cannot move, and is not a silent
skip: every pattern is named in the report with the files it removed, and one
that matched **nothing** is called out so a dead exclusion can be deleted rather
than trusted.

```text
  excluded: tests/** (31 files), nothing_here/** (0 files)
  matched nothing: nothing_here/** -- delete the pattern or correct it

summary: files_scanned=36 files_excluded=31 offences=8 ...
```

See [ADR-ExclusionsAreCounted](docs/ADRs/ADR-ExclusionsAreCounted.md).

## `--fix`

```bash
cargo stern4rust --fix
```

Repairs `test-file-structure` offences — item order, section order, blank lines
— and then reports everything it could not repair, unchanged. The report after
`--fix` is exactly the report you would get without it, minus what was fixed.

```text
  fixed: 12 file(s) rewritten

summary: files_scanned=103 files_excluded=0 offences=4 baselined=0 fixed=12 ...
```

It works from `syn` spans and never reads the text, so a string literal
containing something that looks like Rust is moved like any other line. It only
touches files `test-file-structure` governs, it never reorders imports — that is
rustfmt's — and it changes your working tree, so read the diff.

## Adopting it on a codebase that has never run it

`--rule` lets you enforce one rule today and the next when it is green. What it
cannot express is *every* rule against **new** code while tolerating what is
already there — which for a codebase with hundreds of existing offences is the
difference between a gate that fails forever and no gate at all.

```bash
cargo stern4rust --write-baseline    # record what is there today
cargo stern4rust                     # from now on, only new offences fail
```

The baseline is keyed on the file, the rule and the description — **not the
line** — so an offence that moves because somebody added an import above it is
still the same offence. Counts are recorded rather than a set, so fixing one of
two identical offences and adding another still passes, while adding a third
does not.

**Nothing is hidden quietly.** Every run that used a baseline names it and says
how much it suppressed, and an entry that no longer matches anything is called
out so the file can be refreshed rather than trusted:

```text
  baseline: stern4rust-baseline.json (417 suppressed)
  1 baseline entries matched nothing -- rerun with --write-baseline to refresh it

summary: files_scanned=312 files_excluded=0 offences=2 baselined=417 rules_broken=1 ...
```

## `stern4rust.toml`

Every switch above can live beside the manifest instead of on the command line,
which is what lets a gate script, a pre-commit hook and a developer's terminal
run the same check:

```toml
baseline = "stern4rust-baseline.json"
max-files-per-directory = 20
max-subfolders-per-directory = 5
header-file = "docs/header.txt"
offence-threshold = 100
rules = ["header", "tests-layout"]
skip = ["test-file-structure"]
exclude = ["vendor/**"]
```

Every key is optional. **The command line wins, per setting** — override one for
one run without restating the rest. For the list settings that is replacement
rather than merging, because `--rule header` meaning "header *plus* whatever the
file selected" would be the opposite of what naming one rule means everywhere
else here.

An unknown key is an error, not a silently ignored line: a misspelled `exclude`
that quietly did nothing would look exactly like one that worked. A file that
exists and cannot be parsed is an error too — it was written on purpose, and
running as though it were absent would apply a configuration nobody chose.

**The report names the file it used**, so the switches in force are never
invisible:

```text
  config: /path/to/stern4rust.toml
```

## The rules

Each rule is independent, names itself in the report, and can be pointed at a
whole workspace or one package. Every one has an ADR recording why it exists,
what was rejected, and — the part that matters for a checking tool — what it
does **not** catch.

### `readable-source`

Every `.rs` file can be read and parsed. This one exists because silence is
indistinguishable from success: a corrupted file produces no rows, and a file
with no rows looks exactly like a clean file. See
[R004](docs/ADRs/R004-ADR-ReadableSourceRule.md) for the incident that forced it.

### `header`

Every `.rs` file opens with the repository's header, supplied by
`--header-file` because it is never the same twice — MIT here, Apache 2.0 in a
sibling repository, a different year again next year. The offence carries the
whole correct header, not only the line that diverged, so a fix is one pass
rather than a loop. [R001](docs/ADRs/R001-ADR-HeaderRule.md)

### `imported-paths`

A function is called through a name this file imported, not through a path. A
file's `use` statements are its list of dependencies, and
`syn::parse_file(...)` is a dependency that never reaches that list — the file
compiles with nothing in it mentioning `syn`.

One imported segment is the point rather than an exception: `use std::fs;`
followed by `fs::read_to_string(...)` names the route once at the top and still
says at the call site where the function came from. Type qualifiers —
`Widget::new()`, `Self::inner()` — are left alone. The correction names the
import to add:

```text
src/finding/registry_parser.rs  24  imported-paths  `syn::parse_file` is reached through a path; no import of this file names it
                                             fix: add `use syn::parse_file;` and call `parse_file`
src/settings/args.rs     58  imported-paths  `std::env::args` is reached through a path; no import of this file names it
                                             fix: add `use std::env;` and call `env::args`
```

[R008](docs/ADRs/R008-ADR-ImportedPathsRule.md)

### `directory-file-count`

A directory holds at most **20** `.rs` files, not counting its own index —
`max-files-per-directory` in `stern4rust.toml` changes it. This is the only rule
whose number is taste rather than fact, which is why it is configuration.

It is also the rule most in tension with the others: one struct per file, one
implemented type per file and one test file per source file all manufacture files
by design, so the limit has to be generous enough that the conventions producing
the files are not themselves the offence.

**It cannot be autofixed** — the correction is a `git mv`, a new `mod.rs`, a
declaration in the parent, and a matching move under `tests/`.
[R010](docs/ADRs/R010-ADR-DirectoryFileCountRule.md)

### `directory-subfolder-count`

At most **5** subfolders per directory, checked at every level. The counterweight
to the rule above: that one creates folders, and without this the cheapest way to
satisfy it is a folder per file. It finds nothing across the family today and
says so.
[R011](docs/ADRs/R011-ADR-DirectorySubfolderCountRule.md)

### `registry-completeness`

A registry declares every module beside it, so nothing in the tree goes
uncompiled. A `mod.rs` that is valid and simply fails to mention `alpha_tests`
leaves `alpha_tests.rs` uncompiled — it still exists, still looks like a test
file, and nothing runs it.

Only the silent direction is checked. `pub mod missing;` with no `missing.rs` is
a compile error `rustc` already reports; an orphan `.rs` file produces no error
and no warning at all.

On its first run it found **8 never-compiled test files** in a published sibling
tool — about thirty tests that had never once executed.
[R009](docs/ADRs/R009-ADR-RegistryCompletenessRule.md)

### `module-registry`

A `mod.rs` or `lib.rs` is a registry and holds nothing else: the header, the
crate-level attributes, `extern crate alloc`, and `pub mod` declarations. A type
defined in a registry has no file named after it.
[R006](docs/ADRs/R006-ADR-ModuleRegistryRule.md)

### `single-implemented-type`

A source file outside `tests/` holds at most one type that carries behaviour —
one `struct` or `enum` that is both declared in the file and has an `impl`
block. Structs and enums *without* `impl` blocks are unlimited, so the payload
types a subject needs stay beside it. The offence names the type to move and the
file to move it to.
[R007](docs/ADRs/R007-ADR-SingleImplementedTypeRule.md)

### `pure-traits`

A trait declares; it does not implement. No method in a `trait` declaration in
`src/` may have a default body.

A default reads as a convenience and works as a decision nobody made: an
implementor that says nothing about a method is indistinguishable from one that
considered it and found the default right. Removing the body makes every
implementor answer in its own file.

The other half — that every implementor implements every method — needs no rule
here. With no default to fall back on, `rustc` rejects an incomplete impl with
`E0046`, so only the half the compiler is silent about is checked.

Associated types and associated constants may still carry defaults; neither is
behaviour. `tests/` is exempt, where a trait with a body is a deliberate fake.
[R014](docs/ADRs/R014-ADR-PureTraitsRule.md)

### `test-file-structure`

A test file reads top to bottom in one order: header, imports, constants,
helpers, tests. Each group is alphabetical; imports run together and everything
else is separated by exactly one blank line.

```rust
#[test]
fn poll_with_an_empty_queue_returns_nothing() {
    // Arrange
    let queue = Queue::new();

    // Act
    let polled = queue.poll();

    // Assert
    assert!(polled.is_none());
}
```

Order is what makes a test file skimmable without reading it. Once a constant
sits below a helper, the file has no shape and every later addition goes
wherever the last one happened to end.
[R002](docs/ADRs/R002-ADR-TestFileStructureRule.md)

### `test-free-source`

Tests live in `tests/`, and the production source tree carries none of them. A
`#[cfg(test)] mod tests` inside `src/` is invisible to everything else here: it
is not the mirrored test file `twin4rust` looks for, it is not declared from
`all_tests.rs`, it has no required shape, and it is compiled under a
configuration the shipped build never uses.

`#[cfg_attr(test, ...)]` is caught for the same reason: a type carrying a derive
only under test means one thing to the tests and another to the shipped build.

The line is `test`, not conditional compilation. `#[cfg(feature = "...")]` and
`#[cfg_attr(feature = "serde", derive(Serialize))]` are ordinary library work
and are left alone — a feature is selectable by the shipped build, so what is
tested is what somebody runs. `test` is the one predicate no shipped build ever
sets. [R005](docs/ADRs/R005-ADR-TestFreeSourceRule.md)

### `tests-layout`

A tests folder is reached through exactly one door: `tests/all_tests.rs`, plus a
`mod.rs` in every subfolder on the way down. Miss one and the files beneath it
are never compiled — they still exist, still look like tests, and nothing runs
them. The failure is silent by construction, because a test that is never
compiled cannot fail. [R003](docs/ADRs/R003-ADR-TestsLayoutRule.md)

## Output

Clean:

```text
stern4rust report

All rules are satisfied.
```

Broken — grouped by file, then by line, each offence followed by what to do
about it:

```text
stern4rust report

file                     line  rule          offence
-----------------------  ----  ------------  ------------------------------------------------------
tests/all_tests.rs          1  tests-layout  the import `use std::fmt;` does not belong in a registry
                                             fix: move the import `use std::fmt;` out of the registry into the file that needs it
tests/all_tests.rs          3  tests-layout  the constant `LIMIT` does not belong in a registry
                                             fix: move the constant `LIMIT` out of the registry into the file that needs it
tests/rules/deep/mod.rs     1  tests-layout  a tests subfolder has no mod.rs, so nothing in it is compiled
                                             fix: create tests/rules/deep/mod.rs with the header and one `pub mod` line per file in that folder

summary: files_scanned=4 offences=3 rules_broken=1
```

Two things every offence carries. It **names the thing it is about** — "something
in this file is not a declaration" is true of the whole file and actionable
nowhere in it. And it carries a **correction**: what to do, not only what is
wrong. That field is required rather than optional, so a rule that can say what
is broken has to say how to fix it.

### `--offence-threshold`

A first run against a large codebase can find a thousand offences, and a
thousand rows is not a report — it is a wall that gets scrolled past. The
report prints the first 100 by default:

```text
... and 47 more offences not shown. Raise --offence-threshold (currently 100, use 0 for all) to see them.

summary: files_scanned=312 offences=147 rules_broken=4
```

**The cap is on what is shown and never on what is counted.** `offences=147`
is the true total, the omitted count is stated outright, and the exit code is
decided from every offence rather than from the printed ones — capping to 1
still exits `2` if there are 200. A report that quietly said 100 when the tree
held 147 would be the precise failure this tool exists to catch, and it would
be this tool committing it.

Because offences are sorted by file then line, what survives the cap is whole
files from the top rather than a scattering across the tree: fix what is shown,
re-run, get the next file.

### Adopting it on an existing codebase

A first run against a codebase that has never been checked will find hundreds of
offences, and a gate nobody can turn on is a gate nobody turns on. `--rule`
narrows the run to what you are ready to enforce:

```bash
cargo stern4rust --header-file docs/header.txt --rule header
```

Measured against `braintax4rust`, which has never been through this tool:

| run | offences |
|---|---:|
| all eight rules | 204 |
| `--rule imported-paths` | **50** |

204 is a wall; 50 is an afternoon. Enforce one rule today, add the next when the
first is green.

`--skip` is the other direction — everything except the one that is noisy for
you. Skipping wins over selecting, so a rule named in both is not applied.

**A run that did not apply every rule never claims otherwise.** Every report
names the rules it applied, and names the ones it did not along with why:

```text
All applied rules are satisfied.

  applied: readable-source, test-file-structure, test-free-source, tests-layout
  not applied: header (needs --header-file)

summary: files_scanned=61 offences=0 rules_broken=0 rules_applied=4 rules_skipped=0 rules_unconfigured=1
```

Three states, not two. A rule you turned off with `--skip` is *skipped*; a rule
that could not run because you did not pass `--header-file` is *unconfigured*.
Calling the second one skipped would blame you for a choice you did not make;
calling it nothing at all would let the run check less than it appears to. The
JSON carries `rules_applied`, `rules_skipped` and `rules_unconfigured`.

An unknown rule name is an error, not a switch that quietly matches nothing —
as is `--rule header` without `--header-file`, which would otherwise apply no
rules at all. Both exit `1`.

## `--format json`

The table is sized to its contents and meant for a person; nothing can parse it
reliably, since paths and descriptions both contain spaces. `--format json`
renders the same run as a document:

```json
{
  "files_scanned": 4,
  "offences_found": 3,
  "rules_broken": 1,
  "offences": [
    {
      "file": "tests/all_tests.rs",
      "line": 3,
      "rule": "tests-layout",
      "description": "the constant `LIMIT` does not belong in a registry, ...",
      "correction": "move the constant `LIMIT` out of the registry into the file that needs it",
      "subject": "the constant `LIMIT`",
      "expected": null
    }
  ]
}
```

`correction` is what to do about the offence, and is always present.
`subject` is the thing the offence is about. `expected` is the correct text
where the rule knows it — the header rule puts the entire header there, so a
consumer applies the fix in one pass instead of re-running to find the next
wrong line. Every key is always present, so the shape does not vary.

## Exit codes

The family shares one contract, so a wrapper script can tell the two apart:

| code | meaning |
|---|---|
| `0` | every rule satisfied |
| `1` | the tool could not run — bad manifest, unknown package |
| `2` | at least one rule was broken |

Only `2` is a finding. A script that treats every non-zero code the same cannot
distinguish "your code has a problem" from "I could not look at your code",
which is the difference between a real failure and a broken CI step.

The line between `1` and `2` is whether the work can still be enumerated. A bad
manifest leaves no list of files to judge and is a `1`. A single unreadable file
is a `2`, reported against `readable-source` like any other finding — it is a
fact about the tree, and aborting on it would discard every offence already
found in every other file.

## Documentation

| document | what it is for |
|---|---|
| [RULES.md](docs/RULES.md) | what each rule requires, every offence it can emit, and what it does **not** catch |
| [ARCHITECTURE.md](docs/ARCHITECTURE.md) | the pipeline, the components, and how to add a rule |
| [ADRs/](docs/ADRs/README.md) | the load-bearing decisions. `R<NNN>-ADR-` for a rule, unnumbered `ADR-` for the machinery |
| [IMPLEMENTED-FEATURES.md](docs/IMPLEMENTED-FEATURES.md) | what is built today |
| [OPEN_POINTS.md](docs/OPEN_POINTS.md) | known gaps in what is built, sharpest first |
| [ROADMAP.md](docs/ROADMAP.md) | where this is going, and what it will not do |

There is no `FORMULA.md`, unlike `crap4rust`, `iceberg4rust` and `grip4rust`.
Nothing here is scored: a rule is satisfied or it is not.

Every rule has an ADR, because a rule is the unit a reader argues with when it
fails their build. Each records what the rule requires, what was rejected on the
way there, and — the part that matters most for a checking tool — what it does
**not** catch.

## Licence

MIT — see [LICENSE](LICENSE).
