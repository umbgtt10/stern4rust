# Rules

Thirty-seven rules, each independent, each naming itself in the report. This is the
reference; the reasoning behind each one is in its ADR.

Every offence carries a **correction** as well as a description — what to do,
not only what is wrong — and the field is required, so a rule cannot be added
without answering it.

Each rule also carries an **explanation** — what it wants, a scrap of source
that breaks it and the same scrap put right — and that field is required too.
`cargo stern4rust --rules` prints all thirty-seven, in text or with
`--format json`, without scanning anything. See
[ADR-RulesExplainThemselves](ADRs/ADR-RulesExplainThemselves.md).

| rule | ADR | needs configuration |
|---|---|---|
| `readable-source` | [R004](ADRs/R004-ADR-ReadableSourceRule.md) | no |
| `arrange-act-assert` | [R017](ADRs/R017-ADR-ArrangeActAssertRule.md) | no |
| `declared-by-name` | [R018](ADRs/R018-ADR-DeclaredByNameRule.md) | no |
| `directory-file-count` | [R010](ADRs/R010-ADR-DirectoryFileCountRule.md) | optional, default 20 |
| `directory-subfolder-count` | [R011](ADRs/R011-ADR-DirectorySubfolderCountRule.md) | optional, default 5 |
| `imported-paths` | [R008](ADRs/R008-ADR-ImportedPathsRule.md) | no |
| `injected-time` | [R034](ADRs/R034-ADR-InjectedTimeRule.md) | no |
| `records-and-objects` | [R035](ADRs/R035-ADR-RecordsAndObjectsRule.md) | no |
| `record-home` | [R037](ADRs/R037-ADR-RecordHomeRule.md) | no |
| `injected-dependencies` | [R038](ADRs/R038-ADR-InjectedDependenciesRule.md) | no |
| `registry-completeness` | [R009](ADRs/R009-ADR-RegistryCompletenessRule.md) | no |
| `paired-test-file` | [R016](ADRs/R016-ADR-PairedTestFileRule.md) | no |
| `test-file-name-postfix` | [R015](ADRs/R015-ADR-TestFileNamePostfixRule.md) | no |
| `test-file-structure` | [R002](ADRs/R002-ADR-TestFileStructureRule.md) | no |
| `test-free-source` | [R005](ADRs/R005-ADR-TestFreeSourceRule.md) | no |
| `tests-layout` | [R003](ADRs/R003-ADR-TestsLayoutRule.md) | no |
| `timeless-tests` | [R033](ADRs/R033-ADR-TimelessTestsRule.md) | no |
| `module-registry` | [R006](ADRs/R006-ADR-ModuleRegistryRule.md) | no |
| `ordered-imports` | [R019](ADRs/R019-ADR-OrderedImportsRule.md) | no |
| `spdx-matches-manifest` | [R020](ADRs/R020-ADR-SpdxMatchesManifestRule.md) | `license` in the manifest |
| `workspace-dependencies` | [R021](ADRs/R021-ADR-WorkspaceDependenciesRule.md) | no |
| `single-implemented-type` | [R007](ADRs/R007-ADR-SingleImplementedTypeRule.md) | no |
| `named-subject` | [R023](ADRs/R023-ADR-NamedSubjectRule.md) | no |
| `owned-functions` | [R024](ADRs/R024-ADR-OwnedFunctionsRule.md) | no |
| `pure-traits` | [R014](ADRs/R014-ADR-PureTraitsRule.md) | no |
| `returned-mutations` | [R022](ADRs/R022-ADR-ReturnedMutationsRule.md) | no |
| `immutable-returns` | [R028](ADRs/R028-ADR-ImmutableReturnsRule.md) | no |
| `distinct-tuple-elements` | [R029](ADRs/R029-ADR-DistinctTupleElementsRule.md) | no |
| `leading-constants` | [R031](ADRs/R031-ADR-LeadingConstantsRule.md) | no |
| `shared-constants` | [R032](ADRs/R032-ADR-SharedConstantsRule.md) | no |
| `implemented-in-one-file` | [R026](ADRs/R026-ADR-ImplementedInOneFileRule.md) | no |
| `public-or-private` | [R025](ADRs/R025-ADR-PublicOrPrivateRule.md) | no |
| `ordered-impl-functions` | [R027](ADRs/R027-ADR-OrderedImplFunctionsRule.md) | no |
| `typed-errors` | [R030](ADRs/R030-ADR-TypedErrorsRule.md) | no |
| `test-naming` | [R012](ADRs/R012-ADR-TestNamingRule.md) | no |
| `tested-public-api` | [R013](ADRs/R013-ADR-TestedPublicApiRule.md) | no |
| `header` | [R001](ADRs/R001-ADR-HeaderRule.md) | `--header-file` |

`--rule <NAME>` applies only the named rules; `--skip <NAME>` subtracts. Both
repeatable, both default to everything, and skipping wins over selecting. Every
report names the rules it applied; a run that did not apply all of them says
`All applied rules are satisfied` and names each absence with its reason --
`(skipped)` or what the rule was waiting for, such as `(needs --header-file)`. The JSON carries `rules_applied`,
`rules_skipped` and `rules_unconfigured`. An unknown rule
name is an error, as is `--rule header` without `--header-file`. See
[ADR-RuleSelection](ADRs/ADR-RuleSelection.md).

A rule with nothing to work from is left out of the registry rather than
registered and silently passing -- and is then named in the report as not
applied, so a clean run cannot be mistaken for "the header rule passed" when it
never ran.

## `readable-source`

Every `.rs` file can be read and parsed.

This one exists because silence is indistinguishable from success. The other
parsing rules give up quietly on source they cannot read, trusting `rustc` to
say so more clearly — right for a file somebody is editing, wrong for a file
nobody is looking at. A corrupted file produces no rows, and a file with no rows
looks exactly like a clean file.

| offence | correction |
|---|---|
| file could not be read | check that the file exists and that its permissions allow reading it |
| file does not parse as Rust | correct the syntax error rustc reports, or restore the file if it is corrupted |

**Does not catch:** anything about validity beyond parsing. A file that parses
but does not compile — unknown type, borrow error, missing import — is this
rule's idea of fine, and rightly so.

## `header`

Every `.rs` file opens with the repository's header, supplied by
`--header-file`.

The expected text is data because it is never the same twice: MIT here,
Apache 2.0 in a sibling repository, a different year again next year. The
comparison is exact after normalisation — a BOM, CRLF line endings and a
trailing newline in the header file are all absorbed, so a wrong year or a
swapped licence line still fails while a Windows checkout does not.

Exactly one offence per file: the first divergence. A file with no header at all
would otherwise emit one row per header line and bury the workspace behind it.
The offence carries the **whole** expected header in `expected`, so the fix is
one pass rather than a loop.

| offence | correction |
|---|---|
| file is empty, so it carries no header | make the first N lines of the file match the expected header |
| expected `X` but found `Y` | *(same)* |
| file has N lines but the header is M | *(same)* |

**Does not catch:** it compares text and nothing else. A well-formed header
naming the wrong copyright holder passes. An SPDX identifier disagreeing with
`Cargo.toml` used to pass too; `spdx-matches-manifest` now holds those two
together, and does so without needing `--header-file`.

## `test-file-structure`

A test file reads top to bottom in one order: header, imports, constants,
helpers, tests. Each group alphabetical, case-insensitively. Imports and
constants run together; helpers and tests are separated by exactly one blank
line.

The split is between lists and bodies rather than between imports and the rest.
Imports and constants are both runs of one-line declarations, read by scanning
down a column: a blank line between each entry doubles the height of the list
and tells the reader nothing about it. Helpers and tests are multi-line bodies,
where the gap is the only thing marking where one ends and the next begins.

The count is exact in both directions, so two blank lines between constants is
an offence for the same reason zero between tests is.

**The packing is applied per section, not per constant**, and the "runs of
one-line declarations" argument is not true of every constant. A constant whose
value spans lines is packed against its neighbour like any other, so two
multi-line fixtures meet with no boundary between them; and a constant
introduced by a comment has that comment folded into it, so the comment runs on
from the constant above and reads as a trailing note on it. Constants whose
values are continued across indented lines are unaffected in practice. See
[R002](ADRs/R002-ADR-TestFileStructureRule.md) for the cost and the refinement
that would separate the cases.

`Helpers` is defined by **exclusion** — whatever is neither an import, nor a
constant, nor a test. That is what keeps the set of item kinds closed: a
`struct`, an `impl`, a type alias and a plain `fn` are all helpers, so a kind
nobody has thought of yet lands where a reader would put it.

Applies to `tests/` only, and skips `all_tests.rs` and `mod.rs` — those are
registries, and demanding a blank line between each `pub mod` would make the one
file whose whole job is to be scannable the hardest to scan. Their shape is
`tests-layout`'s business.

**Imports whose order rustfmt decides are left alone.** rustfmt sorts `self`,
`super` and `crate` ahead of every other path, and treats case as significant in
*opposite* directions at the two levels: an uppercase-initial crate sorts behind
every lowercase one (`Bbb::gamma` after `zzz::last`), while an uppercase-initial
segment later in a path sorts ahead of its lowercase siblings
(`serde_json::Value` before `serde_json::from_str`). None of that matches the
alphabet. Demanding the alphabet there would make the file unsatisfiable rather
than merely wrong, since `cargo fmt` runs first and writes the other order back.

So the check stands down, and the decision is **per pair** rather than per
import: where the two paths first differ, if the segments there are of different
case, rustfmt decides. Keying it on an import's first segment alone was a bug --
`use serde_json::Value;` beside `use serde_json::from_str;` share theirs and part
company at the second, which left a file no edit could make green. Everything
else is still ordered.

Two shapes trigger it: a shared helper inside the tests tree, reached as
`use crate::support::builders::a_widget;`, and a same-crate pair diverging by
case.

| offence | correction |
|---|---|
| a `constant` follows a `helper` | move \`X\` up above the helpers |
| `X` is out of alphabetic order | move \`X\` above \`Y\` |
| expected N blank line(s) before `X` | leave exactly N blank line(s) between \`Y\` and \`X\` |

**Does not catch:** it judges shape, not content. The AAA convention —
`// Arrange`, `// Act`, `// Assert` inside a body, and the
`<method>_<description>_<outcome>` naming pattern — is not checked. A file that
does not parse reports nothing here; `readable-source` reports it instead.

## `directory-file-count`

A directory holds at most **20** `.rs` files, not counting its own index.
`max-files-per-directory` in `stern4rust.toml` changes the limit.

This is the only rule whose number is taste rather than fact, which is why it is
configuration. It is also the one most in tension with the rest: one struct per
file, one implemented type per file and one test file per source file all
manufacture files by design, so the limit has to be generous enough that the
conventions producing the files are not themselves the offence.

Registries do not count -- a `mod.rs`, `lib.rs` or `all_tests.rs` is an index
*of* the directory rather than something *in* it. `main.rs` does count: it is an
entry point holding real code.

| offence | correction |
|---|---|
| `src` holds 42 files, more than the 20 a directory may hold | group the files of `src` into subfolders of at most 20 each, each with its own `mod.rs` declared by this index |

Reported against the directory's index, because that is where the `pub mod`
lines for the new subfolders have to go.

**Does not catch:** size. Twenty files of two thousand lines each satisfy it --
that is `crap4rust`'s question. Non-`.rs` files are invisible. And it does not
say *where* to split: a cap forces a division and is silent about which one.

**It cannot be autofixed.** The correction is a `git mv`, a new `mod.rs`, a
declaration in the parent, and a matching move under `tests/`.

## `directory-subfolder-count`

A directory holds at most **5** subfolders containing source, checked at every
level so that pushing sprawl one directory down does not escape it.
`max-subfolders-per-directory` changes the limit.

The counterweight to `directory-file-count`: that rule creates folders, and
without this one the cheapest way to satisfy it is a folder per file.

| offence | correction |
|---|---|
| `src` holds 7 subfolders, more than the 5 a directory may hold | group the subfolders of `src` so that no directory holds more than 5 |

**It finds nothing today.** Across eight repositories the deepest tree is two
levels and no directory has more than one subfolder. It is a guard against a
shape the family has not reached, kept because `PackageTree` already models what
it needs and because the folders it counts are about to be created.

**Does not catch:** what is in the folders -- five subfolders of two hundred
files each satisfy it, and `directory-file-count` is what catches that. The two
are only meaningful together.

## `imported-paths`

Anything this file names -- a function it calls, a type it mentions, a trait it
implements -- is reached through a name it imported, not through a path.

A file's `use` statements are its list of dependencies. `syn::parse_file(...)`
compiles with nothing in the file mentioning `syn`, so a reader scanning the top
to find out what this file needs is quietly given a wrong answer.
`std::env::args()` is a different cost: it spells out at the call site a route
that belongs at the top, and spells it out again at every other call.

Type position costs the same and was added in `0.11.0`. `-> std::io::Result<()>`
and `impl std::fmt::Display for Widget` leave the import list just as wrong --
in the second case, nothing at the top says this file implements `Display`.

Three shapes are left alone. An **unqualified** call has nothing to import. A
**type qualifier** -- `Widget::new()`, `Self::inner()` -- is not a path standing
in for an import, since the type itself was imported and the qualifier says which
type is being constructed. And **one imported segment** -- `use std::fs;` with
`fs::read_to_string(...)` -- is the point of the rule rather than an exception to
it: it names the route once and still says at the call site which module the
function came from. A bare `read_to_string(...)` would satisfy a stricter rule
while saying strictly less.

Module and type are told apart by **case**, a convention rather than a
resolution, because this tool has no type information. The primitives are the
exception, and they are named rather than guessed: `u8` through `u128`, the
signed and pointer-sized forms, `f32`, `f64`, `bool`, `char` and `str` are
types whatever their case says. Reading `u64` as a module made
`u64::from_le_bytes` an offence whose correction was `use u64::from_le_bytes;`
-- which does not compile, since a primitive's inherent function cannot be
imported at all. Matched whole, so a module named `u64_helpers` is still a
module.

| offence | correction |
|---|---|
| `syn::parse_file` is reached through a path | add `use syn::parse_file;` and call `parse_file` |
| `std::env::args` is reached through a path | add `use std::env;` and call `env::args` |

The two shapes split differently on purpose. A two-segment path imports whole,
because `use syn;` would be legal and leave the call site unchanged. A longer one
imports all but the last segment, keeping `env` because `env::args()` reads
better than a bare `args()`.

Applies to **both** productive and test files -- the only rule so far with no
`tests/` exemption, because a test file has the same reader and the same list of
dependencies at its top.

**Does not catch:** paths outside call position. A `let x: std::path::PathBuf` or
a `std::fmt::Result` return type passes, since the standard is about function and
method qualifiers. Macros are not checked -- `serde_json::json!(...)` is an
`ExprMacro`, not a call. And a lowercase-named type or an uppercase-named module
is judged by its case rather than by what it is.

## `test-free-source`

Tests live in `tests/`, and the production source tree carries none of them.

A `#[cfg(test)] mod tests` inside `src/` is invisible to everything else: it is
not the mirrored test file `twin4rust` looks for, it is not declared from
`all_tests.rs`, it has no required shape, and it is compiled under a
configuration the shipped build never uses — so it can drift out of step with
the code it tests and no build notices.

Three shapes, all outside `tests/`:

- a function carrying a test attribute, matched on the **last path segment**, so
  `#[tokio::test]` counts without enumerating harnesses
- `#[cfg(...)]` whose predicate mentions `test`
- `#[cfg_attr(...)]` whose predicate mentions `test`

Both `cfg` forms are recognised through the *predicate*, so `any(test, ...)` and
`not(test)` are caught. The predicate is scanned for an **identifier**, not a
substring, so `#[cfg(feature = "test")]` is a feature named test and not a gate.

The walk descends into inline modules. An item that is itself an offence is not
descended into — a `#[cfg(test)]` module is one decision, not one per test
inside it.

**The line is `test`, not conditional compilation.**
`#[cfg(feature = "...")]` and `#[cfg_attr(feature = "serde", derive(Serialize))]`
are ordinary library work and are left alone: a feature is selectable by the
shipped build, so what is tested is what somebody runs. `test` is the one
predicate no shipped build ever sets.

| offence | correction |
|---|---|
| the `#[cfg(test)]` module `X` | move the tests to `tests/<mirror>_tests.rs` and delete this from the source tree |
| the test function `X` | *(same)* |
| the `#[cfg_attr(test, ...)]` on the struct `X` | apply the attribute unconditionally, or move what it guards into `tests/<mirror>_tests.rs` |

The correction names the **mirrored file** — `src/<path>.rs` maps to
`tests/<path>_tests.rs`, the same pairing `twin4rust` enforces.

**Does not catch:** a test-only helper carrying no test attribute and no gate —
an ordinary `pub fn make_test_widget()` — is invisible, because nothing in the
source distinguishes it from production code.

## `registry-completeness`

A registry declares every module beside it: each sibling `.rs` file, and each
subfolder that has a registry of its own. Nothing in the tree goes uncompiled.

**Only one direction is checked, and it was measured.** `pub mod missing;` with
no `missing.rs` is a compile error -- `rustc` reports `E0583` immediately and
more clearly than this could. An orphan `.rs` file that no registry declares
produces **no error and no warning at all**. Silence is the whole failure, so
silence is all this looks for.

`pub` is not required here: a private `mod name;` compiles the file just as
well, and being compiled is the concern. `module-registry` is the rule that
wants `pub`. An inline `mod name { ... }` declares no file and does not count.
`main.rs` counts as a registry beside `lib.rs`, so a file declared only from the
entry point is not reported.

| offence | correction |
|---|---|
| `beta_tests` is not declared here, so its file is never compiled | add `pub mod beta_tests;` to `tests/all_tests.rs` |

Reported against the **registry**, not the orphan -- the orphan is a perfectly
good file and the edit that fixes it is one line somewhere else. An unparseable
registry silences the rule for that directory rather than reporting every file
beside it; `readable-source` reports the registry itself.

**Does not catch:** a file declared through `#[path = "..."]`, which reads as
undeclared -- which is why `declared-by-name` forbids that attribute outright, so
this can only be reached by a repository that skipped it. A file the walker never
reached, including anything under
`--exclude`. And `#[cfg(...)]`-gated declarations count as declarations, which
is right for "is it ever compiled" and wrong for "is it compiled in this
configuration".

## `module-registry`

A `lib.rs` or `mod.rs` outside `tests/` is an index of the modules beneath it,
and holds nothing else: the header, the crate's inner attributes,
`extern crate alloc;`, and `pub mod` declarations.

Inner attributes need no exception -- `syn` keeps `#![no_std]` on the file
rather than among its items, so a no_std crate root passes without the rule
knowing which attributes exist. `extern crate alloc;` is the one non-`mod` item
allowed: a no_std crate has to say it somewhere and the crate root is where it
belongs. `pub` is required, because a private `mod` hides part of the crate's
shape from the file whose job is to state it.

The sharpest thing it catches is the **re-export shim** -- `pub use` in a
registry -- which these standards forbid outright and which forms in exactly
this file.

| offence | correction |
|---|---|
| the import `use std::ffi::OsString;` | move the import into a module of its own |
| the function `run` | *(same)* |
| the module `hidden` (private) | *(same)* |
| the inline module `alpha` | *(same)* |

`tests/` is left to `tests-layout`, which asks a different question of the same
filenames and gives a different answer about a private `mod`.

**Does not catch:** whether the declarations are *complete* or *ordered* -- a
`lib.rs` omitting a module that exists on disk passes, the same gap
`tests-layout` has. It says nothing about `main.rs`, which is an entry point
rather than an index and legitimately holds code.

## `single-implemented-type`

A source file outside `tests/` holds at most one type that carries behaviour: at
most one `struct` or `enum` that is both **declared in the file** and **has
behaviour** in it -- an object, as `records-and-objects` defines one. Records
and data enums are unlimited -- plain data is not a subject, and a file's payload
types belong beside the subject that uses them, where `record-home` puts them.

Both halves do work. *Declared here*, so an `impl Display for SomeoneElsesType`
does not make this file that type's home. *Has behaviour*: an inherent method
other than a getter, a plain constructor or a `with_` step, or an impl of a trait
outside the value traits -- `Display`, `From`, `Default` and the rest of that
fixed list are how a record prints and converts, and leave it a record. Inherent
and trait impls are behaviour alike; `#[derive(...)]` is not an impl block in the
syntax tree and correctly does not count. (Amended 2026-10-25,
[R007](ADRs/R007-ADR-SingleImplementedTypeRule.md).)

The subject is the implemented type the file is named after, wherever it stands
in the file -- `WireClientRequest` in `wire_client_request.rs` -- and every other
one is reported, so the offence names the type to move. Named after is read as
`named-subject` reads it, so `UdpListener` is the subject of `listener.rs`. A
file with no type named for it takes the first as its subject. The walk descends into inline modules, since
wrapping a second subject in `mod detail { ... }` changes nothing a reader cares
about.

| offence | correction |
|---|---|
| `ColumnWidths` is a second type with an impl block; this file's subject is already `ReportPrinter` | move `ColumnWidths` and its impl blocks into `column_widths.rs` |

`tests/` is exempt: a test file legitimately holds several fakes that each carry
an impl block, which is the shape `test-file-structure` asks for.

**Does not catch:** size. One type with forty methods satisfies this completely
-- that is `crap4rust`'s question. It says nothing about free functions, and does
not treat a `trait` with default method bodies as a subject -- `pure-traits`
covers the other half of that gap. Nor does it ask whether the file is named
after its subject; `named-subject` does.

## `named-subject`

A source file is named after its subject.

The candidates are the types the file implements, as `single-implemented-type`
counts them: a `struct` or `enum` declared in the file with behaviour there -- an
object. A record or data enum with value-trait impls is a declared type, not an
implemented one (amended 2026-10-25, [R023](ADRs/R023-ADR-NamedSubjectRule.md)). A file with none is named after one of the traits and types it
declares instead, traits first, since a trait declares behaviour and a data type
does not. A trait beside an implemented type is that type's companion and not a
candidate.

**Named after** means the file's stem is the type's name, underscores and case
aside, or its last whole words. The words in front are usually the crate's:
`loopback/src/driver.rs` holding `LoopbackDriver` reads right where it stands,
and so does `listener.rs` holding `UdpEiListener`. A type's name splits at its
capitals, a run of capitals being one word, so `http_server.rs` is named after
`HTTPServer`. Only the front may be left out: `protocol_observer.rs` holding
`Observer` claims more than the type says, and `dex.rs` is not named after
`Index`.

One offence per file, against the first candidate's declaration, listing every
candidate:

| offence | correction |
|---|---|
| `` `listener.rs` is named after none of the types it implements: `UdpListenerTask` `` | rename the file to `udp_listener_task.rs`, or rename `UdpListenerTask` after the file |
| `` `cluster_query_types.rs` is named after none of the traits and types it declares: `QueryRequest`, `QueryResponse` `` | rename the file to `query_request.rs`, or rename `QueryRequest` after the file |

A file with two implemented types, one of them named for it, is
`single-implemented-type`'s, which reports the other, and this rule is silent.

Exempt: `lib.rs`, `main.rs`, `mod.rs` and `build.rs`, named for their role in the
package, and `tests/`, `benches/`, `examples/` and `src/bin/`, named for what they
exercise or run. A file of functions and constants alone has nothing to be named
after.

**Does not catch:** whether a name is a good one -- `data.rs` holding `MetaData`
is named after it. Only the stem is read, so `udp/listener.rs` holding
`TcpListener` passes. A trait beside an implemented type is not checked, and a
type declared by a macro is invisible.

## `owned-functions`

A file with a subject owns its functions: a source file that declares a struct,
an enum or a trait, or holds an impl block, holds no free function.

A free function is any `fn` outside an impl block or a trait -- at module level,
inside an inline module, or nested in another function's body, so that hiding a
helper inside a method is no way out. The subject is the one
`single-implemented-type` and `named-subject` read: the implemented type named
for the file, else the first; else the type an impl block is for; else a
declared type, then a trait.

| offence | correction |
|---|---|
| `` `border` is a free function in a file whose subject is `Widget` `` | move `border` into `impl Widget` as an associated function |
| `` `checksum` is a free function in a file whose subject is `Widget` `` | move `checksum` into a file of its own, or into `impl Widget` if it belongs there |
| `` `init` is a free function in a file whose subject is `UdpStackFactory` `` | move `init` into a file of its own |

The correction reads the file. A function the subject's impl calls, or whose body
names the subject, is offered the impl; one that touches neither is offered a
file of its own first. Beside a trait only a file of its own is offered, since
holding the function would take a default body and `pure-traits` forbids one.

Left out: a function something other than the author requires to be free -- an
attribute whose last segment is `alloc_error_handler`, `entry`, `exception`,
`export_name`, `interrupt`, `main`, `no_mangle`, `panic_handler`, `pre_init`,
`proc_macro`, `proc_macro_attribute`, `proc_macro_derive` or `task`, also inside
`#[unsafe(...)]`, or an `extern "..."` function -- and `main` in an entry file
(`main.rs`, `src/bin/`, `examples/`, `benches/`). `#[cfg(test)]` items are
`test-free-source`'s. `tests/`, `lib.rs`, `mod.rs` and `build.rs` are exempt.

A file of functions alone has no subject and is left alone: moving a helper into
one separates it from the type, which is what the rule asks.

**Does not catch:** whether the fix is a good one -- a private associated
function satisfies it exactly as an extracted struct does. Functions generated by
a macro are invisible, and so are closures stored in statics.

## `pure-traits`

A trait declares; it does not implement. **No method in a `trait` declaration in
`src/` may have a default body.**

A default reads as a convenience and works as a decision nobody made. The
implementor that says nothing about a method is indistinguishable from the one
that considered it and found the default right, and the question of which you are
looking at cannot be answered by reading either file. Make the body a declaration
and every implementor has to answer, in its own file, where the answer is.

The second half of the requirement -- that every implementor implements every
method -- **needs no rule**. With no body to fall back on, `rustc` rejects an
incomplete impl with `E0046`, immediately and more precisely than this tool
could. Only the half the compiler is silent about is checked here, which is the
same split [R009](ADRs/R009-ADR-RegistryCompletenessRule.md) made.

Only methods are reported. An associated type and an associated constant may
carry a default: neither is behaviour, so neither lets an implementor inherit a
decision while appearing to have made one.

| offence | correction |
|---|---|
| `` `Collection::is_empty` has a default body, so an implementor that says nothing about it cannot be told from one that chose it `` | move the body into each implementor |

The offence names the method rather than the trait, because a trait with three
defaults is three separate edits landing in three different sets of files.

`tests/` is exempt: a test file declares traits to stand in for real ones, and a
stand-in with a body is the shape those fakes are supposed to have.

**Does not catch:** a **blanket impl**, which puts one body behind every
implementor at once and is an `Item::Impl` rather than an `Item::Trait` -- a
trait emptied of defaults can have all of them restored this way and the rule
will say nothing. Nor a default inherited from a supertrait in another crate, nor
whether a removed body was moved into the implementors or simply deleted --
`rustc` guarantees each implementor has *a* body, not the right one. Anything
inside a macro is invisible, as everywhere else in this tool.

## `returned-mutations`

A function returns its output; it does not write it through a `&mut` parameter.
**No parameter of a function in `src/` may have a type that is, or structurally
contains, a mutable reference.**

A `&mut T` argument is an output dressed as an input: the caller hands in a place
for the function to write, and the call's real result is the edit it leaves
behind rather than the value it returns. The reader of the signature cannot tell
what comes back without reading the body, and the caller has to build the
destination before there is anything to put in it. Returning the value makes the
output something the signature names and the caller owns.

The type is walked through references, tuples, slices, arrays and
angle-bracketed generic arguments, so `(&mut A, B)` and `Vec<&mut T>` are both
reported. Every offending parameter is reported separately, because each is its
own edit.

Four things are left alone, each because the edit does not live there:

- **The `&mut self` receiver.** It carries no parameter type, so it falls out of
  the check by shape. A method mutating the thing it belongs to is the point of a
  method.
- **Trait impl methods.** `impl Display for X` takes `&mut Formatter` because
  `Display` says so, and no implementor can change that. Where the trait is the
  author's own, the `&mut` is reported once, at the declaration -- the one place
  changing it is both possible and enough. The same split as `pure-traits` and
  `registry-completeness`.
- **Callback contracts.** `impl FnMut(&mut T)` and `fn(&mut T)` describe the
  callback's parameter list, not this function's, so the walk stops at a
  parenthesised `Fn`-trait argument and at a bare function pointer.
- **A plain constructor storing the borrow.** `new(protocol: &'a mut T) -> Self`
  whose whole body is `Self { protocol }` writes nothing through its argument; it
  keeps it, and the writes come later through `&mut self`. A component that
  borrows its owner for one call is built this way. A constructor that does
  anything else with its parameters is judged like any other function.
  ([R022](ADRs/R022-ADR-ReturnedMutationsRule.md), amended 2026-10-25)

| offence | correction |
|---|---|
| `` `render::out` is a `&mut` parameter, so its result is written through an argument instead of being returned `` | `` return the new value instead of writing it through `&mut out` `` |

`tests/` is exempt: a test file hands a fake a `&mut` to stand in for a real
collaborator, and a stand-in shaped that way is what those fakes are for.

**Does not catch:** interior mutability -- `&Cell<T>`, `&RefCell<T>`,
`&Mutex<T>` mutate through a shared reference the rule never sees; raw `*mut T`
pointers; a `&mut` inside an `impl Trait` or `dyn` bound such as
`impl Iterator<Item = &mut T>`; a `&mut` reaching a function through a generic,
as `fn f<W: Write>(w: W)` called with `&mut buf`; and anything a macro
generates. Only the signature is read, so a `&mut` parameter the body never
writes is still reported -- the signature still says the call writes through it.

## `immutable-returns`

A function does not hand out a mutable reference. **No function in `src/` may
have a return type that is, or structurally contains, a mutable reference.**

`&mut self` is the one channel through which the house standard lets a type
change. A function returning `&mut T` opens a second one, owned by whoever holds
the reference: the caller changes the type's internals without going through any
of its methods, and every invariant the type keeps holds only while nobody uses
what it lent. `returned-mutations` stops a mutation coming in through a
parameter; this rule stops one going out through a return value.

The return type is walked as `returned-mutations` walks a parameter's -- through
references, tuples, slices, arrays and angle-bracketed generic arguments -- by the
same `MutableReferenceSearch`, so `Option<&mut T>` and `(usize, &mut [T])` are
both reported. One offence per function.

Three things are left alone:

- **`&'static mut`.** A static handed over once -- the `StaticCell` shape an
  embedded executor needs for its stack resources -- has no type behind it to
  lend. The search passes over it but still looks inside it, so a borrow it
  points at is found.
- **Trait impl methods.** `DerefMut`, `IndexMut`, `AsMut` return `&mut` because
  the trait says so. Where the trait is the author's own, it is reported once, at
  the declaration.
- **Callback contracts.** A returned `impl FnMut(&mut T)` says what the caller's
  closure receives, not what this function lends.

| offence | correction |
|---|---|
| `` `total_mut` returns a mutable reference, so its caller can change what it lends without going through its methods `` | `` return the value, or give the type a method that makes the change `` |
| the same, for `-> &mut Self` | `` take and return `self` by value instead of `&mut Self` `` |

`tests/` is exempt.

**Does not catch:** interior mutability handed out -- `&RefCell<T>`, `&Cell<T>`,
an `Rc<RefCell<T>>` -- which mutates through a shared reference; raw `*mut T`
pointers; a `&mut` inside an `impl Trait` bound such as
`impl Iterator<Item = &mut T>`; and anything a macro generates.

## `distinct-tuple-elements`

A tuple in a return type repeats no element type. **No function in `src/` may
return a type holding a tuple with two elements of the same type.**

`-> (u64, u64)` returns two numbers and says nothing about which is which, and a
caller that swaps them still compiles. Most tuples are not like that: in
`(PathBuf, u64)` the types say which element is which and a swap does not
compile. The rule asks only about the narrow case, two elements of one type,
where position is the only thing telling them apart.

The return type is walked as the other signature rules walk it -- through
references, tuples, slices, arrays and angle-bracketed generic arguments -- so
`Option<(Instant, Instant)>` and `Vec<(usize, usize)>` are reported, private
functions as much as public ones, since a private function's callers can swap
its results as easily. The outer tuple is found first; one offence per function.

Types are compared as written, with lifetimes and needless whitespace dropped and
nothing resolved: `&'a str` and `&'static str` are one type, and `(Height, u64)`
passes even where `Height` aliases `u64`. An alias that names what a value means
is the cure for the ambiguity, not a way around it.

Left alone: tuples of distinct types; a returned callback's parameter list,
`impl Fn(u8, u8) -> u8`, which is its contract; and trait impl methods, whose
signature is the trait's -- a trait of the author's own is reported at its
declaration.

| offence | correction |
|---|---|
| `` `bounds` returns `(u64, u64)`, whose `u64`s only their position tells apart `` | `` return a struct whose fields say which `u64` is which `` |

`tests/` is exempt.

**Does not catch:** a tuple inside an `impl Trait` or `dyn` bound, such as
`impl Iterator<Item = (u64, u64)>`; parameter lists with two arguments of one
type, which have the same problem on the way in; two types that convert into each
other silently; and anything a macro generates.

## `leading-constants`

A source file reads header, imports, constants, then everything else. **No
module-level `const` or `static` in `src/` may be declared after an item that is
neither an import nor a constant.**

A test file already reads in one order; a source file had none for its
constants, and the common shape parked one beside its first reader, so a file's
numbers could not be found without reading all of it. Module-level items are
order-independent in Rust -- a constant can be read before its declaration -- so
where one sits is pure layout, and a placement a tool can enforce and repair.

The first item that is neither an import nor a constant leads the rest of the
file: a struct, enum, trait, impl, function, type alias, union, macro or inline
module. Imports, `extern crate` and `mod` declarations are not judged here --
their order is `ordered-imports`' and rustfmt's -- and an inline module is one
item, whose contents are not looked at. The constants group is every constant
before the leader, in the order written; sorting it is not asked. One offence
per file, at the first constant out of place.

| offence | correction |
|---|---|
| `` `LIMIT` is declared after `Widget`; constants come straight after the imports `` | `` move it, and every constant below `Widget`, straight after the imports, or run `--fix` `` |

`--fix` moves every misplaced constant -- whole line ranges from `syn` spans,
with its attributes and the comment directly above it -- to the end of the
constants group, or straight after the imports when there is none; every other
line keeps its place and its bytes, and the blank lines around the group and
where a constant left become one. It declines three kinds of file, and their
offence says so, ending *"by hand: `--fix` leaves alone"* and the reason:

- a file with a `macro_rules!`, whose textual order decides what a later item
  can call;
- a constant, or the leader, sharing a line with another item, which cannot be
  moved as lines;
- a file whose opening comment runs into its leader with nothing above it, where
  the header cannot be told from the leader's own comment.

Registries (`lib.rs`, `mod.rs`) and `tests/` are exempt.

**Does not catch:** associated constants, which belong to their impl; constants
inside an inline module or a function body; and anything a macro generates.

## `shared-constants`

A module-level constant is private, or public and shared from a constants file.
**A module-level `const` in `src/` is private, or `pub` in a constants file and
read by two or more other files of the workspace.**

A constant has two honest states, as a method does under `public-or-private`.
It is one file's, private and read where it is declared; or it is a fact
several files share, and then its home is none of its readers but a file of
constants a reader finds by name.

- **A constants file** holds imports, constants, statics and type aliases, and
  nothing else -- no struct, enum, trait, impl, function or macro. Its name is
  not prescribed; the correction suggests `<topic>_constants.rs`.
- **Read by** is a name match across every member of the workspace, `tests/`
  included: a path, an import, a pattern, a macro's tokens. A declaration is not
  a read. A run scoped to one member still counts the others' readers.
- **Restricted visibility** -- `pub(crate)`, `pub(super)`, `pub(in ...)` -- is an
  offence on its own.
- **Statics** are runtime state, living with their owner, and **associated
  constants** are their type's: both are left alone. An inline module is not
  descended into.
- **A library whose constants other repositories read** cannot be judged from
  inside one; it skips the rule in `stern4rust.toml`: `[package.<name>] skip =
  ["shared-constants"]`.

| offence | correction |
|---|---|
| `` `LIMIT` is public, and no other file reads it `` | `make it private, or delete it if nothing reads it` |
| `` `LIMIT` is public, and only `node/src/pool.rs` reads it `` | `move it into node/src/pool.rs and make it private` |
| `` `LIMIT` is read here and by `node/tests/pool_tests.rs`, from a file that is not a constants file `` | `move it into a constants file ..., or make it private if node/tests/pool_tests.rs can do without it` |
| `` `LIMIT` is read by 2 other files, from a file that is not a constants file `` | `move it into a constants file -- imports, constants, statics and type aliases only --, such as <topic>_constants.rs` |
| `` `LIMIT` is `pub(crate)`; a constant is private, or public and shared `` | `make it private, or pub and shared from a constants file` |

Registries (`lib.rs`, `mod.rs`) and `tests/` are exempt.

**Does not catch:** same-named constants in different modules, which a name
match conflates -- a reader of one counts for the other; a constant read through
a glob import alone, or built by a macro; statics and associated constants, by
decision.

## `implemented-in-one-file`

A type's inherent impl lives in one file. **No type may have inherent `impl`
blocks in two or more files of its package.**

`single-implemented-type` keeps a file to one implemented type and
`named-subject` names the file after it; nothing kept a type to one file. Rust
accepts an inherent impl in any file of the crate, so a type that grew could be
continued file after file, each passing both rules and the whole a subject no
file names -- and, privacy being per module, everything those files share had to
be `pub(crate)` or `pub(super)`.

The whole package is read at once. A type is the last segment of the path an
impl is for, nothing resolved: `impl<T> Store<T>`, `impl Store<u8>` and
`impl crate::store::Store` are all `Store`. Trait impls are not counted, and
impls inside inline modules are. One offence per type, at its declaration -- or,
with none in the package, at its first impl -- naming every file.

A name declared in two or more files is read as alternatives -- platform types a
`#[cfg]` chooses between, or unrelated types sharing a name -- each implemented
beside its own declaration, and reported only when an impl sits in a file
declaring none of them.

| offence | correction |
|---|---|
| `` `Store` is implemented in 2 files: `src/store.rs`, `src/store_queries.rs` `` | `` move every `impl Store` into `src/store.rs`, or split `Store` into types of their own `` |
| `` `Flash` is declared in 2 files and implemented in 1 that declares neither: `src/flash_ops.rs` `` | `` move each `impl Flash` beside the declaration it is for, or give the types that share the name names of their own `` |

Where two or more of the implementing files are ones a `#[cfg]` selects -- on the
file's `mod` declaration or one above it, as `#![cfg(...)]` at its top, or on the
impl itself -- the correction is *"put the contract in a trait in `traits/` and
give each platform its own implementing type"*. With no declaration in the
package, it is to move the impls into one file.

`tests/` is exempt.

**Does not catch:** a type implemented in one file and declared in another,
which is allowed; a type reached through an alias, which is a second name; `mod`
declarations inside an inline module, whose gates are not followed; and impls a
macro generates.

## `public-or-private`

A function in an inherent impl is `pub` or private. **No method of an inherent
impl in `src/` may be `pub(crate)`, `pub(super)`, `pub(self)` or `pub(in ...)`.**

`fn` is a type's implementation and `pub fn` its contract, which
`tested-public-api` holds to a test. The restricted forms are a third state that
falls between the gates: not private to the file, not an entry point
`tested-public-api` counts, and out of reach of a test in `tests/`, which compiles
as another crate. They usually exist because something outside the type's file
needs the method -- most often a type whose impl is spread over files.

Inherent impls only, inline modules included: a trait impl's methods take the
trait's visibility, and free functions, fields, types and modules are not this
rule's. One offence per method, at its name. The correction reads the package's
call sites, matched on name and arity -- the receiver not counted -- as
`tested-public-api` matches them:

| when | correction |
|---|---|
| no other file calls it, or it is `pub(self)` | `` make it private: `fn` `` |
| another file calls it | `` make it `pub` and give it a test, or move the caller's need behind the type's public contract `` |
| its type's impl is spread over files | `` implement `RaftProtocol` in one file, as `implemented-in-one-file` asks: the restriction serves its spread over 13 files `` |

| offence |
|---|
| `` `election` is `pub(crate)` in `impl RaftProtocol`; a method is `pub` or private `` |

`tests/` is exempt.

**Does not catch:** restricted visibility anywhere else -- free functions, fields,
types, constants, modules; a `pub fn` that should have been private, which
`tested-public-api` and review judge; and methods a macro generates.

## `ordered-impl-functions`

An inherent impl block lists its functions in a fixed order. **Constructors
first, fewest parameters first; then every `pub fn`; then every other function
-- each group alphabetical, constants and types ahead of them all.**

- **A constructor** is a function with no receiver whose return type names
  `Self` or the impl's type, directly or wrapped: `Self`, `Result<Self, E>`,
  `Option<Store>`, `(Self, Vec<Outcome>)`. Its visibility does not matter.
  Constructors with as many parameters as each other are alphabetical.
- **A `pub fn`** is any function written `pub`, `const`, `async` and `unsafe`
  alike. Everything else is the rest: private functions, and restricted ones
  where `public-or-private` is not adopted.
- **Alphabetical** is byte by byte, a raw identifier without its `r#`. Two
  functions of one name -- `cfg`-gated twins -- stay adjacent, as written.
- **Associated constants and types** come before every function, in the order
  written.

Where a function sat was decided by whoever wrote it; in a repository written
mostly by agents the author changes every session, so narrative order drifts.
A fixed order is the same in every session, takes the decision out of every
change, and keeps a new function's diff to the lines it adds.

| offence | correction |
|---|---|
| `` `impl Bookkeepers` is out of order: `catch_up` belongs before `cluster` `` | `` list constructors first, fewest parameters first, then its `pub` functions, then the rest, each alphabetical, its constants and types ahead -- or run `--fix` `` |

One offence per block, at the impl: a single insertion can displace every
function after it. Trait impls keep the trait's order and are left alone, and
`tests/` is exempt.

`--fix` reorders every block out of order, moving whole members -- each with its
attributes, doc comments and the comment directly above it -- and changing only
the lines between the block's braces: members one blank line apart, constants
and types none, a comment before the first member kept first and one after the
last kept last. It declines a block, which keeps its offence, ending *"by hand:
`--fix` leaves alone"* and the reason, when it holds a macro invocation among
its items, two members on one line or on a brace's line, or a comment standing
free between two functions. `rustfmt` does not reorder impl items, so the order
is never undone by the formatter.

**Does not catch:** trait impls; whether a long impl should be split; and
functions a macro generates.

## `typed-errors`

A library's public functions fail with an error type of their own. **No public
function of a library target may return a `Result` whose error is a `String`, a
`&str` or a boxed `dyn ...Error`.**

`Result<T, String>` makes a failure a sentence. A caller can print it and do
nothing else, telling one failure from another means comparing text, and a test
pinning a failure path pins its wording, so a reworded message breaks a test
that was right. An error type of the crate's own -- an enum, a variant per way
to fail -- lets a test match the failure it means and a caller handle one and
pass the rest.

- **Public** is a `pub fn`, a `pub` method of an inherent impl, and every method
  of a `pub` trait, reported at the trait's declaration. Trait impls are the
  trait's.
- **The library target** is `src/lib.rs` and every file its module declarations
  reach, as `name.rs` or `name/mod.rs`. A binary's files -- `src/main.rs`,
  `src/bin/`, and the modules only they declare -- end on a terminal and are
  left alone, and so is a package with no `src/lib.rs`.
- **The error** is found wherever the `Result` sits in the return type -- in an
  `Option`, in a `Vec` -- as a path ending in `Result` with two type arguments.
  An alias with one, `io::Result<T>`, names its error elsewhere.
- **A library only in form** -- a system-test harness, an `xtask` with a
  `lib.rs` -- skips the rule in `stern4rust.toml`: `[package.<name>] skip =
  ["typed-errors"]`.

| offence | correction |
|---|---|
| `` `read` fails with a `String` `` | `give the crate an error enum its callers and tests can match` |

`tests/` is exempt.

**Does not catch:** an error type with one variant holding a `String`, a string
in a costume; `anyhow::Error`, or an alias such as `anyhow::Result<T>` hiding
the error type; `Option` where a failure has more than one cause; and panics,
which `clippy`'s `unwrap_used`, `expect_used` and `panic` lints are for.

## `declared-by-name`

A module is declared by name: `mod alpha;` reaches `alpha.rs` or `alpha/mod.rs`,
and nothing else decides which file that is. `#[path = "..."]` on a `mod` is an
offence, anywhere in the package.

This is not a rule about taste. It is the one attribute that makes another rule
here give a **confident wrong answer**: `registry-completeness` resolves
declarations by convention, so a file reached through an explicit path is
reported as never compiled when it compiles perfectly well. That rule accepted
the gap on the grounds that the house standard forbids `#[path]` -- a convention
nothing enforced until now.

It applies package-wide rather than to registries alone. The standard names
`all_tests.rs` because that is where the temptation is; the harm is the same
wherever the attribute appears.

| offence | correction |
|---|---|
| `` `mod alpha` is reached through `#[path = "elsewhere/other.rs"]`, so the file it declares cannot be found from its name `` | move `elsewhere/other.rs` to `alpha.rs` beside this file and drop the `#[path]` attribute |

`#[cfg_attr(unix, path = "...")]` is deliberately left alone -- a platform-gated
module is the one honest use of the attribute, and reporting it would accuse
correct code.

**Does not catch:** a `cfg_attr`-gated path, by decision, which
`registry-completeness` would still misread on the platform where it applies.
Nor a `#[path]` produced by a macro, nor whether the file it points at exists --
that is `rustc`'s `E0583`.

See [R018](ADRs/R018-ADR-DeclaredByNameRule.md).

## `ordered-imports`

Imports in `src/` run in alphabetic order, on the pairs where the alphabet is
the authority.

`test-file-structure` has asked this of `tests/` since 0.2.0 and nothing asked
it of the source tree -- which is where `imported-paths` routinely *adds* lines,
with nothing saying where a new one lands.

**The stand-downs are the design.** `cargo fmt` runs first in the gate and sorts
`self`, `super`, `crate` and uppercase-initial paths by rules of its own, so
demanding the alphabet there writes a file **no edit can make green**: each run
undoes the last. The rule asks `ImportPath` -- the same seam
`test-file-structure` uses -- rather than deciding again.

**Pairs it does judge are compared on the parsed path, not the line as
written.** A line ends in `;` (0x3B), which loses to any digit or letter, so
`aaa::select` compared as text sorts after `aaa::select4`; a line may open with
`pub `, whose `p` beats the `u` of `use`, so every re-export sorted above every
import. rustfmt compares segments and does neither.

Case is read as a *shape* rather than as a first letter: the initial, and
whether the segment is all capitals. `WAL_V2_MAGIC` and `WalRecord` both open
with a capital and the style editions still disagree about them -- 2021 orders
`WalRecord` first, 2024 `WAL_V2_MAGIC` -- so a screaming-case segment beside a
camel-case one stands down like any other case divergence. Segments sharing a
shape are still judged, because `ALPHA_TWO` against `ZETA_ONE`, `Alpha` against
`Zeta` and `alpha` against `zeta` were each measured to sort identically under
2021, 2024 and a plain sort.

A block ends where lines stop being consecutive, so a blank line or a comment
separates two imports and the first of a block is compared with nothing.

| offence | correction |
|---|---|
| `` `use aaa_crate::Alpha;` is out of alphabetic order; it follows `use zzz_crate::Zed;` `` | move `use aaa_crate::Alpha;` above `use zzz_crate::Zed;` |

**Does not catch:** any pair `cargo fmt` decides -- measured on this crate, that
is **56% of adjacent import pairs in `src/`**, because a source file usually
leads with a `crate::` block while a test file never does. So more than half of
what the rule appears to check, it does not. Nor grouping: whether `std`,
external crates and `crate::` are separated at all, or in what order those
blocks appear. Nor imports inside inline modules or macros.

See [R019](ADRs/R019-ADR-OrderedImportsRule.md).

## `workspace-dependencies`

A workspace declares its dependencies once, in the root, and every member takes
them from there with `.workspace` notation.

**Three requirements, one check.** The root holding every reference, each member
using `.workspace`, and no member declaring its own are the same requirement:
`cargo` refuses to build a `foo = { workspace = true }` the root does not
declare, so only the middle one needs code. The same split
[R009](ADRs/R009-ADR-RegistryCompletenessRule.md),
[R014](ADRs/R014-ADR-PureTraitsRule.md) and
[R016](ADRs/R016-ADR-PairedTestFileRule.md) each found before it.

Read from the TOML rather than from `cargo metadata`, because the question is
*how* a dependency was written and resolution erases exactly that. All three
tables count -- `dependencies`, `dev-dependencies`, `build-dependencies` -- since
a member pinning its own `proptest` splits the workspace as surely as a runtime
dependency does. Intra-workspace path dependencies are included.

| offence | correction |
|---|---|
| `` validation/Cargo.toml declares `node` in [dependencies] rather than taking it from the workspace `` | add `node` to [workspace.dependencies] in the root manifest, and write `node = { workspace = true }` here |

A package that is **not** a workspace has no root to centralise into, so the rule
says nothing -- the silence `tests-layout` keeps about a package with no tests
tree, not the `(not configured)` state.

**Does not catch:** whether the root's `[workspace.dependencies]` is tidy -- an
entry nothing references is invisible. Nor version drift within the root, nor
`[patch]`, `[replace]` or target-specific tables, nor whether a member needs the
dependency at all.

See [R021](ADRs/R021-ADR-WorkspaceDependenciesRule.md).

## `spdx-matches-manifest`

Every file's `SPDX-License-Identifier` says what the manifest's `license` says.

`header` compares a header against a text file and nothing else, so an SPDX line
disagreeing with `Cargo.toml` passes it. This rule takes its expected value from
**the package being judged** rather than from a flag -- which is why it needs no
`--header-file` to hold, and why it can catch a file in a repository that has no
header file at all.

The header is the comment block a file opens with: everything before the first
line that is neither blank nor a `//` comment. An SPDX line below that declares
nothing.

| offence | correction |
|---|---|
| `` src/widget.rs carries no `SPDX-License-Identifier:`, so nothing ties it to the `MIT` the manifest declares `` | add `// SPDX-License-Identifier: MIT` to the header, or correct the manifest |
| `` src/widget.rs declares `Apache-2.0` where the manifest declares `MIT` `` | change the header to `// SPDX-License-Identifier: MIT`, or correct the manifest |

Both corrections end **"or correct the manifest"**: the rule knows the two
disagree, not which is right.

A manifest naming no `license` leaves the rule nothing to work from, so it is
**not applied** rather than offended -- named in the report, as the header rule
is.

**Does not catch:** a licence stated only in prose, which reads as a *missing*
identifier rather than a contradicting one. Nor whether the licence is correct,
nor the copyright holder or year. A workspace whose packages declare different
licences leaves the rule unconfigured.

See [R020](ADRs/R020-ADR-SpdxMatchesManifestRule.md).

## `arrange-act-assert`

A test reads `Arrange`, then one or more `Act`/`Assert` pairs, with a blank line
separating the sections.

Every marker expands into the phases it names, and the expanded sequence must be
`Arrange` followed by one or more `Act`, `Assert` pairs. The merged forms expand
identically to the separate ones, so one check covers every legal shape:

| written | expands to |
|---|---|
| `// Arrange` `// Act` `// Assert` | Arrange, Act, Assert |
| `// Arrange & Act` `// Assert` | Arrange, Act, Assert |
| `// Arrange` `// Act & Assert` | Arrange, Act, Assert |
| `// Arrange & Act & Assert` | Arrange, Act, Assert |

and the same check rejects an Act with no Assert, an Assert with no Act, a test
with no markers, and an Arrange **dropped rather than merged** -- a bare
`// Act` first is an offence, because the merged form exists to say so.

A marker may carry **trailing prose** after `--`, `:` or `.`; all three are
established style. A marker ends on a **word boundary**, so `// Actually this
needs explaining` is prose. Comment lines above a marker are **folded into it**,
so a marker documented over two lines is not a spacing offence.

| offence | correction |
|---|---|
| `` `new_empty_collection_is_empty` reads Act, Assert; a test is `Arrange` followed by one or more `Act`/`Assert` pairs `` | label the sections `// Arrange`, `// Act` and `// Assert`, merging adjacent ones as `// Arrange & Act`, `// Act & Assert` or `// Arrange & Act & Assert` |
| `` `// Act & Assert` in `a_method_call_on_bare_self…` is not preceded by a blank line `` | put a blank line before `// Act & Assert` |

The markers are comments, and `syn` discards comments -- so this rule reads
lines, and **skips every line a literal occupies**, taken from the token stream.
Without that it reports this repository's own Rust-in-a-raw-string fixtures: a
naive scanner finds seven offences here that are all string literals.

**Does not catch:** whether a section does what it says -- an `// Assert` block
that asserts nothing passes. Nor a **stray or duplicated marker**, since several
`Act`/`Assert` pairs are legal and a copy-pasted `// Act` cannot be told from a
second pair. Nor tests generated by a macro, nor anything outside `tests/`.

See [R017](ADRs/R017-ADR-ArrangeActAssertRule.md).

## `paired-test-file`

A `tests/<path>/<X>_tests.rs` names the source file it exercises, and that file
exists. The counterpart of `tests/a/b_tests.rs` is `src/a/b.rs`, matched **by
path rather than by name alone** -- a test file in the wrong directory is as
unpaired as one whose source is gone.

This is the other side of the pairing from `twin4rust`, which starts at a source
file and looks for its test. Nothing asked the reverse, and a test file outlives
the module it was named for **silently**: it still compiles, still runs, still
passes, and its name now points at nothing.

`all_tests.rs` is exempt -- it ends in `_tests.rs` but is a registry, and would
resolve to `src/all.rs`. `_proptest_tests.rs` is exempt because a property-test
suite is a second suite for a module it does not name, so its stem resolves to a
file nobody meant to write.

| offence | correction |
|---|---|
| `` tests/state/etheram_state_ibft_tests.rs is named for src/state/etheram_state_ibft.rs, which does not exist `` | rename it after the source file it exercises, or delete it if that file is gone |

The correction does **not** say "create the missing file". Measured against a
real tree, every unpaired file tested something real under a name that had
drifted, so the file to create is never the answer.

**It assumes the package is mirrored.** A *harness* crate -- one whose `src/` is
apparatus and whose `tests/` are scenarios named after behaviours rather than
files -- is not, and every one of its test files is reported.
`--skip paired-test-file` is the answer there, and a skipped rule is named as
skipped in the report, so it cannot be mistaken for a pass.

**Does not catch:** whether the name is *honest* -- `widget_tests.rs` beside
`widget.rs` containing tests for something else entirely passes. Nor anything in
a `_proptest_tests.rs` file, by decision. Nor a source file with no test at all,
which is `twin4rust`'s direction. Nor a module that was emptied rather than
removed.

See [R016](ADRs/R016-ADR-PairedTestFileRule.md).

## `test-file-name-postfix`

A file under `tests/` holding at least one test is named `<X>_tests.rs`.

The name is what pairs a test file with the source file it exercises, and that
pairing is the basis of the whole mirrored layout. It was enforced from one side
only: `twin4rust` starts at a source file and looks for its test, so a file full
of tests under any other name was invisible to every tool in the family --
`tests-layout` cared only that a registry existed, `registry-completeness` only
that the file was declared, `test-file-structure` only about the order inside
it, `test-naming` only about the function names.

**One direction only.** Holding a test obliges the name; a `_tests.rs` file
holding none is a different failure with a different fix, and is not this rule.

A test is a function whose attribute's last path segment is `test`, so
`#[tokio::test]` counts. The walk descends into inline modules.

| offence | correction |
|---|---|
| `` tests/rules/widget.rs holds 2 test(s) but its name does not end in `_tests.rs`, so nothing pairs it with the source file it exercises `` | rename it `tests/rules/widget_tests.rs` |

The correction names the exact path, and `expected` carries the same string for
a consumer of the JSON report. The offence sits at line 1, because the file's
name is what is wrong rather than any one test in it.

**Two exemptions, both load-bearing.** `src/` is exempt because a `#[test]`
there is already `test-free-source`'s offence and renaming would not fix it --
that file has to *move*. Registries are exempt because a `#[test]` in an
`all_tests.rs` or `mod.rs` is already `tests-layout`'s, and `mod.rs` cannot be
renamed at all. In both cases this rule's correction would be wrong.

**Does not catch:** an **orphan** -- `banana_tests.rs` with no `banana.rs`
behind it passes, because `<X>` is never resolved against the source tree. That
gap stays open on `twin4rust`'s blind side. Nor a `_tests.rs` file holding no
tests, by decision; nor tests generated by a macro, which never reach the syntax
tree; nor whether the name matches the *right* source file.

See [R015](ADRs/R015-ADR-TestFileNamePostfixRule.md).

## `test-naming`

A test's name has at least three underscore-separated parts, following
`<method>_<conditions>_<result>`.

A test's name is the only part of it anybody reads at the moment it matters:
`cargo test` prints names, not bodies, and `scores_good` says nothing about what
broke. A name with fewer than three parts cannot carry a method, a condition and
a result, whatever its words are.

The rule reads the name and **nothing else**, which is a deliberate retreat.
Three earlier versions tried to verify the leading part was the method actually
under test -- by looking in the body, then through the test file's helpers
transitively, then against the mirrored source file. Measured across 1559 tests
in eight repositories, all three accused correct code, and counting underscores
instead took 592 offences down to 5. See
[R012](ADRs/R012-ADR-TestNamingRule.md).

| offence | correction |
|---|---|
| `` `scores_good` has fewer than 3 parts, so it cannot say what it calls, under what conditions, and with what result `` | rename it `<method>_<conditions>_<result>`, starting with the method `scores_good` calls |

Only `#[test]` functions under `tests/` are judged. A helper beside them is not a
test; `all_tests.rs` and `mod.rs` are registries.

**Does not catch:** whether the first part is the method under test -- the whole
point of the retreat, and answered from the other end by `tested-public-api`. Nor
whether the name is *true*, nor whether the body asserts anything: a three-part
name on an empty test passes. `foo_bar_baz` passes. Tests generated by a macro
are invisible.

## `tested-public-api`

Every public entry point declared in `src/` is called by at least one test.

Two shapes count: a free `pub fn` and a `pub fn` in an inherent impl.

**Neither half of a trait counts.** A method implementing a trait has no
visibility of its own and is reached through the trait rather than named; a
method a trait *declares* is not an implementation at all, so there is no
behaviour behind it to test. Counting declarations while excusing implementations
demanded a fake per trait whose only purpose was to be asserted against.

Matched on **name and arity**. Types and parameter order are not checked and
cannot be without type inference, which this tool does not do. Call sites are
gathered from macro token streams as well as parsed expressions, and that is
load-bearing: a Rust test puts its assertion in `assert!` or `assert_eq!`, whose
contents never become syntax, so skipping them would report the best-tested code
as untested. See [R013](ADRs/R013-ADR-TestedPublicApiRule.md).

| offence | correction |
|---|---|
| `` `with_fixed` is public but no test calls it with 1 argument(s) `` | call `with_fixed` from a test, or stop exposing it if nothing outside needs it |

The correction offers two answers deliberately: an uncalled `pub fn` is as often
over-exposure as it is missing coverage.

**Does not catch:** whether the call *tests* anything -- a bare
`let _ = thing.method();` satisfies it. Matching by name and arity across the
whole test tree means the rule **under-reports**: two entry points sharing both
are indistinguishable, so a test calling one marks both. A call in a test helper
no test ever invokes still counts. Macro-generated entry points, derived methods
and trait impls are all outside its idea of an entry point.

## `tests-layout`

A tests folder is reached through exactly one door, and every door must exist.

- exactly one `tests/all_tests.rs`; one lower down is a file with a misleading
  name that no `pub mod` will ever reach
- a `mod.rs` in **every** folder on the way down, not only those directly
  holding a file — an intermediate folder is a folder too, and a gap there hides
  everything beneath it
- both registry kinds hold nothing but the header and `pub mod` declarations

A declaration is a declaration whether or not it is `pub`; a private `mod name;`
compiles that file just as well, and being compiled is the whole concern. An
inline `mod name { ... }` is not a declaration — it is code hiding in the one
file a reader scans expecting a list.

The failure this rule exists for is silent by construction: **a test that is
never compiled cannot fail.**

| offence | correction |
|---|---|
| a tests folder has no `all_tests.rs` | create `tests/all_tests.rs` with the header and one `pub mod` line per file in `tests/` |
| a tests subfolder has no `mod.rs` | create `<path>` with the header and one `pub mod` line per file in that folder |
| only `tests/all_tests.rs` is a registry | rename it to `mod.rs`, or delete it and declare its contents from `tests/all_tests.rs` |
| the constant `X` does not belong in a registry | move the constant `X` out of the registry into the file that needs it |

**Does not catch:** it verifies a registry *exists*, not that its declarations
are *complete* — that is `registry-completeness`. `#[cfg(...)]`-gated
declarations are treated as ordinary ones.

## `timeless-tests`

A test reads no clock and waits on no timer. **No function in test code may
read a clock, wait on a timer, race one, or draw unseeded randomness.**

A test whose verdict depends on how fast the machine is measures the machine,
and passes on one and fails on another. Whether a test is deterministic cannot
be read from syntax -- it can wait on a clock inside the code it calls --, but
whether the test itself reads one can, and that is the part its author
controls.

- **Test code** is `tests/`, `benches/`, and in `src/` the `#[cfg(test)]`
  modules and the functions carrying a test attribute. Every function in it is
  judged, test or helper.
- **A clock** is `Instant::now` -- `std`'s, Tokio's, `embassy_time`'s --,
  `SystemTime::now`, `chrono`'s `Utc::now` and `Local::now`, `time`'s
  `OffsetDateTime::now_utc` and `now_local`, and an `.elapsed()` read off one,
  which reads `now` inside; `.duration_since(earlier)` is arithmetic between two
  instants and reads none. **A wait** is `std::thread::sleep`,
  `tokio::time::sleep` and `sleep_until`, `async_std::task::sleep`,
  `embassy_time::Timer::after` and `at`. **A race** is `tokio::time::timeout`
  and `timeout_at`, `embassy_time::with_timeout` and `with_deadline`. **An
  unseeded draw** is `rand::thread_rng`, `rand::rng`, `rand::random` and
  `OsRng`.
- **Names are resolved through the file's `use` items** -- a path, a group, a
  rename, `self`, a glob --, so a bare `sleep` imported from `tokio::time` is
  Tokio's, and a function of the file's own called `sleep` is not. A macro's
  arguments are read where they parse as expressions; a string literal is data.
- **On Tokio's paused clock**, `#[tokio::test(start_paused = true)]`,
  `tokio::time` is virtual and let through. `std`'s clock and sleep are not.
- **One offence per function**, at its name, naming the first call. A
  `Duration` is a value and never an offence, and neither is a seeded generator.
- **A harness whose subject is real time** -- real clusters over real sockets,
  hardware -- skips the rule in `stern4rust.toml`: `[package.<name>] skip =
  ["timeless-tests"]`.

| offence | correction |
|---|---|
| `` `acquire_blocks` waits on a timer with `sleep`; a test's verdict must not depend on how fast the machine is `` | `wait for the event itself -- a channel, a barrier, a rendezvous of the parties --, run on Tokio's paused clock, or inject a clock into the code under test and drive it from the test; name a temp directory after the test, or let a temp-directory crate choose` |
| `` `shuffles` draws unseeded randomness with `rand::random`; a test's verdict must not depend on a draw it cannot repeat `` | `seed the generator, StdRng::seed_from_u64, so every run draws the same` |

**Does not catch:** time read inside the code a test calls -- that is
`injected-time`'s question; real concurrency, thread scheduling
and process start-up; other nondeterminism -- `HashMap` order, fixed temp paths
and ports, the network; time handed to a shell, and time a macro generates.

## `injected-time`

Time reaches production code through a port. **In the library target, no
function may read a clock, wait on a timer or race one, unless it implements a
port: a trait the package or a dependency declares.**

A test may not wait on real time (`timeless-tests`), so code with a timeout is
testable only through a clock it was handed -- and a protocol core that reads
none is what a deterministic simulation or a model checker needs. Nothing kept
either from being lost to the next change reaching for `Instant::now()`.

- **The calls** are `timeless-tests`' clocks, waits and races, resolved through
  the file's imports. Randomness is left alone: it is another input, and another
  port.
- **Production code** is the library target -- `src/lib.rs` and what its module
  declarations reach -- outside test functions and `#[cfg(test)]`: free
  functions, inherent methods, a standard trait's methods, through inline
  modules.
- **A port** is a trait the package declares in its `src/`, matched by an
  impl's last segment, or a dependency's, its origin read through the file's
  imports -- a family's core crate declares the ports its runtimes implement.
  Its impls are the adapters a test replaces, and its own default bodies a
  test's implementation overrides; both are left alone. The standard library's
  traits are no seam: `impl Display` reading the clock is reported, and so is a
  bare prelude name such as `Iterator`. An origin the rule cannot tell counts
  as a port.
- **A binary's files** -- `src/main.rs`, `src/bin/`, the modules only they
  declare -- are the composition root, where the real clock is chosen, and are
  left alone, as is a package without a library.
- **Adopted as a guard on a core.** A runtime workspace skips it at its root,
  and the packages meant to be time-free take it back with a section stating
  their own list, `[package.node] skip = []`.

| offence | correction |
|---|---|
| `` `expired` reads the clock with `Instant::now`; take time through a port the package declares, so a test can drive it `` | `declare a trait for the clock or timer it needs, implement it with the real one in an adapter and inject it -- or, for a timer that only marks a deadline, take the deadline in as an event the caller feeds` |

`tests/` is exempt; it is `timeless-tests`'.

**Does not catch:** time inside a dependency, which reads the clock or times out
on its own; time a macro generates; randomness, by decision.

## `records-and-objects`

Every struct is a record or an object, and each has one shape. **In `src/`, a
record with a field that is not `pub`, a record with accessors and an object
with a field that is not private are offences.**

Privacy earns its keep where there is behaviour to protect: a type with methods
can hold an invariant between its fields and change their representation behind
them. A struct with no behaviour has no invariant of its own -- its fields are
its contract -- and accessors around them add code, tests and files and protect
nothing.

**Behaviour** is an inherent method that is none of these, or an impl of a
trait outside the value traits:

- **A getter** takes `&self` and nothing else, and its whole body reads one
  field: `self.f`, `&self.f`, `&*self.f`, or `.clone()`, `.as_ref()`,
  `.as_deref()`, `.as_str()`, `.as_slice()`, `.as_path()`, `.to_vec()`,
  `.to_string()` or `.to_owned()` on it.
- **A plain constructor** has no receiver, returns `Self` or the type, and its
  whole body is `Self { a, b }`, `Type { a: a, b: b }` or `Self(a, b)` with
  every field a parameter.
- **A `with_` step** takes `self` by value and one parameter, and its whole body
  is `Self { field, ..self }`.
- **The value traits** are `Clone`, `Copy`, `Debug`, `Display`, `Default`,
  `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`, `From`, `TryFrom`, `FromStr`,
  `AsRef`, `Error`, `Serialize`, `Deserialize`, `Send` and `Sync` -- a fixed
  list, not configuration.

**A record** has no behaviour: every field `pub`, no getter, no plain
constructor, no `with_` step; it is built by literal, a wide one with
`..Default::default()`. **An object** has some: every field private,
`pub(self)` counting as private, with getters, constructors and steps as it
needs them, and no setters. A constructor that validates, computes or fills a
default is behaviour. An impl counts wherever in the package it is, so the rule
reads the package at once.

| offence | correction |
|---|---|
| `` `Export` is a record with a private field `data`; a record's fields are public `` | `make it pub: a struct with no behaviour has no invariant to keep` |
| `` `Config` is getter-only -- 16 getters, 16 `with_` steps and no behaviour `` | `delete the getters and steps, make the fields pub and build it by literal` |
| `` `Store` is an object exposing `size` and `items`; an object's fields are private `` | `make them private: give a reader a getter and a builder a constructor` |

One offence per struct, at its name, naming the fields -- all of them up to
three, the first two and a count beyond, a tuple struct's by position. A record
with private fields and accessors is reported once, as getter-only. Enum
variants' fields are public by the language and are not this rule's. `tests/`,
`benches/` and `#[cfg(test)]` code are exempt: test support is not an API.

**Does not catch:** shared mutability behind a getter -- `&RefCell<T>`,
`&Mutex<T>`; an object's getters nobody needs; a record that should have an
invariant -- once a constructor checks it, the struct is an object.

## `record-home`

A record or a data enum lives where its consumers put it. **In `src/`, a record
or data enum is `pub` in a file named after it, or private inside the one file
that uses it.**

Nothing else decides it. A record shared by five files may sit in one of them,
and a record only one file uses may have a file of its own: both pass every
other rule, so which one a codebase gets would be taste.

- **A data enum** is an enum with no behaviour, by `records-and-objects`'
  definition. An enum with behaviour is an object, and
  `single-implemented-type`'s.
- **A user** is a file of the workspace, other than the declaring one, that
  names the type -- in code, in a macro's tokens, in a signature, in a `use` --
  tests and other crates included; and the declaring file itself, where its
  code other than the declaration and the type's impls names it.
- **The public API** is what another crate can be handed: the signature of a
  `pub fn`, of a `pub` method and of a `pub` trait's methods; the type of a
  `pub` field of a `pub` struct; a `pub` enum's variants; a `pub` alias,
  `const` or `static`.

The home, first match wins:

1. **Named in the public API** -- `pub`, in a file of its own.
2. **Used nowhere** -- `pub`, in a file of its own: it exists for consumers
   this tool cannot see.
3. **Used by two or more files, or by one outside the crate's `src/`** -- a
   test, another crate -- `pub`, in a file of its own.
4. **Used by one file of `src/`** -- private, inside that file; its fields stay
   `pub`.

**A file of its own** is named after the type, as `named-subject` reads
"named after".

| offence | correction |
|---|---|
| `` `LogLine` is used by 3 files but declared in `src/parser.rs` `` | `move LogLine into src/log_line.rs and make it pub` |
| `` `LogLine` is used by 3 files but is not `pub` `` | `make it pub` |
| `` `Window` is used only by `src/scheduler.rs` but has a file of its own `` | `move Window into src/scheduler.rs and drop its pub` |
| `` `Window` lives in its only consumer, `src/scheduler.rs`, but is `pub` `` | `drop its pub: no other file uses it` |
| `` `Verdict` is named in the crate's public API but declared in `src/report.rs` `` | `move Verdict into src/verdict.rs and make it pub` |

One offence per type, at its name. Only types declared at the top of a file are
judged; `tests/`, `benches/` and `#[cfg(test)]` code are exempt.

**Does not catch:** a consumer in another repository that the public API does
not name -- a record internal to a library that another repository names is told
to go private, and fails loudly there; two types sharing a name, counted as one,
which errs toward `pub` in a file of its own.

## `injected-dependencies`

A type does not build its own dependencies. **In `src/`, a field declared as a
trait receives only what the caller passed in.**

A field declared as a trait is a seam: the type promises its part can be
swapped. A constructor that builds the part itself closes the seam -- the type
is testable only through some other constructor, and the dependency is hidden
from every caller of this one. Parts are built where they are composed: `main`,
a builder, a test.

- **A field declared as a trait** has a type that is, or wraps in `Box`, `Rc`,
  `Arc`, `Option`, `Vec` or a reference, a `dyn Trait`, or a type parameter
  bounded -- inline or in a `where` clause -- by a trait outside the value
  traits.
- **Passed in** is a parameter, as it is or as `Some(p)`, `Box::new(p)`,
  `Rc::new(p)`, `Arc::new(p)`, `p.clone()`, `Rc::clone(&p)`, `Arc::clone(&p)`
  or `p.into()`, or a parameter's field; `None`, `Vec::new()` or `vec![]`; or a
  local bound to any of these, by name or by destructuring -- a local shadows
  the parameter it is named after, and a name rebound from itself reads its
  earlier binding.
- **Judged:** every constructor -- no receiver, a return type naming `Self` --
  and every `Default`, `From` and `TryFrom` impl, for the fields it fills and
  for what it hands another constructor of its type; and every
  `self.field = ...` in any other method.
- **A builder is not judged.** A type with a `build(self)` returning another
  type exists to choose and assemble parts; its product's constructor is
  judged.

| offence | correction |
|---|---|
| `` `App::new` builds `StdoutReporter` for `reporter`, a field declared as a trait `` | `take it as a parameter and build it where the parts are composed: main, a builder, a test` |
| `` `App::new` passes `CargoPackageResolver`, which it built, to `App::with_deps` `` | `make with_deps the constructor and compose the parts in main` |
| `` `Storage::reset` assigns `FlashRegistry::current()` to `flash`, a field declared as a trait `` | `take the replacement as a parameter` |

One offence per field and method, at the value. `tests/`, `benches/` and
`examples/` are exempt: a test is a composition root.

**Does not catch:** a concrete collaborator built inside -- the rule judges the
seams a type declared, not whether it should have declared one; a dependency
used without being stored; a builder in name only.

## What is not walked

- `target/` — generated code nobody wrote
- `.git/`

`--exclude <GLOB>` removes further paths, repeatable and matched against the
package-relative path. It is not a silent skip: every pattern is named in the
report with the number of files it removed, `files_excluded=N` sits in the
summary, and a pattern that matched **nothing** is called out by name so a dead
exclusion can be deleted rather than trusted. An uncompilable pattern is an
error. See [ADR-ExclusionsAreCounted](ADRs/ADR-ExclusionsAreCounted.md).

Nothing else is skipped by default. A nested package with its own `Cargo.toml`
is walked like any other directory: a manifest is a fact about cargo, not about whose conventions
apply, and skipping on sight let a whole tree go unreported with nothing in the
report saying so. Sample code a tool analyses belongs beside the package rather
than inside it — see
[ADR-WalkEveryFileInThePackage](ADRs/ADR-WalkEveryFileInThePackage.md) and
[OPEN_POINTS.md](OPEN_POINTS.md).

## Output

The table is the default. `--format json` renders the same run as a document
with a stable shape, for a gate script or an agent. `--offence-threshold N`
caps how many offences are **printed** — never how many are counted, and never
the exit code.

Exit codes: `0` clean, `1` could not run, `2` at least one rule broken. Only `2`
is a finding. See [ADR-ExitCodeContract](ADRs/ADR-ExitCodeContract.md).
