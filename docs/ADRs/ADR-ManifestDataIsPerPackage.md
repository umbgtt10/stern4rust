# ADR-ManifestDataIsPerPackage

- **Status:** Accepted
- **Date:** 2026-08-21

## Context

Two rules answer to the manifest rather than to the source tree.
`spdx-matches-manifest` compares the identifier in a file's header against the
licence the package declares. `workspace-dependencies` asks whether a member
takes its dependencies from the workspace. Neither can run without reading a
`Cargo.toml`, and neither takes a flag for it.

Both are read **once for the whole run**, before the scan starts:

```rust
let config = Config {
    manifest_license: ManifestResolver::license(&config),
    workspace_dependencies: ManifestResolver::workspace_dependencies(&config),
    ..config
};
```

The comment above it says why: *"The one piece of configuration that comes from
the package being judged rather than from the command line, read once for the
run."* That is exactly right for a run that judges one package, and it is what
every run did until a workspace was scanned whole.

`license` already knows the multi-package case exists and tries to be careful:

```rust
let declared: BTreeSet<&String> = selected
    .iter()
    .filter_map(|package| package.license.as_ref())
    .collect();
if declared.len() != 1 || declared.len() != selected.len() {
    return None;
}
```

The intent reads plainly enough — one licence, and everybody declared it. The
comparison does not say that. `declared` is a set of **distinct licence
strings**; `selected` is a list of **packages**. The two are only ever the same
length when there is exactly one package.

So four members all declaring `Apache-2.0` gives `declared.len() == 1` and
`selected.len() == 4`, and the guard returns `None`. Measured on
`etheram-raft`, whose four members all declare `Apache-2.0`:

```
cargo stern4rust --manifest-path Cargo.toml
  not applied: spdx-matches-manifest (needs a `license` field in Cargo.toml)
  summary: files_scanned=252 ... rules_applied=20 rules_unconfigured=1
```

Every one of those 252 files carries `// SPDX-License-Identifier: Apache-2.0`
and every manifest declares it. The rule stood down and said the licence was
missing.

**As written, `spdx-matches-manifest` can only ever apply to a single-package
scan.** It has never fired on a workspace, and the message it gives while
declining points at the wrong thing — a reader checks the manifest, finds the
field, and has nowhere to go.

## Decision

**Manifest-derived configuration is resolved per package, inside the scan loop,
rather than once for the run.**

The loop already runs per package:

```rust
let roots = ManifestResolver::package_roots(&config)?;
for root in &roots {
    ... walk, read, check ...
}
```

What it lacks is the package's identity: `package_roots` returns bare
`PathBuf`s, so by the time a file is judged there is nothing left saying which
member it came from. It returns name and path instead, and the licence, the
dependency list, the rule registry and the exclusion set are all built from the
package about to be walked.

**`license` stops aggregating.** There is no "the one licence the scanned
packages agree on" any more, because there is no longer one question. Each
package answers for itself, and a workspace mixing MIT and Apache-2.0 — which
this family will have, since the tools are MIT and `etheram-*` is Apache-2.0 —
gets both rules holding rather than both standing down.

This ADR exists because the fix is not only a repaired comparison. Saying what
the guard meant -- one licence, and every selected package declared it -- would
be *safe*: a workspace whose members disagree returns `None` and the rule stands
down, so it would never fire against correct code. An earlier draft of this ADR
claimed otherwise and was wrong.

What it would not do is **work**. A mixed-licence workspace is exactly the shape
this family is heading for -- the tools are MIT, `etheram-*` is Apache-2.0 -- and
under the repaired aggregate every file in it goes unjudged, silently, forever.
The rule would be correct and useless. **The aggregate is the defect; the
comparison is a symptom**, and the corrected aggregate survives in the runner as
the one thing the *report* can honestly claim about a whole run.

## Consequences

**`spdx-matches-manifest` starts applying where it never has.** Any repository
scanning more than one package at once will see the rule move from *not applied*
to *applied*, and offences may appear. They are real: the rule was never able to
speak there. This is a behaviour change on a published tool and belongs in a
minor version with the changelog saying so plainly.

**`workspace-dependencies` gets the same treatment for the same reason**, though
its aggregate is less wrong — it reads every member's manifest already, because
its question is about the workspace. What changes is that it is asked once per
package rather than once per run, and that is the whole difficulty: the same
finding is then available to be stated once per member.

The answer is **filtering, not deduplication**. Each package is handed only the
declarations stated in its own manifest, so the finding about `alpha/Cargo.toml`
exists in exactly one package's registry — `alpha`'s — and nothing downstream
has to work out whose it was.

Deduplication was tried first and is not sufficient, in either direction. A
global one collapses two members' `src/lib.rs` into one finding when they are
two real files sharing a package-relative path; that was `0.10.2`. A per-package
one lets a manifest finding through once per member; that was `0.10.5`, caused
by `0.10.2` and surviving three releases because a count that is too large reads
as a codebase with work to do rather than as a defect.

The distinction deduplication cannot make, and the loop makes for free, is
whether an offence is about a file *inside* the package that produced it. Two
`src/lib.rs` are two files. Twenty-nine reports of `alpha/Cargo.toml` are one.

**It is a prerequisite for
[ADR-PerPackageConfiguration](ADR-PerPackageConfiguration.md), not a
consequence of it.** Per-package rule sets need the loop to know which package it
is in; so does this. Doing this first means the licence bug is fixed on its own,
against its own test, rather than arriving inside a larger change where a
regression would be hard to attribute.

**The report's `not applied` reason becomes per package.** "Needs a `license`
field in `Cargo.toml`" has to say *which* `Cargo.toml`, or a workspace where one
member is missing the field reads as though all of them were.

## What this does not solve

**A virtual manifest declares nothing.** A workspace root with only
`[workspace]` has no `license` field and no package, so a scan naming it still
has nothing to compare against for the root itself. That is correct — there is
no package there — but the message should say so rather than reusing the
"needs a `license` field" wording aimed at a real package that lacks one.
