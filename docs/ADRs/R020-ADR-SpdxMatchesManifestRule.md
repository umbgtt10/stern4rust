# R020-ADR-SpdxMatchesManifestRule

- **Status:** Accepted
- **Date:** 2026-08-20

## Context

[R001](R001-ADR-HeaderRule.md) compares a header against a text file and nothing
else, and says so in its own "does not catch": a file whose header is perfectly
formatted can name the wrong copyright holder, and an SPDX identifier can
disagree with the `license` field in `Cargo.toml`.

A package states its licence twice — once in the manifest, once in every header
— and nothing held the two statements together. `OPEN_POINTS.md` has carried
this since `0.2.0`.

## Decision

**Every file's `SPDX-License-Identifier` says what the manifest's `license`
says.**

The expected value comes from the **package being judged** rather than from a
flag, and that is the difference from `header` that matters: this rule needs no
`--header-file` to hold. It is the first rule whose configuration is read from
the manifest, and `ManifestResolver::license` reads it once for the run.

The header is the comment block a file opens with — everything before the first
line that is neither blank nor a `//` comment. An SPDX line below that is prose
or code and declares nothing.

**A manifest that names no licence leaves the rule with nothing to work from, so
it answers `is_configured` with false** and the registry drops it. The report
then names it as not applied, which is the tool's third state and exactly what
it exists for.

That was measured rather than chosen. The first design reported the silent
manifest as an offence against `Cargo.toml`, and because `Runner` asks the
workspace question once per package root, `braintax4rust` — a workspace of twenty
packages — produced **twenty identical lines**. `is_configured` is the mechanism
this codebase already had for "nothing to work from", and using it removes the
duplication entirely.

| offence | correction |
|---|---|
| `` src/widget.rs carries no `SPDX-License-Identifier:`, so nothing ties it to the `MIT` the manifest declares `` | add `// SPDX-License-Identifier: MIT` to the header, or correct the manifest |
| `` src/widget.rs declares `Apache-2.0` where the manifest declares `MIT` `` | change the header to `// SPDX-License-Identifier: MIT`, or correct the manifest |

Both corrections end "**or correct the manifest**". The rule knows the two
disagree, not which one is right, and a correction that assumed the header was
wrong would be guessing.

## Forcing constraints / Evidence

Measured across the family before the rule was written:

| repository | manifest `license` | files without a matching SPDX |
|---|---|---|
| `stern4rust`, `twin4rust`, `iceberg4rust`, `slotgate`, `crap4rust`, `grip4rust` | MIT | 0 |
| `braintax4rust` | MIT | **1** |
| `etheram-core` | **absent** | 25 files claim Apache-2.0 in prose, 0 carry SPDX |

**`braintax4rust/core/src/traits/mod.rs` carries no header at all** — no
copyright, no licence, no SPDX, just three `pub mod` lines. It is the kind of
file that slips through precisely because it looks like nothing: a registry
three lines long.

That single finding is the rule's justification, and it makes the case for
manifest-derived configuration on its own. `header` **cannot** catch it there,
because `braintax4rust` has no header file to configure `header` with. This rule
needs none.

`etheram-core` is the other shape and the reason `is_configured` is the right
answer: no `license` field, no SPDX anywhere, 25 files asserting Apache-2.0 in
prose that nothing machine-readable confirms. The rule reports **nothing** and
the run says `spdx-matches-manifest (not configured)` — visible, unambiguous, and
not mistakable for a pass.

## Rejected alternatives

**Report the silent manifest as an offence.** Rejected on measurement: twenty
identical `Cargo.toml` lines on a twenty-package workspace, because the workspace
question is asked once per package root. See the Decision.

**Take the expected licence from a flag**, as `header` does. Rejected: the
manifest already states it, and a second place to state the same fact is the
problem this rule exists to solve.

**Check the copyright holder too.** Rejected: the manifest's `authors` field is
optional, frequently stale, and not the same claim as a copyright line. There is
no second source of truth to check it against.

**Require a specific SPDX spelling** (`Apache-2.0` versus
`Apache License, Version 2.0`). Rejected: the rule compares the manifest's string
with the header's, so whatever the manifest says is what is required. It holds
the two together without having an opinion about either.

**Apply only to `src/`.** Rejected: a test file carries the same header and ships
in the same package.

## Consequences

**Configuration now flows from the manifest into a rule**, which is new. `Config`
gained `manifest_license`, `Runner` fills it once before building the registry,
and `ManifestResolver` reads it. The seam is small but it is a seam: a second
rule wanting manifest data would follow the same path rather than inventing one.

**A workspace whose packages declare different licences is not judged.**
`ManifestResolver::license` returns a value only when every selected package
agrees, so a mixed workspace leaves the rule unconfigured rather than holding
files to a licence that is not theirs.

### What this rule does not catch

**A licence stated only in prose.** `// Licensed under the Apache License,
Version 2.0` with no SPDX line is reported as *missing an identifier*, not as
contradicting one — the rule reads SPDX and nothing else.

**Whether the licence is correct**, or whether the `LICENSE` file at the root
agrees with either. It holds two statements together; it does not know which is
right.

**The copyright holder or year**, which `header` governs where it is configured.

**A package with no `license` field**, which is an absence of configuration
rather than an offence. It is named in the report, not counted.

**Mixed-licence workspaces**, by decision, above.

## Enforcement

`tests/rules/spdx_matches_manifest_rule_tests.rs` — 11 tests covering the
matching identifier, the mismatch, the missing identifier, the file with no
header at all, the prose-only claim, an SPDX line below the header,
`check_workspace`, and both `is_configured` answers.

`tests/settings/manifest_resolver_tests.rs` — 2 tests on reading the licence:
this crate's `MIT`, and `None` where no single answer exists.

`tests/rule_registry_tests.rs` — `fully_configured` now supplies both a header
and a licence, so the "every rule" lists pin that two rules need configuration
rather than one.

## Related

- [R001-ADR-HeaderRule](R001-ADR-HeaderRule.md) — compares the header against a
  text file; this compares one line of it against the manifest, and needs no
  flag to do so. R001 records this gap in its "does not catch".
- [ADR-RuleSelection](ADR-RuleSelection.md) — the three-state reporting that
  makes an unconfigured rule visible rather than silent.
