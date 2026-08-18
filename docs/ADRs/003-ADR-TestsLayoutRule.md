# 003-ADR-TestsLayoutRule

- **Status:** Accepted
- **Date:** 2026-08-18

## Context

A tests folder in this convention is reached through exactly one door.
`Cargo.toml` sets `autotests = false` and declares a single
`[[test]] name = "all_tests"`, so `tests/all_tests.rs` is the only
integration target; it declares `pub mod` for each subfolder, each
subfolder's `mod.rs` declares the files below it, and so on down.

Miss one of those declarations and the files beneath it are **not compiled
at all**. They still exist on disk. They still look like tests. They are
still counted by anyone reading the directory, still show up in a file
search, still get maintained in code review. Nothing runs them.

This is the failure the rule exists for, and its defining property is that
it is silent by construction: *a test that is never compiled cannot fail*.
No gate in the toolchain notices — not `cargo test`, which reports a
green run over the tests it was given; not `clippy`; not coverage, which
reports on the code that was instrumented. The only visible symptom is a
test count that is lower than it should be, which nobody is watching.

There is a second, quieter concern. Both registry files exist to be
scanned: a reader opens `all_tests.rs` to find out what a package tests.
Anything else living in them — a helper, a constant, a `use`, an inline
`mod` with a body — is logic hiding in the one file nobody reads carefully,
because its whole job is to be a list.

## Decision

A tests folder is reached through exactly one `tests/all_tests.rs` and a
`mod.rs` in **every** subfolder on the way down; both kinds of registry hold
nothing but the header and `pub mod` declarations.

Four checks implement that sentence: the top-level door must exist
(`missing_door`); an `all_tests.rs` anywhere below the top is not a door and
is reported as such (`stray_doors`); every folder beneath `tests/` must have
a `mod.rs` (`missing_mod_files`); and every registry's items must all be
declarations (`registry_contents`).

Each stray in a registry is reported **at its own line and by its own name** —
"the constant `LIMIT`", "the import `use std::fmt;`" — rather than as a
repeated statement that the file contains something it should not.
`RegistryParser` exists for that naming and for nothing else. The first
version of this rule discarded the offending item and emitted one fixed
sentence per stray, so a registry holding four of them produced four
byte-identical rows all pointing at line 1: a report asserting that a file
had four problems while naming none of them, and useless to a reader and a
tool alike.

**This is the rule that widened the `Rule` trait.** The first two rules judge
one file at a time, but here the offending file is usually the one that
*does not exist* — so there is nothing for `check(&SourceFile)` to be handed,
and the report has to name a path no walker ever produced. `Rule` therefore
carries a second method, `check_workspace(&[SourceFile])`, taking the whole
package at once. Both methods default to returning nothing, so each rule
implements only the question it actually answers and `RuleRegistry` calls
both without caring which. `Runner` now reads every file in a package before
judging any of them.

## Forcing constraints / Evidence

**The silence is the whole argument.** Every other rule in this tool catches
something a careful reader would eventually notice. This one catches
something no reader notices, because the evidence of the failure is the
*absence* of output. That asymmetry is why it is worth a rule despite being
the least interesting of the three to describe.

**The trait widening was forced, not chosen.** Two of the four checks cannot
be expressed against a single file at all: `missing_door` has no file to be
handed when `tests/all_tests.rs` is precisely what is absent, and
`missing_mod_files` cannot know whether a sibling `mod.rs` exists without
seeing the other files. `check_of_a_single_file_reports_nothing` pins the
consequence — the rule must not report the same fact twice through both
doors — and `rule_registry_tests::check_of_a_workspace_only_rule_reports_nothing`
pins the same property one level up, at the seam itself.

**Intermediate folders are the case that would have been missed.**
`subfolders` walks `for depth in 2..=parts.len()`, so
`tests/a/b/x_tests.rs` requires both `tests/a/mod.rs` and `tests/a/b/mod.rs`
— not only the folder that directly holds the file. Pinned by
`check_workspace_an_intermediate_folder_without_a_mod_file_reports_it_missing`
and `check_workspace_a_deeply_nested_subfolder_without_a_mod_file_reports_it_missing`.

**Verified against a real target, not only fixtures.** Run against
`etheram-core` — 31 files, a `tests/node_common/` subfolder — this rule
reports nothing, while `test-file-structure` reports 8 genuine ordering
offences in the same run. A rule that fires on every real repository is
indistinguishable from a rule that is wrong, and this one does not.

## Rejected alternatives

**Report a missing `mod.rs` against the orphaned test file.** Rejected: the
actionable fact is the path that has to be created, so the offence names
`tests/rules/mod.rs` rather than the file that is being ignored because of
it. The offending path being absent from disk is exactly why the workspace
seam exists.

**Count only folders that directly contain a file.** Rejected: an
intermediate folder is a folder too, and a gap there hides everything
beneath it just as completely — see the evidence above.

**Treat any `all_tests.rs` as a door, wherever it sits.** Rejected: with a
single declared `[[test]]` target, only the top-level one is ever compiled.
A second one lower down is a file with a name that promises something it
cannot deliver, and no `pub mod` will ever reach it. Pinned by
`check_workspace_a_second_all_tests_below_the_top_reports_it` and
`check_workspace_an_all_tests_only_in_a_subfolder_reports_the_top_one_missing`.

**Allow an inline `mod name { ... }` in a registry.** Rejected: a
declaration points at a file; a module with a body is code, and it is code
placed in the one file a reader scans expecting a list.
`RegistryParser::is_declaration` therefore requires `Item::Mod` with
`content.is_none()`. Pinned by
`check_workspace_a_registry_holding_an_inline_module_reports_it`.

**Require `pub` on a declaration.** Rejected: a private `mod name;` compiles
that file just as well, and being compiled is the entire concern of this
rule. Pinned by `strays_of_a_private_mod_declaration_returns_nothing`.

**Permit `extern crate` in a registry.** Rejected as unnecessary: the strict
form is header plus `pub mod` and nothing else, and neither this crate nor
`etheram-core` needs an exception. If a repository turns up that does, this
is the decision to revisit — and the fact that it was considered and
declined is recorded here so the next reader does not have to rediscover the
question.

**Fold these checks into `test-file-structure` as a branch.** Rejected: that
rule judges the shape of a file's contents and this one judges the shape of
a tree. Keeping them apart is what lets the structure rule skip registries
outright rather than growing a second mode.

## Consequences

`Rule` now has two methods where it had one, and every future rule faces a
choice about which to implement. The defaults make that cheap — a rule
implements one and inherits a no-op for the other — but the seam is now part
of the trait's surface and a rule that implements both must be careful not
to report the same offence twice.

`Runner` holds an entire package's files in memory before judging any of
them, rather than streaming file by file. For a linter over a source tree
this is not a meaningful cost, and it also removes a subtler hazard: results
can no longer depend on the order the walker happened to return files in.

Adding a test file to a subfolder now requires editing that folder's
`mod.rs`, and the rule will not tell you so — see below.

**What this rule does not catch.** It verifies that each registry *exists*
and that its contents are declarations. It does not verify that the
declarations are *complete*. A `tests/rules/mod.rs` that is present, valid,
and simply fails to mention `alpha_tests` leaves `tests/rules/alpha_tests.rs`
uncompiled — the same silent failure this rule exists for, one level down,
and `rustc` says nothing because an undeclared file is not an error. Closing
that gap means resolving each `pub mod` to the file it names and each file
to the declaration that should point at it, in both directions. It is the
obvious next rule and it is deliberately not this one.

Two narrower gaps, both accepted: `#[cfg(...)]`-gated declarations are
treated as ordinary ones, and a registry that does not parse is skipped in
silence rather than reported.

## Enforcement

`tests/rules/tests_layout_rule_tests.rs` — 19 tests, written before the
implementation and confirmed red on
`error[E0432]: unresolved import stern4rust::rules::tests_layout_rule`,
covering each of the four checks, the nested and intermediate folder cases,
the source-tree exclusion, the empty registry, the multiple-offence case,
and the single-file no-op.

`tests/registry_parser_tests.rs` — 13 tests — covers the naming of every item
kind a registry can wrongly hold, including
`strays_reports_each_stray_at_its_own_line`, which is the regression test for
the byte-identical-rows defect.

`tests/rule_registry_tests.rs::check_workspace_collects_the_offences_of_every_registered_rule`
pins that the registry actually fans the workspace question out to every
rule and hands it the whole file set, and `run_stage_2.ps1` runs the compiled
binary against this crate's own tree — where, unavoidably and correctly, the rule's own test file is
subject to it.

## Related

- [002-ADR-TestFileStructureRule](002-ADR-TestFileStructureRule.md) — skips the two
  registry files precisely because their shape is decided here.
- [001-ADR-HeaderRule](001-ADR-HeaderRule.md) — the registries are subject to it,
  since "header plus `pub mod` declarations" still begins with the header.
