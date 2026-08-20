# R018-ADR-DeclaredByNameRule

- **Status:** Accepted
- **Date:** 2026-08-20

## Context

`mod alpha;` reaches `alpha.rs` or `alpha/mod.rs`. That convention is what
[R009](R009-ADR-RegistryCompletenessRule.md) resolves declarations by, and it is
the only thing letting that rule tell a declared file from an orphan.

`#[path = "..."]` breaks it. A module reached through an explicit path resolves
to a file no convention predicts, and `registry-completeness` reports that file
as **never compiled** when it compiles perfectly well.

R009 accepted the gap, and said so in its own "does not catch":

> `#[path = "..."]` is not understood. A file declared through an explicit path
> attribute would be reported as undeclared. The house rules forbid `#[path]` in
> `all_tests.rs`, and nothing in the family uses it, so this is a known
> false-positive class rather than an observed one.

That reasoning rests on a convention **nothing enforced**. `CLAUDE.md` says "in
`all_tests.rs`, reference test files one by one without `#[path = ...]`",
`OPEN_POINTS.md` records the false-positive class, and `RULES.md` repeats it —
three documents describing a standard no rule checked.

## Decision

**A module is declared by name.** `#[path = "..."]` on a `mod` declaration is an
offence.

**It applies to the whole package, not only to registries.** The house standard
names `all_tests.rs` because that is where the temptation is, but the harm does
not depend on the file: a `mod` in an ordinary source file is resolved by the
same convention and misread in the same way. This is the reason the rule is not
folded into `module-registry` or `tests-layout` — see Rejected alternatives.

The walk descends into inline modules, since an attribute one level down is as
invisible to name resolution as one at the top.

**`#[cfg_attr(unix, path = "...")]` is deliberately left alone.** A
platform-gated module is the one honest use of the attribute; it cannot resolve
by name on every platform anyway, and reporting it would accuse correct code —
the direction every rule here refuses to lean. Mechanically it falls out for
free: a `cfg_attr` is a `Meta::List` named `cfg_attr`, so it never matches a
`Meta::NameValue` named `path`.

| offence | correction |
|---|---|
| `` `mod alpha` is reached through `#[path = "elsewhere/other.rs"]`, so the file it declares cannot be found from its name `` | move `elsewhere/other.rs` to `alpha.rs` beside this file and drop the `#[path]` attribute |

The correction names **both** files, because the fix is a move plus a deletion
rather than an edit to the declaration. `expected` carries the implied filename
for a consumer of the JSON report.

## Forcing constraints / Evidence

**Zero offences across ten repositories** — the eight in the family plus
`etheram-ibft` and `etheram-raft`. Not one `#[path]`, and not one
`cfg_attr(..., path = ...)` either.

That confirms R009's claim empirically rather than by assertion, and it is the
whole case for the rule: **R009 is relying on this being true.** A rule that
holds a convention another rule already depends on is worth having at zero,
because the day it stops being true, the failure does not look like a `#[path]`
attribute — it looks like `registry-completeness` accusing an innocent file of
never being compiled. The wrong answer would arrive somewhere else entirely,
which is the hardest kind to trace.

[R011](R011-ADR-DirectorySubfolderCountRule.md) and
[R015](R015-ADR-TestFileNamePostfixRule.md) are the precedent for shipping a
rule that finds nothing and saying so plainly.

## Rejected alternatives

**Fold it into `module-registry`.** Rejected on two counts. That rule is scoped
to `src/` registries and explicitly leaves `tests/` to `tests-layout`, so the
check would have to be written **twice** — the duplication `OPEN_POINTS.md`
already regrets elsewhere. And its offence sentence says an item "does not
belong in a module registry, which holds the header, inner attributes,
`extern crate alloc;` and `pub mod` declarations" — but `#[path = "x.rs"] pub
mod x;` **is** a `pub mod` declaration. The host rule's own wording would
contradict the offence.

**Fold it into `tests-layout`.** Rejected for the same wording reason, and
because it covers only the tests tree while `registry-completeness` is misled in
both.

**Fold it into `registry-completeness`**, the rule that suffers. Rejected: that
rule answers "is every file declared", and this answers "how is it declared".
Two questions with two corrections, and R009 made a point of splitting exactly
this kind of pair.

**Report it only in registries**, matching `CLAUDE.md`'s literal wording.
Rejected: the standard names `all_tests.rs` because that is where it comes up,
not because a `#[path]` elsewhere is acceptable.

**Also report `#[cfg_attr(..., path = ...)]`.** Rejected: it is the legitimate
case, and a platform-gated module is not something this repository wants to
forbid. Recorded below as not caught.

**Leave it unenforced, as R009 assumed.** Rejected now that the cost is visible:
the convention was load-bearing for another rule and had nothing holding it up.

## Consequences

**A repository that genuinely needs `#[path]`** — vendored layouts, generated
trees — must `--skip declared-by-name`, and a skipped rule is named as skipped
in the report. That is rule selection working as designed rather than a
concession.

**It uses a slot in a bounded set.** `CLAUDE.md` states the rule set will be
"fifteen to twenty"; this is the eighteenth, and `ROADMAP.md` still lists two
planned. The set is close to full, and that was weighed against putting a
sentence inside `module-registry` that its own ADR contradicts.

### What this rule does not catch

**`#[cfg_attr(..., path = "...")]`**, by decision. A module gated onto a path by
configuration is invisible here, and `registry-completeness` would still misread
it on the platform where it applies.

**A `#[path]` inside a macro.** `syn` does not descend into macro token streams,
so an attribute generated by one is never seen.

**Whether the file the attribute points at exists.** The rule reads the
declaration, not the filesystem; a `#[path]` naming nothing is `rustc`'s
`E0583`, reported better there.

**`#[path]` on anything other than a `mod`.** It has no meaning elsewhere, so
there is nothing to report.

**The false positive it prevents, once it has been skipped.** A repository that
skips this rule and uses `#[path]` gets `registry-completeness`'s wrong answer
back, and nothing connects the two. That linkage lives here and in
`OPEN_POINTS.md`.

## Enforcement

`tests/rules/declared_by_name_rule_tests.rs` — 11 tests covering the path
attribute, the module declared by name, the nested module, the attribute in a
test file, the `cfg_attr` exemption, an unrelated attribute, several attributes
in one file, an unparseable file, `check_workspace`, `is_configured`, and the
rule's name. The reported case asserts the exact correction naming both files.

`tests/rule_registry_tests.rs` — the four hardcoded rule-name lists include
`declared-by-name`.

Stage 2 runs the tool against this crate at zero offences.

## Related

- [R009-ADR-RegistryCompletenessRule](R009-ADR-RegistryCompletenessRule.md) —
  the rule this protects. It resolves declarations by convention and gives a
  confident wrong answer when `#[path]` breaks that convention; it recorded the
  gap and relied on a standard nothing enforced.
- [R006-ADR-ModuleRegistryRule](R006-ADR-ModuleRegistryRule.md) and
  [R003-ADR-TestsLayoutRule](R003-ADR-TestsLayoutRule.md) — the two rules this
  was deliberately not folded into, and why.
- [ADR-RuleSelection](ADR-RuleSelection.md) — how a repository that needs
  `#[path]` opts out.
