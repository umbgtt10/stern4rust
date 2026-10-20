# R032-ADR-SharedConstantsRule

- **Status:** Proposed
- **Date:** 2026-10-20

## Context

A module-level constant has two honest states, as a method does in
[R025](R025-ADR-PublicOrPrivateRule.md). It is one file's: private, read where it
is declared. Or it is a fact several files share: public, and then it needs a
home that is not any one of its readers -- a file of constants, which a reader
looking for the number finds by name.

What the family does instead: a `pub const` declared beside a type and read by
one other file, which could have held it privately; a widely shared constant
living in whichever file happened to need it first, so its readers depend on a
file for its number and not its type; `pub(crate)` on a constant, the third
state R025 removes from methods.

## Decision

**A module-level constant is private, or public and shared from a constants
file.** In `src/`, a module-level `const` is an offence unless it is:

- **private** -- no visibility; or
- **`pub`, in a constants file, and read by two or more other files** of the
  workspace.

Read as:

- **A constants file** holds imports, constants, statics and type aliases, and
  no struct, enum, trait, impl, function or macro. Its name is not prescribed:
  [R023](R023-ADR-NamedSubjectRule.md) has no subject to name it after, and the
  correction suggests `<topic>_constants.rs`.
- **Read by** is a name match across the workspace's files, `tests/` included --
  a constant both the code and its tests depend on is shared -- the way
  [R013](R013-ADR-TestedPublicApiRule.md) matches calls. Matched across the
  workspace and not the crate: a constant read only by a sibling crate is shared,
  and counting by crate condemned those.
- **Restricted visibility** -- `pub(crate)`, `pub(super)`, `pub(in ...)` -- is an
  offence on its own: private or `pub`.
- **Statics are left alone.** A `static` is runtime state -- a channel hub, a
  cell an executor fills -- and it lives with the types that serve it; moving it
  to a constants file would separate state from its owner.
- **Associated constants are left alone**: `Type::LIMIT` is the type's contract,
  and [R027](R027-ADR-OrderedImplFunctionsRule.md) places it.
- **A library whose constants other repositories read** cannot be judged from
  inside one repository; such a package skips the rule in `stern4rust.toml`, as
  harnesses skip [R030](R030-ADR-TypedErrorsRule.md).
- **One offence per constant**, at its name, the correction chosen by what was
  found:

  | found | correction |
  |---|---|
  | `pub`, read by no other file | *"make it private"*, or delete it |
  | `pub`, read by one other file | *"move it into `<file>` and make it private"* |
  | `pub`, shared, beside other items | *"move it into a constants file"* |
  | restricted visibility | *"make it private, or `pub` and shared"* |

- **Adoption is by baseline** ([ADR-Baselines](ADR-Baselines.md)).
- **Exempt:** `tests/`, registries.

## Forcing constraints / Evidence

The census of [R031](R031-ADR-LeadingConstantsRule.md), the same sixteen
repositories, found **1,147** module-level constants under `src/`:

| found | constants |
|---|---:|
| private | 817 |
| `pub`, shared, in a constants file | 75 |
| `pub`, read by one other file | 98 |
| `pub`, shared, beside other items | 76 |
| restricted visibility | 62 |
| `pub`, read by no other file in its repository | 19 |

So **255 offences**: `etheram-embassy` 127, `etheram-ibft` 91,
`etheram-ibft-embassy` 14, `etheram-raft-embassy` 10, `grip4rust` 5, an analysis
workspace 3, `etheram-tools` and `dry4rust` 2 each, `crap4rust` 1.
`etheram-core`, `etheram-raft`, `braintax4rust`, `iceberg4rust`, `slotgate`,
`stern4rust` and `twin4rust` have none.

Three findings shaped the decision:

- **Counting by crate was wrong.** The first pass found constants "read by no
  other file" that sibling crates of the same repository read; counted across
  the repository, 37 of the public constants and statics it had called unread
  are shared.
- **Statics push back.** The same census found 125 module-level statics, among
  them channel hubs -- `TRANSPORT_HUB`, `CLIENT_REQUEST_HUB` -- declared in a file
  named after the hub, beside the types it carries. That is state with its
  owner, not a misplaced number.
- **`config.rs` files** hold an application's constants beside a function or
  two; under this rule the constants go to a constants file and the functions
  stay, which is the split the name `config` was already promising.

The 19 constants read nowhere in their repository are dead code or a public API
for another repository; which, the census cannot tell, and the per-package skip
is for the second.

## Rejected alternatives

**Allowing `pub` constants anywhere** and judging only the restricted forms. It
would leave 193 of the 255 offences -- a shared fact homed in one of its readers,
and a public constant one reader could hold privately. Rejected.

**Counting references within the crate.** Rejected: it accuses a constant every
sibling crate reads.

**Including statics.** Rejected on the census, above.

**Requiring the constants file's name.** Deferred: it was not measured, and
R023's machinery has nothing to name such a file after.

## Consequences

**A constant has one home.** Private at the top of the file that uses it
([R031](R031-ADR-LeadingConstantsRule.md)), or shared from a constants file a
reader finds by name. Together with the open point on repeated literals, a
number that appears in two places is a number with a name in one.

**Two firmware repositories pay most**, their applications' `config.rs` files and
protocol tags being where the shared constants are; the rest is small.

### What this rule does not catch

- **Same-named constants in different modules**, which a name match conflates;
  matching on the import path where there is one narrows it.
- **A constant read through a glob import** or a macro.
- **Statics and associated constants**, by decision.

## Enforcement

To be written test-first when the rule is built: a private constant left alone;
a `pub` constant read by none, one and two other files, in a constants file and
beside other items, each with its correction; references from `tests/` counted;
a sibling crate's reference counted; restricted visibility; statics and
associated constants left alone; a package skipping the rule; registries and
`tests/` exempt; the unparseable file; `check_workspace`, `is_configured`, the
name and the requirement; `tests/rule_registry_tests.rs` updated. This crate has
no offence today.

## Related

- [R025-ADR-PublicOrPrivateRule](R025-ADR-PublicOrPrivateRule.md) -- the same two
  states, for methods.
- [R031-ADR-LeadingConstantsRule](R031-ADR-LeadingConstantsRule.md) -- where in a
  file a constant goes.
- [R030-ADR-TypedErrorsRule](R030-ADR-TypedErrorsRule.md) -- the per-package skip
  this rule borrows for libraries read from outside.
