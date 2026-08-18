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

> **Status: scaffold.** The packaging, CLI and test harness are in place and the
> crate publishes. No rule is implemented yet, so a run reports that and exits 0.

## Install

```bash
cargo install cargo-stern4rust
```

## Use

```bash
cargo stern4rust
cargo stern4rust --package my-crate
cargo stern4rust --manifest-path path/to/Cargo.toml --package a --package b
```

| flag | meaning |
|---|---|
| `--manifest-path <PATH>` | workspace manifest to analyse; defaults to the one in the current directory |
| `--package <NAME>` | restrict to these packages; repeatable. Omit to take the manifest's own package |

## The rules

Each rule is independent, names itself in the report, and can be pointed at a
whole workspace or one package. The set is open — these are the first two.

### AAA structure in tests

A test reads as three movements: set the world up, do the one thing, check what
happened. When the marker comments are missing, the boundary between them stops
being visible, and a test that quietly asserts in its arrange section or acts
twice looks exactly like one that does not.

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

Two contractions are part of the rule rather than exceptions to it:

- `// Arrange & Act` when there is nothing to set up separately
- `// Act & Assert` when the call and the check are one expression

### At most one struct with an impl block per file

A file holds one behaviour-bearing type. Plain data declarations sitting beside
it are fine; a second type that carries methods is a second subject, and a file
with two subjects has no name that describes it.

This is what keeps a mirrored test file meaningful — `src/foo.rs` answering to
`tests/foo_tests.rs` only says something when `foo.rs` has one subject to test.

## Output

Clean:

```text
stern4rust report

All rules are satisfied.
```

Broken:

```text
stern4rust report

file                          line  rule              offence
----------------------------  ----  ----------------  ----------------------------------
src/peer_registry.rs            41  one-struct-file   second struct with an impl block
tests/peer_registry_tests.rs    18  aaa-structure     no Act section
tests/peer_registry_tests.rs    57  aaa-structure     Assert before Act

summary: files_scanned=42 offences=3 rules_broken=2
```

## Exit codes

The family shares one contract, so a wrapper script can tell the two apart:

| code | meaning |
|---|---|
| `0` | every rule satisfied |
| `1` | the tool could not run — bad manifest, unknown package, unreadable source |
| `2` | at least one rule was broken |

Only `2` is a finding. A script that treats every non-zero code the same cannot
distinguish "your code has a problem" from "I could not look at your code",
which is the difference between a real failure and a broken CI step.

## Licence

MIT — see [LICENSE](LICENSE).
