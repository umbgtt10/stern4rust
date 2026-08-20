# Rules

Fifteen rules, each independent, each naming itself in the report. This is the
reference; the reasoning behind each one is in its ADR.

Every offence carries a **correction** as well as a description — what to do,
not only what is wrong — and the field is required, so a rule cannot be added
without answering it.

| rule | ADR | needs configuration |
|---|---|---|
| `readable-source` | [R004](ADRs/R004-ADR-ReadableSourceRule.md) | no |
| `directory-file-count` | [R010](ADRs/R010-ADR-DirectoryFileCountRule.md) | optional, default 20 |
| `directory-subfolder-count` | [R011](ADRs/R011-ADR-DirectorySubfolderCountRule.md) | optional, default 5 |
| `imported-paths` | [R008](ADRs/R008-ADR-ImportedPathsRule.md) | no |
| `registry-completeness` | [R009](ADRs/R009-ADR-RegistryCompletenessRule.md) | no |
| `test-file-name-postfix` | [R015](ADRs/R015-ADR-TestFileNamePostfixRule.md) | no |
| `test-file-structure` | [R002](ADRs/R002-ADR-TestFileStructureRule.md) | no |
| `test-free-source` | [R005](ADRs/R005-ADR-TestFreeSourceRule.md) | no |
| `tests-layout` | [R003](ADRs/R003-ADR-TestsLayoutRule.md) | no |
| `module-registry` | [R006](ADRs/R006-ADR-ModuleRegistryRule.md) | no |
| `single-implemented-type` | [R007](ADRs/R007-ADR-SingleImplementedTypeRule.md) | no |
| `pure-traits` | [R014](ADRs/R014-ADR-PureTraitsRule.md) | no |
| `test-naming` | [R012](ADRs/R012-ADR-TestNamingRule.md) | no |
| `tested-public-api` | [R013](ADRs/R013-ADR-TestedPublicApiRule.md) | no |
| `header` | [R001](ADRs/R001-ADR-HeaderRule.md) | `--header-file` |

`--rule <NAME>` applies only the named rules; `--skip <NAME>` subtracts. Both
repeatable, both default to everything, and skipping wins over selecting. Every
report names the rules it applied; a run that did not apply all of them says
`All applied rules are satisfied` and names each absence with its reason --
`(skipped)` or `(needs --header-file)`. The JSON carries `rules_applied`,
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
naming the wrong copyright holder, or an SPDX identifier that disagrees with
`Cargo.toml`, passes.

## `test-file-structure`

A test file reads top to bottom in one order: header, imports, constants,
helpers, tests. Each group alphabetical, case-insensitively. Imports run
together; everything else is separated by exactly one blank line.

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

A function is called through a name this file imported, not through a path.

A file's `use` statements are its list of dependencies. `syn::parse_file(...)`
compiles with nothing in the file mentioning `syn`, so a reader scanning the top
to find out what this file needs is quietly given a wrong answer.
`std::env::args()` is a different cost: it spells out at the call site a route
that belongs at the top, and spells it out again at every other call.

Three shapes are left alone. An **unqualified** call has nothing to import. A
**type qualifier** -- `Widget::new()`, `Self::inner()` -- is not a path standing
in for an import, since the type itself was imported and the qualifier says which
type is being constructed. And **one imported segment** -- `use std::fs;` with
`fs::read_to_string(...)` -- is the point of the rule rather than an exception to
it: it names the route once and still says at the call site which module the
function came from. A bare `read_to_string(...)` would satisfy a stricter rule
while saying strictly less.

Module and type are told apart by **case**, a convention rather than a
resolution, because this tool has no type information.

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
undeclared. A file the walker never reached, including anything under
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
most one `struct` or `enum` that is both **declared in the file** and has **at
least one `impl` block** in it. Structs and enums without impl blocks are
unlimited -- plain data is not a subject, and a file's payload types belong
beside the subject that uses them.

Both halves do work. *Declared here*, so an `impl Display for SomeoneElsesType`
does not make this file that type's home. *At least one impl block*, inherent or
trait, because both are behaviour -- though `#[derive(...)]` is not an impl block
in the syntax tree and correctly does not count.

The first implemented type is the subject; every later one is reported, so the
offence names the type to move. The walk descends into inline modules, since
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
covers the other half of that gap.

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

Three shapes count: a free `pub fn`, a `pub fn` in an inherent impl, and every
method of a `pub trait` -- a trait's methods are as public as the trait. A method
*implementing* a trait is not counted; it has no visibility of its own and is
reached through the trait rather than named.

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
