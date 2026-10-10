# R022-ADR-ReturnedMutationsRule

- **Status:** Accepted
- **Date:** 2026-10-09

## Context

A `&mut T` parameter is an output wearing an input's clothes. The caller hands in
a place, the function fills it, and the result of the call is the edit left
behind rather than the value handed back. Two things are lost. The reader of the
signature cannot tell what the call produces without opening the body: `fn
render(node: &Node, out: &mut String)` returns `()`, and the one thing it is for
is invisible in its type. And the caller must build the destination before there
is anything to put in it, so a half-formed value is in scope for exactly as long
as it takes the function to finish it -- a window a return value does not open.

The house standard already names this: *no `&mut` input parameters; prefer
return values*, with `&mut self` the one exception. It was a line in a style
guide, kept by hand, which is the shape of thing this tool exists to turn into a
gate.

## Decision

**A function returns its output; it does not write it through a `&mut`
parameter.** Any parameter whose declared type is -- or structurally contains --
a mutable reference is an offence, reported against that parameter.

Three things are left alone, each for a reason that is about where the edit
lives rather than about taste:

- **The `&mut self` receiver.** It carries no parameter type, so it falls out of
  the check by shape rather than by special case. A method mutating the thing it
  belongs to is the point of a method, and the standard sanctions it.
- **Trait impl methods.** `impl Display for X` takes `&mut Formatter` because
  `Display` says so; the implementor cannot change that signature, and reporting
  it names an edit that cannot be made there. Where the trait is the author's
  own, the `&mut` is reported once, at the declaration -- the one place changing
  it is both possible and sufficient. This is the [R014](R014-ADR-PureTraitsRule.md)
  and [R009](R009-ADR-RegistryCompletenessRule.md) move again: of a two-ended
  requirement, only the end nobody else owns becomes a rule.
- **Callback contracts.** `impl FnMut(&mut T)` and `fn(&mut T)` describe the
  parameter list of the callback, not of the function holding it. The type
  walker descends through references, tuples, slices, arrays and angle-bracketed
  generic arguments, but stops at a parenthesised `Fn`-trait argument and at a
  bare function pointer.

A constructor that stores the borrow -- `new(protocol: &'a mut T) -> Self`,
mutating through `&mut self` methods afterwards -- is reported like any other
parameter. It is the standard's shape one step removed: the type can own the
value instead, or the mutation can become a method on the type it borrows.

Scope is `src/`. `tests/` is exempt: a test file hands a fake a `&mut` to stand
in for a real collaborator, and a stand-in shaped that way is what those fakes
are for.

| offence | correction |
|---|---|
| `` `render::out` is a `&mut` parameter, so its result is written through an argument instead of being returned `` | `` return the new value instead of writing it through `&mut out` `` |

## Forcing constraints / Evidence

Run across the `*4rust` family with `--rule returned-mutations`, after the
trait-impl and callback exemptions:

| repository | offences | note |
|---|---:|---|
| `stern4rust` | **0** | the tool is clean under its own new rule; stage 2 proves it |
| `slotgate` | **0** | |
| `braintax4rust` | **0** | its one raw `&mut` was a `Display::fmt` formatter -- exempted |
| `grip4rust` | 1 | `resolve::visiting`, a traversal accumulator |
| `crap4rust` | 2 | an `&mut HashSet` threaded through two collectors |
| `iceberg4rust` | 4 | three `&mut String` render sinks and one `&mut usize` index |
| `twin4rust` | 5 | `&mut Option<String>` / `&mut usize` detector state |
| `dry4rust` | 31 | `&mut impl Write` report sinks and `&mut NormalizationContext` threaded down the normalizer |

The exemptions are what make these numbers the real ones rather than a wall. A
naïve check that read every `&mut` found **50** in `dry4rust` and **1** in
`braintax4rust`; the trait-impl exemption removed nineteen `Display::fmt`-shaped
signatures the author cannot change, including every offence `braintax4rust`
had. What is left is uniformly actionable -- a writer, an accumulator, a piece of
threaded state -- which is the test of whether the line was drawn in the right
place.

`stern4rust` being clean is the whole justification, as it was for
[R014](R014-ADR-PureTraitsRule.md): a rule that codifies what a codebase already
does is not a new constraint but a guarantee the next person cannot quietly stop
doing it.

## Rejected alternatives

**Report `&mut` in trait impls too.** Rejected on the evidence above: it
manufactures offences against `Display::fmt`, `Future::poll` and `io::Write`
sinks whose signatures belong to a trait, often an external one, and whose only
possible "fix" is the one the implementor is forbidden to make.

**Flag `&mut self`.** Rejected: it is the one mutating channel the house
standard keeps, and a rule against it would report every builder and every
stateful type in the family.

**Descend into callback and `Fn`-trait arguments.** Rejected: `FnMut(&mut T)`
states the callback's contract. The function that accepts it never writes
through a `&mut` itself, and demanding the callback return instead would rewrite
a contract this rule has no standing over.

**Match on the textual `&mut` rather than the parsed type.** Rejected: it cannot
tell a parameter from a `let mut`, a return type or a method body, and it cannot
tell `&mut self` from `other: &mut T` without re-implementing the parser badly.

## Consequences

**The standard now fails the build rather than a review comment.** A new `&mut`
parameter in `src/` stops the gate that introduces it, in every family
repository that adopts the rule, with a correction naming the parameter to turn
into a return value.

**Threaded-state code pays for it.** `dry4rust`'s normalizer passes `&mut
NormalizationContext` down a dozen functions; satisfying the rule means those
functions return their contribution instead, which is more returns and, where
the context is genuinely shared, pressure toward a method on the context or an
owned fold. That pressure is intended, but it is real work and the reason the
rule is configurable off per package while a codebase pays it down.

**It is on by default, and the family adopts it in two steps.** The `*4rust`
tools and `slotgate` run it from the release that ships it, and fix what it
reports. Every other repository that installs stern4rust adds
`skip = ["returned-mutations"]` to its `stern4rust.toml` for now -- 238 offences
across 13 repositories were counted before the exemptions settled, most of them
in protocol code whose owners schedule the work -- and removes the skip when it
has paid the rule down. Unpinned installs make the order matter: the skip has to
be in place by the time a repository next installs, or its stage 2 turns red.

### What this rule does not catch

- **`&mut self`, trait impl methods, and callback contracts** -- by decision,
  above.
- **Interior mutability.** `&Cell<T>`, `&RefCell<T>`, `&Mutex<T>` behind a
  `&self` mutate through a shared reference the rule never sees. This is the
  escape hatch, not an oversight.
- **Raw pointers.** `*mut T` is an unsafe pointer, not a mutable reference, and
  its own concerns are not this rule's.
- **`&mut` inside `impl Trait` / `dyn` bounds.** `impl Iterator<Item = &mut T>`
  is not descended into; the common callback shape is, the rarer bound shape is
  not.
- **Macros.** `syn` does not descend into macro token streams, so a signature
  generated by or written inside a macro is invisible.
- **Whether the body truly mutates, or the output is used.** Only the shape of
  the signature is read. A `&mut` parameter never written to is still reported --
  correctly, because the signature still lies about what the call does.
- **Anything outside `src/`.** By decision, above.

## Enforcement

`tests/rules/source/returned_mutations_rule_tests.rs` -- twenty tests covering
the free function, the inherent method, the constructor that stores the borrow,
the trait declaration, the exempt trait
impl, the exempt `&mut self` receiver, the exempt callback and bare-fn pointer,
the `&mut` nested in a tuple and in a generic argument, the shared ref, the
by-value parameter, two parameters reported separately, the inline module, the
test file, the unparseable file, `check_workspace`, `is_configured`, the name
and the requirement.

`tests/rule_registry_tests.rs` -- the four hardcoded rule-name lists include
`returned-mutations`, so the rule cannot be dropped from the registry without
four failures.

`tests/runner_tests.rs::run_against_this_crate_with_its_own_header_is_clean` and
stage 2 both run the tool against this crate and require zero offences, so a
`&mut` parameter cannot enter `src/` here without failing the build that
introduces it.

## Related

- [R014-ADR-PureTraitsRule](R014-ADR-PureTraitsRule.md),
  [R009-ADR-RegistryCompletenessRule](R009-ADR-RegistryCompletenessRule.md) --
  the same "only the end nobody else owns needs a rule" split; here it is the
  trait declaration rather than the impl that carries the offence.
- [R007-ADR-SingleImplementedTypeRule](R007-ADR-SingleImplementedTypeRule.md),
  [R008-ADR-ImportedPathsRule](R008-ADR-ImportedPathsRule.md) -- the other
  `src/`-scoped rules that read the parsed syntax tree rather than the text.
