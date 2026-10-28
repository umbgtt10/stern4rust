# R039-ADR-DistinctDevDependenciesRule

- **Status:** Accepted
- **Date:** 2026-10-26

## Context

A package's tests can use every crate its `[dependencies]` names. A
`[dev-dependencies]` entry naming one of those again is, most of the time, a line
that does nothing: the tests had the crate already. Nothing says so. `cargo`
accepts the repeat without a warning, and `cargo-shear`, which gates unused and
misplaced dependencies across the family, looks for unused ones, not repeated
ones -- this repository's own stage 2 ran it green for months with `clap` in
both tables of its core.

The shape is common because it is left behind rather than written. iceberg4rust's
core repeated `clap`, `serde_json` and `syn`; splitting the core took the first two
away with the code that used them and left `syn`, which a reader noticed by hand.
Every repeat found was the same workspace spec in both tables.

Not every repeat is redundant, and that is what the rule has to tell apart. A
dev-dependency is the only way cargo has of giving the tests a feature
production must not ship -- slotgate's core takes `tokio` plain and repeats it in
`[dev-dependencies]` with `features = ["test-util"]`, the paused clock its tests
run on. It is also how the tests get a crate that is optional in
`[dependencies]`, or the default features a dependency turns off.

## Decision

**A dev-dependency adds to the dependencies.** A `[dev-dependencies]` entry that
repeats a `[dependencies]` entry of the same manifest and adds nothing to it is
reported once, against the manifest, with the correction to remove it.

**Repeating** is decided by `ManifestDependency::repeats`, on the TOML as
written, because resolution erases exactly the difference that matters:

- **The same key.** Cargo refuses one package under two keys ("depends on crate
  `foo` multiple times with different names") and one key for two sources
  ("Dependency 'foo' has different source paths depending on the build target"),
  so a repeat always shares the name. A renamed dependency --
  `foo = { package = "bar" }` -- is matched on its key like any other, and needs
  nothing of its own.
- **A target the dependency covers.** A `[target.'cfg(unix)'.dev-dependencies]`
  entry repeats `[dependencies]`, which covers every target, and
  `[target.'cfg(unix)'.dependencies]`; the target is compared as written. An
  untargeted dev entry beside a targeted dependency gives the tests the crate on
  every other target, and is left alone.
- **Written alike.** Both taken from the workspace, or neither, and one version
  requirement. One inheriting and one spelled out compares an entry against a
  workspace table this manifest does not hold; in a workspace,
  `workspace-dependencies` reports the spelled-out one, and once it is fixed the
  pair is judged.

**Adding nothing** means: the dependency is not optional, the dev entry keeps no
default features the dependency turns off, and it lists no feature the dependency
does not. Turning default features *off* in the dev entry adds nothing, since
features unify.

**Read from the TOML, in every package, workspace or not.** `DependencyTables`
reads one manifest -- the three plain tables and each under `[target.<cfg>]` --
and `ManifestResolver::declared_dependencies` every package; each is handed its
own manifest's entries, the per-package filter `workspace-dependencies` already
uses. Unlike that rule, this one has a subject outside a workspace: a single
crate repeats a dependency as easily as a member.

| offence | correction |
|---|---|
| `` core/Cargo.toml declares `clap` in [dev-dependencies] as well as in [dependencies], and adds nothing to it `` | remove `clap` from [dev-dependencies]: the tests already have it from [dependencies] |

Line 1 of the manifest, as `workspace-dependencies` reports: the line number is
not part of an offence's fingerprint, and the edit is one line deleted.

## Forcing constraints / Evidence

What cargo accepts, probed with path crates and `cargo metadata --offline`:

| shape | cargo |
|---|---|
| the same entry in both tables | accepted |
| the same key, another source or another package | rejected: different source paths |
| the same package under another key | rejected: multiple times with different names |
| a dev entry `optional = true` | rejected: dev-dependencies are not allowed to be optional |
| the dependency optional, the dev entry not | accepted |
| the dev entry adding a feature | accepted |
| `[target.'cfg(unix)'.dependencies]` beside an untargeted dev entry | accepted |
| `[dependencies]` beside `[target.'cfg(unix)'.dev-dependencies]` | accepted |

The family, 2026-10-26, every member manifest of ten repositories:

| repository | entry | verdict |
|---|---|---|
| twin4rust | core `clap`, `syn` -- `{ workspace = true }` in both | reported |
| dry4rust | core `clap` -- `{ workspace = true }` in both | reported |
| stern4rust | core `clap` -- `{ workspace = true }` in both | removed before the rule landed |
| slotgate | core `tokio`, the dev entry adding `test-util` | left alone |
| braintax4rust, grip4rust, crap4rust, iceberg4rust, sentinel4rust, etheram-core | -- | nothing |

None of the ten declares a target-specific dependency table.

## Rejected alternatives

**Extend `workspace-dependencies`.** Rejected: it asks where a dependency is
declared, this whether a dev entry is needed. It stands down outside a workspace,
where this must not, and a repository that deliberately pins per member skips it
while still wanting this -- `skip` works per rule name.

**Report every repeat.** Rejected: a repeat adding a feature, default features,
or an optional crate is the only way cargo gives the tests it, so the correction
would lose something. slotgate's `test-util` would have been reported with no
edit able to clear it.

**Compare a pair written unlike by reading the workspace's entry.** Rejected for
now: the root's `[workspace.dependencies]` would decide what an inherited entry
enables, more reading for a shape no repository has, and one that
`workspace-dependencies` already reports in a workspace.

**Ignore the version requirement.** Rejected: two requirements of one crate may
be deliberate, and a new rule should decline to judge what it cannot weigh
rather than report it.

**Report at the entry's line.** Rejected: a spanned TOML parse for a line number
nobody's fingerprint uses.

**Repair it under `--fix`.** Rejected: the fixers rewrite `.rs` files from `syn`
spans; a manifest rewriter would be new machinery for a one-line deletion.

## Consequences

**It judges a file the walker never reads**, as `workspace-dependencies` and
`spdx-matches-manifest` do, through `Config`.

**Target tables are now read**, and handed to this rule only.
`workspace-dependencies` is still given the plain tables, so nothing it reports
changed; whether it should judge target tables too is open.

**A repository that installs this release gets an offence for each redundant
repeat**: two in twin4rust's core and one in dry4rust's, each cleared by deleting
the line.

### What this rule does not catch

**A pair written unlike, or with two version requirements**, by decision.

**Two spellings of one target** -- `cfg(unix)` and `cfg( unix )` -- are two
targets: the target is compared as written.

**A build dependency repeating a dependency.** A build script is a separate
graph; neither table gives the other anything.

**A dev-dependency the tests do not use.** That is `cargo-shear`'s.

## Enforcement

`tests/rules/manifest/distinct_dev_dependencies_rule_tests.rs` -- the repeat,
its exact description and correction, a targeted repeat naming its table, each
shape left alone (a feature added, an optional dependency, a targeted
dependency, a pair written unlike, a build dependency, a dev entry alone), two
repeats, and the rule's name, configuration, requirement and explanation.

`tests/finding/model/manifest_dependency_tests.rs` -- `repeats`, shape by shape;
`cli/tests/system/dependency_tables_tests.rs` -- what is read from the TOML,
since the core/cli split in the cli.

`tests/runner_tests.rs` -- a package outside a workspace reported against its
`Cargo.toml`, and a six-member workspace stating one member's repeat once.

Stage 2 runs the tool against this repository at zero offences.

## Related

- [R021-ADR-WorkspaceDependenciesRule](R021-ADR-WorkspaceDependenciesRule.md) --
  the other rule about how a dependency is written, beside this one in
  `src/rules/manifest/`; it reports the spelled-out half of a pair this rule
  leaves alone.
- [ADR-ManifestDataIsPerPackage](ADR-ManifestDataIsPerPackage.md) -- why each
  package is handed only its own manifest's entries.
- [ADR-FixOnlyWhatIsSafe](ADR-FixOnlyWhatIsSafe.md) -- why `--fix` does not
  touch manifests.
