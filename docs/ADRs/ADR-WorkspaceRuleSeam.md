# ADR-WorkspaceRuleSeam

- **Status:** Accepted
- **Date:** 2026-08-18

## Context

`Rule` began as one question: given a file, what is wrong with it. That fits
`header` and `test-file-structure` exactly, and it is what makes a rule cheap
to write and cheap to test — a rule is testable against a string of source with
no workspace behind it.

It does not fit every rule. "There is exactly one `tests/all_tests.rs`" and
"every subfolder has a `mod.rs`" are facts about a tree. Worse, the file that
carries such an offence is usually **the one that does not exist**: the report
has to name `tests/rules/mod.rs` precisely because no walker ever found it and
no `check(&SourceFile)` can ever be handed it.

## Decision

`Rule` carries `check_workspace(&[SourceFile])` beside `check(&SourceFile)`,
and **both default to reporting nothing**, so each rule implements only the
question it actually answers and `RuleRegistry` calls both without caring which.

`Runner` reads every file in a package before judging any of them, then asks the
per-file question of each file and the workspace question once.

## Forcing constraints / Evidence

Two of the four checks in [R003](R003-ADR-TestsLayoutRule.md) cannot be
expressed against a single file at all. `missing_door` has no file to be handed
when `tests/all_tests.rs` is precisely what is absent, and `missing_mod_files`
cannot know whether a sibling `mod.rs` exists without seeing the other files.

The defaults are what keep the seam cheap. Without them, every existing rule
would have grown an empty `check_workspace`, and every future rule would have to
write two methods to use one.

Reading the whole package before judging it removed a second hazard that was not
the point of the change: results can no longer depend on the order the walker
happened to return files in.

## Rejected alternatives

**Give `Rule` only `check_workspace` and let per-file rules iterate.** Rejected:
every simple rule would carry a loop that has nothing to do with what it checks,
and the per-file rules are the common case.

**A second trait, `WorkspaceRule`, with its own registry.** Rejected: two
registries, two collection paths in `Runner`, and a rule that wanted both
questions would have to be two objects. The defaulted-method version costs one
line per rule and no new machinery.

**Have the walker synthesise a placeholder `SourceFile` for paths that do not
exist.** Rejected: it would require knowing which paths to synthesise, which is
the very question the rule exists to answer.

**Report a missing `mod.rs` against the orphaned test file instead**, so a
per-file rule could do it. Rejected: the actionable fact is the path to create,
not the file that is being ignored because of its absence.

## Consequences

Every future rule faces a choice about which method to implement. The defaults
make that cheap, but a rule implementing both must be careful not to report the
same offence twice — which is why `TestsLayoutRule` has a test asserting that
its `check` says nothing.

`Runner` holds an entire package's files in memory rather than streaming. For a
linter over a source tree this is not a meaningful cost.

The trait's surface is now two methods where the ADR for a one-method trait
would have been simpler to write. That is the price of the rules that do not fit
the simple shape, and it is paid once.

## Enforcement

`tests/rule_registry_tests.rs::check_workspace_collects_the_offences_of_every_registered_rule`
pins that the registry fans the workspace question out to every rule and hands
it the whole file set.

`check_of_a_workspace_only_rule_reports_nothing` pins the other direction: a
rule that answers only the workspace question stays silent through the per-file
door, so nothing is reported twice.

`tests/rules/tests_layout_rule_tests.rs::check_of_a_single_file_reports_nothing`
pins the same property at the rule.

## Related

- [R003-ADR-TestsLayoutRule](R003-ADR-TestsLayoutRule.md) — the rule that forced
  this seam, and the only one currently using it.
