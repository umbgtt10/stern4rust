# R021-ADR-WorkspaceDependenciesRule

- **Status:** Accepted
- **Date:** 2026-08-20

## Context

A workspace exists so its members agree. The moment a member spells out a
dependency of its own, two versions of the same crate can resolve, a bump has to
be applied in several places, and the root manifest stops being the answer to
"what does this workspace depend on".

Three requirements were asked for:

1. the root manifest holds **all** the references
2. every member takes them with `.workspace` notation
3. members declare **no** new dependencies

## Decision

**A workspace declares its dependencies once, in the root, and every member
takes them from there.**

**The three requirements are one requirement, and only the middle one needs
code.** A member writing `foo = { workspace = true }` for a `foo` the root does
not declare **does not compile** — `cargo` rejects it outright. So requiring the
root to hold every reference costs nothing here, and "no new dependencies in a
member" is exactly what a member using nothing but `.workspace` already means.
One syntactic check delivers all three.

That is the fourth time this repository has found the same shape:
[R009](R009-ADR-RegistryCompletenessRule.md) (missing file is `E0583`),
[R014](R014-ADR-PureTraitsRule.md) (incomplete impl is `E0046`),
[R016](R016-ADR-PairedTestFileRule.md) (one direction only), and now this. Half
of a two-directional requirement keeps turning out to be the compiler's.

**Read from the TOML, not from `cargo metadata`.** The question is *how a
dependency was written*, and resolution erases precisely that: a dependency taken
from the workspace and one spelled out in the member are identical once resolved.
`ManifestResolver` parses each member manifest with the `toml` crate it already
depends on.

**All three dependency tables count** — `dependencies`, `dev-dependencies` and
`build-dependencies`. A member pinning its own `proptest` splits the workspace
exactly as a runtime dependency does.

**Intra-workspace path dependencies are included, and that was decided on
evidence rather than taste.** `etheram-ibft`'s root already centralises
`execution-types = { path = "execution-types" }` and `evm = { path = "evm" }`,
while `node = { path = "../node" }` is spelled out in four members. The
repository is inconsistent with itself, and its own root shows which way it
meant to go.

**A package that is not a workspace has no root to centralise into**, so the
rule says nothing — the same silence `tests-layout` keeps about a package with
no tests tree. This is not `is_configured`: the rule is perfectly configured, it
simply has no subject.

| offence | correction |
|---|---|
| `` validation/Cargo.toml declares `node` in [dependencies] rather than taking it from the workspace `` | add `node` to [workspace.dependencies] in the root manifest, and write `node = { workspace = true }` here |

The correction names **both** edits, because the fix is one line in each of two
files and naming only one of them would leave a build that does not compile.

## Forcing constraints / Evidence

| repository | workspace | offences |
|---|---|---|
| `stern4rust`, `twin4rust`, `iceberg4rust`, `slotgate`, `etheram-core` | **no** | rule silent |
| **`crap4rust`** | yes, **no `[workspace.dependencies]` at all** | **11** |
| **`grip4rust`** | yes, **no `[workspace.dependencies]` at all** | **10** |
| **`etheram-ibft`** | yes | **6** |
| `braintax4rust` | yes | 0 of 65 |
| `etheram-raft` | yes | 0 of 27 |

**27 offences across three workspaces**, and the split is the interesting part.
`braintax4rust` centralises all 65 of its dependencies and `etheram-raft` all 27,
so the convention is demonstrably workable at that size. `crap4rust` and
`grip4rust` have no `[workspace.dependencies]` section whatsoever — every
dependency is a member's own. `etheram-ibft` has the section and uses it for
most things, and leaks exactly on the intra-workspace path dependencies.

## Rejected alternatives

**Check that the root declares every referenced dependency.** Rejected on
evidence: `cargo` refuses to build otherwise. See the Decision.

**Exempt intra-workspace path dependencies.** Rejected on evidence: the one
workspace that leaks them already centralises two of the same kind in its own
root. Exempting them would have hidden all six `etheram-ibft` findings.

**Exempt `dev-dependencies`.** Rejected: a member pinning its own test
dependency splits the workspace the same way, and `etheram-ibft` has exactly one
such case.

**Use `cargo metadata` rather than parsing TOML.** Rejected as impossible:
resolution erases the distinction the rule exists to see.

**Report against the root manifest.** Rejected: the edit that removes the
offence is in the member, and the root edit is named in the correction.

**Treat a non-workspace package as unconfigured.** Rejected: the rule has
everything it needs and simply has no subject, which is not the same as having
nothing to work from. Reporting `(not configured)` on every single-crate
repository would be noise, and would make the third state mean two things.

## Consequences

**The tool now judges files the walker never reads.** Every rule before this one
judged `.rs` files the walker found; this judges `Cargo.toml`. The manifests are
gathered by `ManifestResolver` and handed to the rule through `Config`, the same
path `spdx-matches-manifest` opened. Offences carry manifest paths relative to
the workspace root.

**It exposed a duplication bug in `Runner`, now fixed.** The workspace question
is asked once per package root, so a rule whose subject is the *workspace* rather
than the package states each finding once per member: `etheram-ibft` reported 36
where there are 6. Offences are now deduplicated by content before sorting.
`Vec::dedup` was not enough — two findings about one manifest interleave once
sorted, so consecutive-only removal missed every copy after the first pair.

**A repository that deliberately lets members pin their own versions must skip
the rule.** That is rule selection working as designed.

### What this rule does not catch

**Whether the root's `[workspace.dependencies]` is *tidy*** — an entry nothing
references is invisible here, and `cargo` does not complain either.

**Version drift inside the root.** Two entries for the same crate under
different names, or a `workspace = true` overriding features in a way that
changes resolution, are not judged.

**`[patch]`, `[replace]` or target-specific dependency tables**
(`[target.'cfg(unix)'.dependencies]`). Only the three plain tables are read.

**Whether the member *needs* the dependency at all.** An unused dependency taken
correctly from the workspace passes.

**A non-workspace package**, by decision, which is five of the ten repositories
measured.

## Enforcement

`tests/rules/manifest/workspace_dependencies_rule_tests.rs` — 8 tests covering
the member declaration, the member taking from the workspace, the pinned
dev-dependency, several members at once, the non-workspace package, the per-file
door, `is_configured`, and the rule's name.

`tests/finding/manifest_dependency_tests.rs` — 2 tests on the declaration model
and the three tables it covers.

`tests/settings/manifest_resolver_tests.rs` — that this crate, a single package,
resolves to `None`.

## Related

- [R020-ADR-SpdxMatchesManifestRule](R020-ADR-SpdxMatchesManifestRule.md) —
  opened the path from the manifest into a rule; this is the second to use it,
  and the two live together in `src/rules/manifest/`.
- [R009](R009-ADR-RegistryCompletenessRule.md),
  [R014](R014-ADR-PureTraitsRule.md),
  [R016](R016-ADR-PairedTestFileRule.md) — the same "only one direction needs a
  rule" split, found three times before.
- [R010-ADR-DirectoryFileCountRule](R010-ADR-DirectoryFileCountRule.md) — the
  rule that forced `src/rules/` to be grouped into subfolders before a
  twenty-first rule could be added.
