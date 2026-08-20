# R016-ADR-PairedTestFileRule

- **Status:** Accepted
- **Date:** 2026-08-20

## Context

[R015](R015-ADR-TestFileNamePostfixRule.md) closed one gap in the mirrored
pairing and left the sharper one open, and its "does not catch" said so:
`banana_tests.rs` with no `banana.rs` behind it satisfied every rule this tool
has.

The pairing was enforced from **one side only**. `twin4rust` starts at a source
file and looks for its test, so it reports a source file with no tests. Nothing
started at a test file and asked whether the source it is named after still
exists. A test file outlives the module it was named for **silently**: it still
compiles, still runs, still passes, and its name now points at nothing.

The failure is a reader's rather than the compiler's. Somebody looking for the
tests of `retention_window.rs` finds `retention_window_proptest_tests.rs`, and
never learns that `retention_tests.rs` holds seven more.

## Decision

**A `tests/<path>/<X>_tests.rs` names the source file it exercises, and that
file exists.** The counterpart of `tests/a/b_tests.rs` is `src/a/b.rs`.

Matched **by path, not by name alone**, so a test file in the wrong directory is
as unpaired as one whose source is gone. Both leave a reader looking in the
wrong place.

This is a workspace rule: the file that proves the offence is the source file
that is not there, so there is nothing for `check` to be handed. See
[ADR-WorkspaceRuleSeam](ADR-WorkspaceRuleSeam.md).

**`all_tests.rs` is exempt.** It ends in `_tests.rs` and is a registry, not a
test file; resolving it would look for `src/all.rs`.

**`_proptest_tests.rs` is exempt, and this was measured rather than assumed.** A
property-test suite is a *second* suite for a module it does not name, so its
stem resolves to a file nobody ever meant to write. Before the exemption, the
rule found 7 unpaired files in `etheram-ibft/node`; 3 of them were this class —
`retention_window_proptest_tests.rs`, `world_state_snapshot_proptest_tests.rs`
and `scheduled_validator_fast_forward_proptest_tests.rs`, whose real
counterparts all exist. Excluding them removed exactly those 3 and left the 4
real findings untouched.

| offence | correction |
|---|---|
| `` tests/state/etheram_state_ibft_tests.rs is named for src/state/etheram_state_ibft.rs, which does not exist `` | rename it after the source file it exercises, or delete it if that file is gone |

**The correction deliberately does not say "create the missing source file."**
Measured against a real tree, every unpaired file tested something real under a
name that had drifted — `retention_tests.rs` imports `RetentionWindow` and
exercises it perfectly well. The file to create is never the answer. Either the
name is wrong, or the tests have outlived their subject.

## Forcing constraints / Evidence

**The rule's four findings in `etheram-ibft/node` were predicted by hand before
it existed, and it reproduced them exactly.** 178 source files, 93 test files,
and these four named for source files that exist nowhere in the crate:

| test file | tests | names |
|---|---|---|
| `state/etheram_state_ibft_tests.rs` | 23 | `src/state/etheram_state_ibft.rs` |
| `ibft/ibft_protocol/retention_tests.rs` | 7 | `src/ibft/ibft_protocol/retention.rs` |
| `outgoing/etheram_ibft_output_dispatcher_tests.rs` | 5 | `src/outgoing/…` |
| `state/storage/transaction_receipt_storage_tests.rs` | 5 | `src/state/storage/…` |

**Forty tests in files named after source files that do not exist**, every one
of them passing, in a crate whose gates are green.

Across the family, in packages the mirrored convention applies to:

| package | offences |
|---|---|
| `stern4rust`, `twin4rust`, `iceberg4rust`, `etheram-core` | 0 |
| `slotgate` | 1 |
| `crap4rust` | 2 |
| `grip4rust` | 8 |
| `braintax4rust` | 19 |
| `etheram-ibft/node` | 4 |

**`grip4rust`'s eight are the same eight files
[R009](R009-ADR-RegistryCompletenessRule.md) found never compiled** — the
`*_analysis_tests.rs` suites that `tests/all_tests.rs` declares none of. Two
rules, approaching from unrelated directions, land on the same files. That is
the strongest evidence either rule has: those files are adrift by two
independent measures.

`braintax4rust`'s nineteen are 18 fixture **crates** each holding a
`tests/analysis_tests.rs` with no `src/analysis.rs`, plus one real finding. They
share a relative path but are eighteen distinct files.

## Rejected alternatives

**Report against the missing source file.** Rejected: it does not exist, and the
edit that resolves the offence is a rename of the file that does.

**Say "create `src/<X>.rs`" in the correction.** Rejected on measurement — see
the Decision. It would have been wrong in all four `node` cases.

**Strip `_proptest` and resolve to the base module** rather than exempting the
file. Considered seriously: it fixes the same 3 false positives while still
catching a proptest suite naming a vanished module. Rejected as speculative
precision — nothing in the family exhibits that case, the suffix list is
open-ended (`_stress_tests`, `_regression_tests` cost another rule change each),
and an exemption that is wrong is easier to notice than a resolution that is
subtly clever.

**Accept `src/<X>/mod.rs` as a counterpart.** Rejected: `module-registry`
forbids code in a `mod.rs`, so a directory module has no subject to test.

**Match by filename anywhere in `src/`, ignoring the path.** Rejected: it would
accept a test file in a directory that mirrors nothing, which is half of what
this rule is for.

**Infer whether a package is mirrored** and stay silent where most test files do
not pair. Rejected as the "confident wrong answer" shape this repository keeps
refusing: it goes quiet on a genuinely mirrored crate that has drifted badly,
which is exactly when the rule is worth having.

## Consequences

**The rule assumes the package is mirrored, and that assumption is now explicit
rather than ambient.** A *harness crate* — one whose `src/` is apparatus and
whose `tests/` are scenarios named after behaviours rather than files — is not
mirrored, and every one of its test files is reported.

`etheram-ibft` holds two: **`validation` (53 offences)** and **`system-tests`
(28)**. In `validation`, 12 source files provide a multi-node cluster harness
and 59 test files are named `byzantine`, `partition`, `view_change`,
`recovery_handoff`. Not one pairs, and not one should — the subject under test
is the IBFT protocol in a different crate entirely.

**`--skip paired-test-file` is the answer there**, using the rule selection that
shipped in `0.3.0` for exactly this: a repository adopts the rules that fit it.
No new configuration was added, and the cost is that a workspace mixing both
shapes runs the tool per package.

### What this rule does not catch

**A test file that pairs with a source file but tests something else.**
`widget_tests.rs` beside `widget.rs`, containing tests for a different module
entirely, satisfies the rule completely. It checks that a name resolves, not
that it is honest — the same limit [R012](R012-ADR-TestNamingRule.md) accepts.

**Anything in a `_proptest_tests.rs` file**, by decision. Such a file naming a
module that has since vanished is invisible.

**A source file with no test file at all.** That is `twin4rust`'s direction and
is deliberately not duplicated here.

**Harness crates**, which must skip the rule rather than satisfy it. A package
that skips it is reported as skipped, so this cannot be mistaken for a pass.

**Whether the tests still exercise the module they name.** `retention_tests.rs`
would have been silent had `src/ibft/ibft_protocol/retention.rs` merely been
emptied rather than removed.

## Enforcement

`tests/rules/layout/paired_test_file_rule_tests.rs` — 11 tests covering the unpaired
file, the paired file, the nested paired file, the file in the wrong directory,
the `_proptest` exemption, the `all_tests.rs` registry, a support file that is
not a test file, multiple unpaired files, the single-file `check`,
`is_configured`, and the rule's name.

`tests/rule_registry_tests.rs` — the four hardcoded rule-name lists include
`paired-test-file`.

Stage 2 runs the tool against this crate at zero offences, so a test file added
here without its source counterpart fails the build that introduces it.

## Related

- [R015-ADR-TestFileNamePostfixRule](R015-ADR-TestFileNamePostfixRule.md) — the
  other half. That rule requires a file holding tests to be *named* as one; this
  requires the name to *resolve*. R015 records this gap in its "does not catch".
- [R009-ADR-RegistryCompletenessRule](R009-ADR-RegistryCompletenessRule.md) —
  independently flags the same eight `grip4rust` files, as never compiled.
- [ADR-RuleSelection](ADR-RuleSelection.md) — the mechanism a harness crate uses
  to opt out, and the reason no new configuration was needed.
- [ADR-WorkspaceRuleSeam](ADR-WorkspaceRuleSeam.md) — why this answers
  `check_workspace`.
