# R025-ADR-PublicOrPrivateRule

- **Status:** Accepted
- **Date:** 2026-10-20

## Context

A function in an inherent impl has two honest states. `fn` is the type's
implementation; `pub fn` is its contract, which [R013](R013-ADR-TestedPublicApiRule.md)
holds to a test. Rust offers four more -- `pub(crate)`, `pub(super)`,
`pub(self)` and `pub(in path)` -- and each is a third state that falls between
the gates: not private to its file, yet not counted as an entry point by R013,
and out of reach of a test in `tests/`, which compiles as another crate. A
restricted function is the one kind of code this family can neither test
directly nor require to be tested.

The restricted forms also say something the type does not. Rust's privacy is
per module, not per type, so a plain `fn` in one file cannot be called from
another. A restricted function usually exists because something outside the
type's file needs it: a type whose impl is spread over several files, an
accessor that hands a component to a sibling file, a platform seam selected by
`cfg`. `pub(super)` and `pub(in path)` additionally encode the module tree, which
this tool's own corrections rearrange.

## Decision

**A function in an inherent impl is `pub` or private.** Any `fn` in an inherent
`impl` block in `src/` whose visibility is `pub(crate)`, `pub(super)`,
`pub(self)` or `pub(in ...)` is an offence.

- **Scope:** inherent impls only. A trait impl's items carry no visibility, and a
  trait declaration's carry none either. Free functions, fields, types, constants
  and modules are not this rule's, below.
- **One offence per function**, at its name: *"`election` is `pub(crate)` in
  `impl RaftProtocol`; a method is `pub` or private"*.
- **The correction reads the package**, matching call sites in its `src/` on
  name and arity, the receiver not counted, as R013 does:
  - called from nowhere outside its own file: *"make it private: `fn`"*;
  - `pub(self)`: the same, since it already means private;
  - called from another file: *"make it `pub` and give it a test, or move the
    caller's need behind the type's public contract"*. Where the impl itself is
    spread over files, the correction names
    [R026](R026-ADR-ImplementedInOneFileRule.md) instead: the restriction is a
    symptom of the spread.
- **Exempt:** `tests/`, as for every rule here.
- **Adoption is by baseline** ([ADR-Baselines](ADR-Baselines.md)): the rule
  ships on by default, a repository that cannot clear it records its current
  offences, and any new restricted function fails.

## Forcing constraints / Evidence

A census on 2026-10-20 over sixteen repositories -- the `etheram-*` six, the
seven `*4rust` tools, `slotgate`, `etheram-tools` and an
analysis workspace -- counted 6,934 functions in inherent impls
under `src/` (`tests/`, `target/` and generated code excluded). It used a
heuristic scanner, not this tool's finders, and has to be re-run with them
before the rule ships, as R024's was.

| visibility | functions |
|---|---:|
| `fn` | 1,688 |
| `pub fn` | 5,121 |
| `pub(crate)` | **119** |
| `pub(super)` | **6** |
| `pub(self)`, `pub(in ...)` | 0 |

| repository | restricted |
|---|---:|
| `etheram-embassy` | 42 |
| `etheram-ibft` | 30 |
| `etheram-raft` | 25 |
| `etheram-ibft-embassy` | 8 |
| `etheram-raft-embassy` | 6 |
| `crap4rust` | 5 |
| an analysis workspace | 3 |
| `braintax4rust`, `slotgate` | 2 each |
| `etheram-tools`, `grip4rust` | 1 each |
| `etheram-core`, `dry4rust`, `iceberg4rust`, `stern4rust`, `twin4rust` | **0** |

What the 125 are, and what each group takes:

- **38 sit in the three types whose impl spans files** -- `RecoveryControl`
  (6 files, 18), `IbftProtocol` (13 files, 12), `PlatformFlashAccess` (5
  platform files, 8). They go with the decompositions R026 asks for.
- **6 are called only in their own file** -- `ProbePool::claim`, `claim_all`
  and `all_serials`, duplicated in two repositories -- and become `fn`.
- **58 are called from other files of binary or test-support crates**: the
  raft `system-tests` harness (18), `etheram-embassy`'s hardware validation and
  contract apps (34), probe guards (6). `pub` exposes nothing there; the cost is
  the tests R013 then asks for.
- **23 are called from other files of library crates.** Ten are in the `core`
  libraries of four tools -- `crap4rust` (5), `braintax4rust` and `slotgate`
  (2 each), `grip4rust` (1) -- which exist for their own binaries, as this
  tool's does ([ADR-LibrarySurfaceIsNotAnApi](ADR-LibrarySurfaceIsNotAnApi.md)). The 13
  that would widen a real surface -- seven in `etheram-raft`'s `node`
  (`RaftProtocol`'s component accessors and handlers), three in the analysis
  repository, two in `etheram-ibft-embassy`, one in `etheram-tools` -- are
  design work: a tested contract, or callers that own what they need.

**Re-run with this tool's finders**, the built rule against the sixteen
repositories on 2026-10-21: **124 offences**, against the census's 125 --
`pub(crate)` 118, `pub(super)` 6 -- sorted by the correction each is given:

| correction | offences |
|---|---:|
| `pub` and a test: another file calls it | 80 |
| `implemented-in-one-file`: its type's impl is spread | 38 |
| private: nothing outside its file calls it | 6 |

| repository | offences |
|---|---:|
| `etheram-embassy` | 42 (8 spread) |
| `etheram-ibft` | 30 (all spread) |
| `etheram-raft` | 25 |
| `etheram-ibft-embassy` | 8 (3 private) |
| `etheram-raft-embassy` | 6 (3 private) |
| `crap4rust` | 4 |
| an analysis workspace | 3 |
| `grip4rust`, `slotgate` | 2 each |
| `braintax4rust`, `etheram-tools` | 1 each |
| `etheram-core`, `dry4rust`, `iceberg4rust`, `stern4rust`, `twin4rust` | **0** |

The 38 spread are the census's -- `RecoveryControl`'s 18, `IbftProtocol`'s 12,
`PlatformFlashAccess`'s 8 -- and so are the six own-file ones. The tools' nine
are all called from another file of their own library. Two of `grip4rust`'s,
`StructRegistry::self_ty_name` and `is_trait_object_type`, were `pub(crate)` free
functions -- outside this rule's scope -- until that repository's adoption of
[R024](R024-ADR-OwnedFunctionsRule.md) made them associated functions, keeping
the visibility.

## Rejected alternatives

**Allowing `pub(crate)` and banning only the module-tree forms.** It would clear
6 offences and keep the third state where 119 of the 125 live. Rejected: the gap
in R013 is `pub(crate)`'s, not `pub(super)`'s.

**Exempting binary and test-support crates**, where `pub` reaches no one.
Rejected: what the rule buys there is not encapsulation but the two-state
discipline and R013's reach, and a crate's kind is configuration this tool does
not otherwise read.

**Correcting every offence to `pub`.** Rejected: it would satisfy the rule and
widen thirteen library surfaces and three spread types' internals. The
correction names the honest alternatives instead.

## Consequences

**`fn` and `pub fn` are the only two answers**, and with R026 beside it, `fn`
means private to the type rather than to whatever module the impl happens to
sit in. R013 then reaches every function that crosses a file.

**The first run fails eleven repositories.** Adoption is by baseline; the cheap
offences (the six `pub(super)`, the six own-file functions) clear at once, the
spread types with R026, the rest repository by repository.

### What this rule does not catch

- **Restricted visibility anywhere else** -- free functions, fields, types,
  constants, modules. R024 already leaves few free functions beside a subject;
  the others are separate questions.
- **A `pub fn` that should have been private.** The rule removes the middle
  state; R013 and review judge the two that remain.
- **Functions generated by a macro**, invisible to `syn`.

## Enforcement

`tests/rules/source/public_or_private_rule_tests.rs` -- fourteen tests: a
`pub(crate)` method reported at its name, with the description; each restricted
method reported; the correction private for a method called only in its own
file and for `pub(self)`, `pub` with a test for one another file calls, a call
of another arity not counting, and `implemented-in-one-file` for a spread
type's; `pub` and private methods, `tests/` and an unparseable file left alone;
`check` answering nothing per file; `is_configured`, the name and the
requirement.

`tests/finding/parsing/restricted_method_finder_tests.rs` (six) pins what is
found: each restricted form as written, `pub(in crate::store)` included, its
call, line and owner; a method in an inline module; nothing for `pub` or
private methods, a restricted free function, or a trait impl.
`tests/finding/model/restricted_method_tests.rs` pins `pub(self)` as private
already. The pieces shared with other rules have tests of their own:
`ImplBlocks::inherent` in `impl_blocks_tests.rs` and
`PublicEntryPointFinder::of_signature` in `public_entry_point_finder_tests.rs`.

`tests/rule_registry_tests.rs`' four rule-name lists include
`public-or-private`, and `tests/runner_tests.rs`' `--rules` listing holds
twenty-nine. Stage 2 runs this tool, built from the checkout, against this
crate with every rule applied and zero offences.

## Related

- [R013-ADR-TestedPublicApiRule](R013-ADR-TestedPublicApiRule.md) -- the gate a
  restricted function escapes.
- [R026-ADR-ImplementedInOneFileRule](R026-ADR-ImplementedInOneFileRule.md) --
  the companion: a type's impl in one file, so `fn` can mean private to it.
- [R024-ADR-OwnedFunctionsRule](R024-ADR-OwnedFunctionsRule.md) -- the census
  method and the baseline adoption this rule follows.
- [ADR-Baselines](ADR-Baselines.md) -- how a repository adopts it.
