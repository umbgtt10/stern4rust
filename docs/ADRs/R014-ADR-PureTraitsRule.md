# R014-ADR-PureTraitsRule

- **Status:** Accepted
- **Date:** 2026-08-19

## Context

A trait is a declaration of what implementors provide. A default body quietly
makes it something else: a declaration *and* an implementation, where the second
half applies to whoever failed to mention it.

The cost is that a rule's file stops telling you what the rule does. `Rule`
carried three defaults — `check`, `check_workspace` and `is_configured` — and
with them in place there was no way to open a rule and learn which question it
answered. A file with no `check` meant "this rule's subject is the tree" or
"this rule is half-written", and the two were the same absence. Nothing
distinguished an implementor that had considered a method and found the default
right from one that had never heard of it.

`is_configured` was the sharpest case. Its default was `true`, so every rule
added to this tool was **configured because nobody said otherwise**. A rule that
could not possibly run would still join the registry and report nothing wrong —
which is precisely the silent pass this tool exists to catch, sitting in the
tool itself. [ADR-WorkspaceRuleSeam](ADR-WorkspaceRuleSeam.md) argued the
defaults were what kept the seam cheap. They were, and this is what the cheapness
bought.

## Decision

**A trait declares; it does not implement.** No method in a `trait` declaration
may have a default body.

Syntactically: for each `Item::Trait`, any `TraitItem::Fn` whose `default` is
`Some(_)` is an offence. The offence lands on the method, not the trait, because
the body is the thing that has to move.

**Only one direction needs a rule, and this was measured rather than assumed.**
The second half of the requirement — that every implementor implements every
method — needs no code at all. With no body to fall back on, `rustc` rejects an
incomplete impl with `E0046: not all trait items implemented`, immediately and
more precisely than this tool could. Writing a checker for it would add a
second, worse report of something the compiler already says well.

This mirrors [R009](R009-ADR-RegistryCompletenessRule.md) exactly, where the
missing-file direction was likewise already a compile error and only the silent
direction was worth a rule. In both cases half the stated requirement turned out
to be the compiler's, and in both cases that half is the one nobody needed
help with.

Scope is `src/`. Only methods are reported: an associated type and an associated
constant may carry a default without any of this being true of them, since
neither is behaviour and neither lets an implementor inherit a decision while
appearing to have made one.

| offence | correction |
|---|---|
| `` `Collection::is_empty` has a default body, so an implementor that says nothing about it cannot be told from one that chose it `` | move the body into each implementor |

## Forcing constraints / Evidence

Run across the eight repositories in the family, the rule found **four
offences**, and this was verified by hand before the rule was believed.

| repository | trait methods in `src/` | default bodies |
|---|---|---|
| `stern4rust` | 3 | **3** — `Rule::check`, `Rule::check_workspace`, `Rule::is_configured` |
| `etheram-core` | 22 | **1** — `Collection::is_empty` |
| `grip4rust` | 9 | 0 |
| `crap4rust` | 6 | 0 |
| `braintax4rust` | 5 | 0 |
| `twin4rust` | 1 | 0 |
| `iceberg4rust` | 0 | 0 |
| `slotgate` | 0 | 0 |

The six repositories that are not `stern4rust` or `etheram-core` hold **21 trait
methods between them and not one default body**. The standard was already being
kept everywhere, unwritten, by everyone — except in the tool that was going to
enforce it, which held three quarters of the family's total violations.

That is the whole justification. A rule that codifies what a codebase already
does is not a new constraint; it is a guarantee that the next person cannot
quietly stop doing it. The single case in `etheram-core` is a genuine one:
`Collection::is_empty` defaults to `self.len() == 0`, which is right for every
collection until it is not, and no implementor of `Collection` has ever recorded
whether it agrees.

**The cost was paid here and it is the real number: 27 bodies.** Removing three
defaults meant all fourteen rules had to answer all four questions — an empty
`check_workspace` for the nine per-file rules, an empty `check` for the five
whose subject is the tree, and `is_configured` returning `true` for the thirteen
that need no configuration. Three test fakes in `tests/rule_registry_tests.rs`
grew the same bodies.

Twenty-seven bodies to make fourteen decisions visible is a good trade, but it
is not a free one, and it is the reason this rule is worth an ADR rather than a
line in a style guide.

## Rejected alternatives

**Allow a default when the trait has exactly one implementor.** Rejected: the
count is not knowable from the file the rule is reading, and a rule whose answer
depends on the rest of the tree would report an offence appearing and
disappearing as unrelated files are added.

**Allow defaults that are trivially empty (`Vec::new()`, `true`).** Rejected,
and this is the tempting one — those three were exactly that shape. But a
trivial default hides a decision just as completely as a complex one, and
`is_configured`'s default was a bare `true` that silently configured every rule
in the tool. Triviality of the body is unrelated to the visibility of the choice.

**Check that every implementor implements every method.** Rejected on evidence:
that is `E0046`, a hard compile error. See the Decision.

**Report the offence against the trait rather than the method.** Rejected: a
trait with three defaults is three separate edits landing in three different sets
of files, and one offence naming the trait would say "something in here" where
the rule can say exactly which method.

**Extend the rule to associated constants with values.** Rejected: `const MAX:
usize = 20;` in a trait is a shared fact, not a shared decision. No implementor
inherits behaviour by leaving it alone, so none of the reasoning above applies.

**Apply it to `tests/` as well.** Rejected: a test file declares traits to stand
in for real ones, and a stand-in with a body is the shape those fakes are
supposed to have. Reporting them would report the fakes this tool's own tests
are built from.

## Consequences

**Every trait method is now written out in every implementor**, including the
ones whose answer is "nothing". That is the point, and it is also the cost: this
repository carries 27 lines it did not carry before, and each new rule will
write four methods to answer what used to take one.

**A trait with many implementors is expensive to extend.** Adding a method to
`Rule` now breaks all fourteen rules at once rather than compiling silently. This
is the correct failure — a new question every rule must answer *should* stop the
build until every rule answers it — but it makes a wide trait a wide edit, and
that pressure toward narrow traits is intended.

**It pushes against the blanket-impl idiom.** A trait providing a rich API on top
of one required method — the `Iterator` shape — cannot be written this way. That
idiom is deliberately out of scope for a house rule about a codebase's own
traits; a crate whose public surface is built that way should not adopt this rule.

### What this rule does not catch

**Blanket impls.** `impl<T: Base> Extended for T { ... }` puts a body in exactly
one place for every implementor at once, which is the thing this rule is about,
and it is invisible here because it is an `Item::Impl` rather than an
`Item::Trait`. A trait emptied of defaults can have every one of them restored
through a blanket impl and this rule will report nothing.

**Supertrait defaults.** A default body inherited through a supertrait in another
crate is not in this file, so the implementor's silence is as unreadable as ever.

**Whether the bodies moved anywhere.** The correction says to move the body into
each implementor; nothing verifies it was moved rather than deleted. `rustc`
guarantees each implementor has *a* body, not that it has the right one.

**Traits in `tests/`, and anything outside `src/`.** By decision, above.

**Anything inside a macro.** `syn` does not descend into macro token streams, so
a trait generated by a macro — or written inside one — is invisible. Nothing in
the family does this.

**Whether the trait should exist.** A trait with one implementor and no default
bodies satisfies this rule completely.

## Enforcement

`tests/rules/pure_traits_rule_tests.rs` — 12 tests covering the default body,
two defaults in one trait reported separately, the declaration-only trait, the
associated type, the defaulted associated constant, the trait inside an inline
module, the impl block that is not a trait, the unparseable file, the test file,
`check_workspace`, `is_configured`, and the rule's name.

`tests/rule_registry_tests.rs` — the four hardcoded rule-name lists include
`pure-traits`, so the rule cannot be dropped from the registry without four
failures.

`tests/runner_tests.rs::run_against_this_crate_with_its_own_header_is_clean` and
stage 2 both run the tool against this crate and require zero offences, so a
default body cannot re-enter `src/rule.rs` — or any other trait here — without
failing the build that introduces it.

The compiler enforces the other half. There is no test for it because there is
no way to write the failing case: an incomplete impl does not compile.

## Related

- [R009-ADR-RegistryCompletenessRule](R009-ADR-RegistryCompletenessRule.md) —
  the same shape of decision. One direction of a two-directional requirement was
  already a compile error, so only the silent direction became a rule.
- [ADR-WorkspaceRuleSeam](ADR-WorkspaceRuleSeam.md) — introduced
  `check_workspace` beside `check` with both defaulting to nothing. This rule
  removes those defaults; that ADR records the seam as it now stands.
- [R007-ADR-SingleImplementedTypeRule](R007-ADR-SingleImplementedTypeRule.md) —
  records in its "does not catch" that it does not treat a trait with default
  bodies as a subject. This rule is the other half of that gap.
