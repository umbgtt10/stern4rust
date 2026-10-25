# R038-ADR-InjectedDependenciesRule

- **Status:** Accepted, 2026-10-25
- **Date:** 2026-10-25

## Context

The house standard says it in prose: *"make constructors depend on traits, not directly on concrete
implementations"* and *"ALL dependencies are injected through the SINGLE constructor and stored in the struct"*.
Nothing checks it. The shape it forbids is easy to write and easy to miss in review: `crap4rust`'s `App` declares
its five collaborators as traits -- `Box<dyn PackageResolver>`, `Box<dyn Reporter>` and three more -- and then its
`new` builds the concrete ones itself, beside a `with_deps` that takes them as parameters. The type is testable only
through the second constructor, and the first hides five dependencies from every caller.

grip4rust sees it: `App::new` scores a grip of 0 with five hidden dependencies, `with_deps` 95. But grip4rust counts
every concrete constructor call, a value a function computes and returns as much as a collaborator it hides -- 42 of
`crap4rust`'s 196 functions have "hidden dependencies" -- it scores rather than reports, and no repository but its own
gates on it.

The owner, 2026-10-25: how rare a deviation is today is no reason not to have the rule. A rule is the guard for the
code not yet written.

## Decision

**A type does not build its own dependencies: a field declared as a trait receives only what the caller passed in.**

- **A trait-typed field** is one whose type is, or wraps in `Box`, `Rc`, `Arc` or `Option`, a `dyn Trait` or a type
  parameter bounded by a trait (`T: Port`, in the parameter list or a `where` clause). A field of a concrete type is
  not this rule's.
- **Passed in** means a parameter of the method, as it is or as `Some(p)`, `Box::new(p)`, `Rc::new(p)`,
  `Arc::new(p)`, `p.clone()`, `Rc::clone(&p)`, `Arc::clone(&p)` or `p.into()`; or `None`. Anything else the method
  builds -- `Box::new(StdoutReporter::new())`, `NoOpCoordinator`, `Registry::current()`, a value read from
  configuration -- is the type building its own dependency.
- **Where it applies:** every constructor -- [R027](R027-ADR-OrderedImplFunctionsRule.md)'s definition, no receiver
  and a return type naming `Self` -- and every `Default::default`, `From::from` and `TryFrom::try_from` for the type;
  every value a constructor passes to another constructor of its own type, which is how `new` hands `with_deps` what
  it built; and every assignment `self.field = ...` to a trait-typed field in any other method.
- **A builder is the composition mechanism, and is not judged.** A type with a method `build(self)` that consumes it
  and returns another type -- or a `Result` or `Option` of one -- exists to choose and assemble parts: its defaults,
  its `with_*_variant` steps and its `build` may build whatever they like. The rule judges the product: the constructor
  `build` calls must take every part as a parameter, and does.

*As built, 2026-10-25:* a `Vec` and a reference wrap a trait as a `Box` does -- the registries the census counts hold
`Vec<Box<dyn Rule>>` -- and an empty vector, `Vec::new()` or `vec![]`, holds nothing, as `None` does; a parameter's
field is passed in, a part handed over inside a bundle of them; and a type parameter bounded only by value traits --
`T: Clone + Debug` -- is data the type carries, not a seam. A local is what it was bound to, by name or by
destructuring (`let Parts { wal, .. } = parts`), and a name rebound from itself reads its earlier binding: the census
of 2026-10-25 found `etheram-ibft`'s `IbftProtocol::new` unpacking the caller's parts that way.

One offence per field and method, at the field's initialiser:

| offence | correction |
|---|---|
| `` `App::new` builds `StdoutReporter` for `reporter`, a field declared as a trait `` | `take it as a parameter and build it where the parts are composed: main, a builder, a test` |
| `` `App::new` passes `CargoPackageResolver`, which it built, to `App::with_deps` `` | `make with_deps the constructor and compose the parts in main` |
| `` `Storage::reset` assigns `FlashRegistry::current()` to `flash`, a field declared as a trait `` | `take the replacement as a parameter` |

Exempt: `tests/`, `benches/` and `examples/` -- a test is a composition root -- and generated code a package excludes.
A binary's `main` composes and is not a constructor.

## Forcing constraints / Evidence

A census on 2026-10-25, read-only, with a heuristic program -- trait-typed fields read from struct declarations,
constructors and their struct literals found by brace matching -- over the sixteen repositories; to be re-run with the
rule's finders. Of 1,712 constructors, **about 28 fill one of their own trait-typed fields with something they built.
13 are builder defaults** -- `etheram-raft`'s and a protocol repository's `node-infra` builders, a `Default` that
pre-fills an in-memory implementation -- and the builder shape leaves them alone. **About 15 remain:**

| shape | constructors | where |
|---|---:|---|
| a `new` building what `with_deps` takes | 3 | `App` in `crap4rust`, `grip4rust` and `braintax4rust` |
| a null object chosen inside the type | 4 | bootstrappers building a no-op coordinator in `new` and `inactive`: `etheram-raft`'s `raft-faction` and a protocol repository |
| a global looked up | 2 | two firmware flash storages taking a registry's current instance |
| a dependency built from configuration | 2 | `stern4rust`'s `RuleRegistry::from_config`, `dry4rust`'s `AnalysisOutput::from_config` |
| a `Default` building an engine | 2 | in a protocol repository |
| a fixed list of parts | 2 | `stern4rust`'s `RewriterRegistry::all`; a preset in an analysis workspace |

The global lookups are the most expensive of them to keep: a test cannot replace a dependency the type fetches for
itself.

## Rejected alternatives

**grip4rust as the gate.** It already measures hidden dependencies, but as every concrete construction in a
function's body, results included, and as a score: a per-function threshold would report every function that builds
its return value.

**Any constructor building any concrete value.** Building is what composition roots do -- `etheram-raft`'s
`RaftCluster::new`, a node set-up assembling a whole node -- and what most constructors do for their own concrete
state. The harm is narrower: a field that promises a seam, a trait, and a constructor that closes it.

**Exempting composition roots by name or path.** A list someone maintains. Judging only a type's *own* trait-typed
fields leaves composition roots alone without one: they build other types' parts and pass them in.

**Every collaborator field a trait.** Requiring that a field holding another object be declared as a trait would
catch the concrete dependency this rule does not. Components are concrete by design -- `RaftProtocol`'s are -- and
the rule would ask for a trait per component. Not adopted.

**One constructor per type, named `new`**, as the means of enforcing "the single constructor". It would rename 388
constructors whose names say something, and still not stop the one `new` building its own parts.

## Consequences

**Composition moves to where it belongs.** The tools' `App::new` disappears: `with_deps` becomes the constructor, and
`main` builds the parts. A null object is passed by whoever wants one; a global is injected; a registry takes its
list as a parameter.

**Builders keep everything they do today.** Their defaults, their variants and their `build` are untouched; only the
product's constructor is judged, and the builders' products already take their parts as parameters.

### What this rule does not catch

- **A concrete collaborator** -- a field of a concrete object type, built inside. The rule judges what a type
  declared as a seam, not whether it should have declared one.
- **A dependency used without being stored**: a method calling `Registry::current()` and using the result in place.
  grip4rust's hidden dependencies see it; this rule reads fields.
- **A builder in name only.** A type that gains a `build(self)` is read as a builder; a product hiding behind one is
  visible in review and in grip4rust's score.

## Enforcement

Test-first, in `tests/rules/types/injected_dependencies_rule_tests.rs`: a trait-typed field filled from a parameter
in each passed-in shape, and with `None`, left alone; one filled with a value the constructor built, reported, through
a struct literal, through a call to another constructor of the same type, from a `Default` impl and from a `From` impl;
an assignment in a `&mut self` method reported; a generic field bounded by a trait, and one in a `where` clause; a
concrete field left alone; a builder's defaults, variants and `build` left alone; `tests/` and an unparseable file
left alone; the explanation, the name, `is_configured` and the requirement. It shares R035's classifier, which tells a
builder from an object.

## Related

- [R035-ADR-RecordsAndObjectsRule](R035-ADR-RecordsAndObjectsRule.md) -- records, objects, and the builder this rule
  recognises.
- [R034-ADR-InjectedTimeRule](R034-ADR-InjectedTimeRule.md) -- the same principle for time: through a port, never
  fetched.
- [R027-ADR-OrderedImplFunctionsRule](R027-ADR-OrderedImplFunctionsRule.md) -- what a constructor is.
- [R022-ADR-ReturnedMutationsRule](R022-ADR-ReturnedMutationsRule.md) -- amended for constructors storing a `&mut`.
