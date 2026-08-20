# R009-ADR-RegistryCompletenessRule

- **Status:** Accepted
- **Date:** 2026-08-19

## Context

`tests-layout` and `module-registry` both check that a registry **exists** and
holds only declarations. Neither checks that the declarations are **complete**.

A `mod.rs` that is present, valid, and simply fails to mention `alpha_tests`
leaves `alpha_tests.rs` uncompiled. The file still exists, still looks like a
test file, still has `#[test]` functions in it — and nothing runs them. This is
the same failure `tests-layout` was written for, one level down, and it has been
recorded as the obvious next rule in `OPEN_POINTS.md` since that rule shipped.

## Decision

A registry declares every module beside it: each sibling `.rs` file, and each
subfolder that has a registry of its own.

**Only one direction needs a rule, and this was measured rather than assumed.**
`pub mod missing;` with no `missing.rs` is a **compile error** — `rustc` reports
`E0583: file not found for module`, immediately and more clearly than this tool
could. An orphan `.rs` file that no registry declares produces **no error and no
warning at all**. Silence is the entire failure, so silence is all this rule
looks for. `OPEN_POINTS.md` had proposed resolving the relationship "in both
directions"; half of that work turned out to be the compiler's.

Four decisions keep it from producing wrong answers.

**`pub` is not required here.** `module-registry` demands it in `src/`, and this
rule asks a different question: a private `mod name;` compiles the file just as
well, and being compiled is the whole concern. Requiring `pub` would report a
file that is genuinely reached.

**An inline `mod name { ... }` is not a declaration.** It declares no file, so it
cannot be what reaches one.

**`main.rs` counts as a registry alongside `lib.rs`.** It is an entry point
rather than an index and legitimately holds code, but it may still declare
modules — and a file declared only from `main.rs` *is* reached. The registries of
a directory are treated as one set, so such a file is not reported. The offence
lands on `lib.rs` when a package has both, because that is the index.

**A subfolder without a registry is not expected to be declared.** That folder is
`tests-layout`'s finding, and declaring it would not compile.

The offence is reported against the **registry**, not the orphan. The orphan is a
perfectly good file; the edit that fixes it is one line somewhere else. An
unreadable or unparseable registry silences the rule for that directory — treating
it as declaring nothing would report every file beside it as an orphan, a page of
wrong answers caused by one real one that `readable-source` already reports.

## Forcing constraints / Evidence

Run across eight repositories, the rule found **8 offences in one**: `grip4rust`
has eight `*_analysis_tests.rs` files in `tests/` and `tests/all_tests.rs`
declares **none** of them.

Verified independently of the tool before the rule was believed: the eight files
exist, `grep` finds zero matching declarations, each file carries about four
`#[test]` functions, and `cargo test --test all_tests` runs 220 tests — none of
them from those files. Roughly thirty tests in a published tool have never once
executed, and nothing in any build output says so.

That is the rule's justification in a single case. Every other repository in the
family, and `etheram-core`, is clean.

## Rejected alternatives

**Check both directions.** Rejected on evidence: the missing-file direction is a
hard compile error. A rule duplicating it would add a second, worse report of
something `rustc` already says well.

**Report the offence against the orphan file.** Rejected: the orphan is not
wrong, and an offence on it would name a file that needs no edit. The registry is
where the missing line goes.

**Require `pub mod`.** Rejected: it conflates this rule with `module-registry`
and would report files that are compiled. The correction still *suggests*
`pub mod`, since that is what `module-registry` wants in `src/`.

**Treat an unparseable registry as declaring nothing.** Rejected: one real
offence would become one per sibling file, and every one of them would be wrong.

**Apply only to `tests/`.** Rejected: `module-registry` records the identical gap
for `lib.rs`, and an uncompiled `src/` file is as invisible as an uncompiled test.

## Consequences

**It cannot see a file the walker never reached.** A file inside an `--exclude`d
tree, or under `target/`, is not judged and not expected — correct, but it means
excluding a tree also excludes it from this rule's idea of complete.

**`#[path = "..."]` is not understood.** A file declared through an explicit path
attribute would be reported as undeclared. That convention is now held up by a
rule of its own — [R018](R018-ADR-DeclaredByNameRule.md) forbids the attribute
package-wide, precisely because this rule depends on it — so the false positive
is reachable only by a repository that has skipped `declared-by-name`. A
`#[cfg_attr(..., path = ...)]`, which R018 deliberately allows, is still
misread here on the platform where it applies.

**`#[cfg(...)]`-gated declarations are treated as ordinary ones**, the same
choice `tests-layout` makes. A file declared only under a feature counts as
declared, which is right for "is it ever compiled" and wrong for "is it compiled
in this configuration". The second question needs feature resolution this tool
does not do.

**Conditional compilation aside, a declared file is assumed reached.** The rule
resolves names, not module graphs: a `mod.rs` that is itself never declared makes
its whole subtree unreachable, and each level would be reported separately rather
than as one root cause.

## Enforcement

`tests/rules/registry_completeness_rule_tests.rs` — 11 tests covering the
complete registry, the orphan, the private declaration, the inline module, the
file declared only from `main.rs`, the undeclared subfolder, the subfolder
without a registry, the directory with no registry at all, the unparseable
registry, and the tests-tree shape the gap was first noticed in.

`tests/finding/package_tree_tests.rs` — 9 tests on the directory model, including that
`lib.rs` outranks `main.rs`.

`tests/finding/module_declaration_finder_tests.rs` — 6 tests on what counts as a
declaration.

## Related

- [R003-ADR-TestsLayoutRule](R003-ADR-TestsLayoutRule.md) — checks the registry
  exists; this checks it is complete.
- [R006-ADR-ModuleRegistryRule](R006-ADR-ModuleRegistryRule.md) — records the
  same gap for `lib.rs` in its "does not catch".
