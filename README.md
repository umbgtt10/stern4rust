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

> **Status: five rules, more coming.** The set below is what is implemented and
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
| `--header-file <PATH>` | the header every `.rs` file must open with. Without it the header rule does not run, and the report says which rules did |
| `--format <text\|json>` | `text` (default) is the table below; `json` is the same run as a document |
| `--offence-threshold <N>` | how many offences the report prints. Default `100`, `0` for all. The cap is on what is *shown*, never on what is counted |

A directory holding its own `Cargo.toml` is not walked: it is a different
package, its files are that package's to answer for, and cargo would not
compile them as part of this one either. The shape this matters for is a
fixture crate under `tests/fixtures/` — sample code a tool analyses rather than
code it ships.

## The rules

Each rule is independent, names itself in the report, and can be pointed at a
whole workspace or one package. Every one has an ADR recording why it exists,
what was rejected, and — the part that matters for a checking tool — what it
does **not** catch.

### `readable-source`

Every `.rs` file can be read and parsed. This one exists because silence is
indistinguishable from success: a corrupted file produces no rows, and a file
with no rows looks exactly like a clean file. See
[004](docs/ADRs/004-ADR-ReadableSourceRule.md) for the incident that forced it.

### `header`

Every `.rs` file opens with the repository's header, supplied by
`--header-file` because it is never the same twice — MIT here, Apache 2.0 in a
sibling repository, a different year again next year. The offence carries the
whole correct header, not only the line that diverged, so a fix is one pass
rather than a loop. [001](docs/ADRs/001-ADR-HeaderRule.md)

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
[002](docs/ADRs/002-ADR-TestFileStructureRule.md)

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
compiled cannot fail. [003](docs/ADRs/003-ADR-TestsLayoutRule.md)

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

### `--format json`

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

## Design decisions

Every rule has an ADR in [`docs/ADRs`](docs/ADRs/README.md) — one per rule,
because a rule is the unit a reader argues with when it fails their build. Each
records what the rule requires, what was rejected on the way there, and — the
part that matters most for a checking tool — what it does **not** catch.

## Licence

MIT — see [LICENSE](LICENSE).
