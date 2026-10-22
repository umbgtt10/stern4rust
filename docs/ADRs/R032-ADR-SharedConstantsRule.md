# R032-ADR-SharedConstantsRule

- **Status:** Accepted
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
  | `pub`, read by one other file and its own, beside other items | *"move it into a constants file"*, or private if the other can do without it |
  | `pub`, shared, beside other items | *"move it into a constants file"* |
  | restricted visibility | *"make it private, or `pub` and shared"* |

  A file's declaration is not a read: the names a file reads are the ones in
  its paths, imports, patterns and macro tokens. So the third row is a constant
  its own file uses and one other file does too -- two files sharing a fact,
  whose home is a constants file, and not the other file, which the second row
  would have offered: moved there, the declaring file would have to read it
  back. The census found 28 of them, among them `dry4rust`'s baseline-file name,
  which a test reads.
- **Module-level** is a file's own top-level items. An inline module is not
  descended into, as [R031](R031-ADR-LeadingConstantsRule.md) does not.

- **Adoption is by baseline** ([ADR-Baselines](ADR-Baselines.md)).
- **Exempt:** `tests/`, and the registries `lib.rs` and `mod.rs`, as
  [R031](R031-ADR-LeadingConstantsRule.md) exempts them.

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

**Re-run with this tool on 2026-10-22: 228 offences**, every difference from
the 255 accounted for. 27 of `etheram-embassy`'s and `crap4rust`'s one were
associated constants -- `impl Verdict { pub const STARTED_MARKER ... }` --,
which the decision leaves alone and the census, matching indented lines, had
counted. Two of the analysis workspace's three went with the code it retired
since. And `etheram-ibft-embassy`'s fourteen are seventeen counted per
workspace: thirteen in its root workspace, one in `demo/` and three in
`tools/cluster-console/`, each a workspace of its own whose readers are its
own -- the three are constants only one file of the console reads.

| repository | offences | none | one | here and one | beside | restricted |
|---|---:|---:|---:|---:|---:|---:|
| `etheram-embassy` | 100 | 14 | 21 | 2 | 17 | 46 |
| `etheram-ibft` | 91 | 3 | 27 | 19 | 42 | 0 |
| `etheram-ibft-embassy`, three workspaces | 17 | 0 | 8 | 3 | 2 | 4 |
| `etheram-raft-embassy` | 10 | 2 | 1 | 2 | 5 | 0 |
| `grip4rust` | 5 | 0 | 4 | 0 | 1 | 0 |
| `etheram-tools`, `dry4rust` | 2 each | 0 | 0 | 1 each | 1 each | 0 |
| the analysis workspace | 1 | 0 | 0 | 0 | 1 | 0 |
| `etheram-core`, `etheram-raft`, `braintax4rust`, `crap4rust`, `iceberg4rust`, `slotgate`, `stern4rust`, `twin4rust` | **0** | | | | | |

A run reads every member's files once more to build the index; the largest
workspace, `etheram-ibft`'s, takes 0.8 seconds with it.

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

`tests/rules/source/shared_constants_rule_tests.rs` -- eighteen tests: a `pub`
constant read by no other file, by one, by its own file and one other, and by
two beside a type, each reported at its name with its description and
correction pinned word for word; `pub(crate)` and `pub(in crate::node)` reported
as written; two unread constants reported each; a private constant, a shared
one in a constants file -- imports, constants, a static, a type alias --, one
read only from `tests/` and a sibling crate, a static, an associated constant,
an inline module, a registry, a test file and an unparseable file left alone;
`check_workspace` answering nothing; the explanation breaking the rule and its
mend not; `is_configured`, the name and the requirement.

`tests/finding/parsing/read_names_tests.rs` (seven) pins what reading is -- an
import, a rename, a path inside a type and an expression, a `match` arm, a
macro's tokens -- and that a declaration is not, nor a file that does not
parse; and the index naming each reader by its path.
`tests/finding/model/name_readers_tests.rs` (four) pins the answer from inside a
package: the asking file left out of `others`, the readers named
workspace-relative, and `reads` telling whether the asking file reads a name.
`tests/workspace_sources_tests.rs` (two) pins the files the index is built from:
every member's, named from the workspace root, each through its own exclusions.

`tests/runner_tests.rs`' `run_reporting_over_a_workspace_counts_a_sibling_crates_readers`
builds a two-member workspace whose second member reads the first's constants,
and runs the tool over it whole and scoped to the first: both find the one
constant a single file of the sibling reads, and not the one two files read.
The four rule-name lists in `tests/rule_registry_tests.rs` include
`shared-constants`, and the `--rules` listing holds thirty-three. Stage 2 runs
this tool, built from the checkout, against this crate with every rule applied
and zero offences.

## Related

- [R025-ADR-PublicOrPrivateRule](R025-ADR-PublicOrPrivateRule.md) -- the same two
  states, for methods.
- [R031-ADR-LeadingConstantsRule](R031-ADR-LeadingConstantsRule.md) -- where in a
  file a constant goes.
- [R030-ADR-TypedErrorsRule](R030-ADR-TypedErrorsRule.md) -- the per-package skip
  this rule borrows for libraries read from outside.
