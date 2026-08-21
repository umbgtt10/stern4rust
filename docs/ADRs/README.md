# Architecture Decision Records

Each ADR documents one load-bearing decision behind `cargo-stern4rust` —
succinct, self-contained, citable on its own.

There are two kinds, and the filename says which.

**`R<NNN>-ADR-<Name>.md` — one per rule.** A rule is the unit a reader argues
with when it fails their build, so it is the unit that has to justify itself.
The number is the rule's identity: it is allocated in the order the rules were
built, never reused, and a superseded ADR keeps its number and says what
replaced it, so a reference to `R002` in a commit message or a code comment
stays resolvable forever. A rule also has an identity *outside* these docs —
it prints its own name in every offence it emits — which is what the number
indexes onto.

**`ADR-<Name>.md` — everything else.** The exit-code contract, the shape of
the report, the seams in the `Rule` trait. These are unnumbered, as in
`crap4rust` and `twin4rust`, because they have no external referent to index
against and the name is a better identifier than a number would be:
`ADR-ExitCodeContract` says more than `A002` ever could.

The split is also legible in a directory listing. Anything with an `R` is a
rule; anything without is the machinery the rules run on.

## Rules

| ADR | Rule | Decision |
|---|---|---|
| [R001](R001-ADR-HeaderRule.md) | `header` | The expected header is data supplied by `--header-file`, not a constant; a file satisfies the rule when its first N lines match exactly after normalisation, and exactly one offence — the first divergence — is reported per file. |
| [R002](R002-ADR-TestFileStructureRule.md) | `test-file-structure` | A test file is four sections in a fixed order, each alphabetical, with spacing as part of the shape — and `Helpers` is defined by exclusion so the set of item kinds stays closed. |
| [R003](R003-ADR-TestsLayoutRule.md) | `tests-layout` | A tests folder is reached through exactly one `all_tests.rs` and a `mod.rs` in every subfolder on the way down. |
| [R004](R004-ADR-ReadableSourceRule.md) | `readable-source` | Every `.rs` file must be readable and must parse, and failing either is an offence rather than a reason to say nothing — a file the tool cannot read otherwise reads as a file with nothing wrong with it. |
| [R007](R007-ADR-SingleImplementedTypeRule.md) | `single-implemented-type` | A source file holds at most one type that is both declared there and carries an impl block. Plain data declarations are unlimited -- they are not subjects. |
| [R006](R006-ADR-ModuleRegistryRule.md) | `module-registry` | A `lib.rs` or `mod.rs` outside `tests/` holds the header, inner attributes, `extern crate alloc;` and `pub mod` declarations -- nothing else. Catches the re-export shim where it forms. |
| [R005](R005-ADR-TestFreeSourceRule.md) | `test-free-source` | Tests live in `tests/` and the production source tree carries none of them; `#[cfg(test)]`, `#[cfg_attr(test, ...)]` and test-attributed functions are all offences outside `tests/`. The line is `test` — feature gates ship to somebody and are left alone. |
| [R008](R008-ADR-ImportedPathsRule.md) | `imported-paths` | A function is called through a name the file imported, not through a path. One imported segment stays legal; a path no import names does not. Type qualifiers are told from modules by case. |
| [R009](R009-ADR-RegistryCompletenessRule.md) | `registry-completeness` | A registry declares every module beside it. Only the silent direction is checked -- `pub mod x;` with no `x.rs` is a compile error rustc already reports, while an orphan `x.rs` produces no error and no warning at all. |
| [R010](R010-ADR-DirectoryFileCountRule.md) | `directory-file-count` | A directory holds at most 20 `.rs` files, not counting its own index. The one rule whose number is taste rather than fact, which is why the limit is configuration. |
| [R011](R011-ADR-DirectorySubfolderCountRule.md) | `directory-subfolder-count` | At most 5 subfolders per directory, checked at every level. The counterweight to R010, so a folder per file is not the cheapest way to satisfy it. |
| [R012](R012-ADR-TestNamingRule.md) | `test-naming` | A test's name has at least three underscore-separated parts. The name and nothing else -- three attempts to verify the leading part was the method under test each accused correct code, and were abandoned. |
| [R013](R013-ADR-TestedPublicApiRule.md) | `tested-public-api` | Every public entry point is called by at least one test, matched on name and arity. The question R012 gave up on, asked from the declaration instead of the name. |
| [R019](R019-ADR-OrderedImportsRule.md) | `ordered-imports` | Imports in `src/` run in alphabetic order, on the pairs where the alphabet is the authority. Reuses `ImportPath`, because `cargo fmt` runs first and orders `self`/`super`/`crate` and uppercase paths itself -- demanding the alphabet there writes a file no edit can make green. |
| [R021](R021-ADR-WorkspaceDependenciesRule.md) | `workspace-dependencies` | A workspace declares its dependencies once, in the root, and every member takes them with `.workspace`. The three requirements asked for are one check -- `cargo` refuses to build a `workspace = true` the root does not declare. Read from the TOML, since resolution erases how a dependency was written. |
| [R020](R020-ADR-SpdxMatchesManifestRule.md) | `spdx-matches-manifest` | Every file's `SPDX-License-Identifier` says what the manifest's `license` says. The first rule configured from the package being judged rather than a flag, so it needs no `--header-file`; a manifest naming no licence leaves it unconfigured rather than offended. |
| [R018](R018-ADR-DeclaredByNameRule.md) | `declared-by-name` | A module is declared by name; `#[path = "..."]` on a `mod` is an offence. It exists to hold up the convention `registry-completeness` resolves by -- without it that rule reports a perfectly compiled file as never compiled. `cfg_attr`-gated paths are left alone. |
| [R017](R017-ADR-ArrangeActAssertRule.md) | `arrange-act-assert` | A test reads `Arrange` then one or more `Act`/`Assert` pairs, sections separated by a blank line. The markers are comments, which `syn` discards, so the rule reads lines -- and skips the lines every literal occupies, without which it reports this repository's own string fixtures. |
| [R016](R016-ADR-PairedTestFileRule.md) | `paired-test-file` | A `tests/a/b_tests.rs` names the source file it exercises and `src/a/b.rs` exists, matched by path rather than by name alone. The other side of the pairing from `twin4rust`. `_proptest_tests.rs` and `all_tests.rs` are exempt; a harness crate skips the rule. |
| [R015](R015-ADR-TestFileNamePostfixRule.md) | `test-file-name-postfix` | A file under `tests/` holding at least one test is named `<X>_tests.rs`. One direction only -- holding a test obliges the name. `src/` and registries are exempt because the rule that owns each already reports them, and this rule's correction would be wrong for both. |
| [R014](R014-ADR-PureTraitsRule.md) | `pure-traits` | No method in a `trait` declaration may have a default body -- a default hides which question an implementor actually answered. The other half, that every implementor implements every method, is `rustc`'s `E0046` and needs no rule. |

## Everything else

| ADR | Decision |
|---|---|
| [ADR-Baselines](ADR-Baselines.md) | Record the current offences and fail only on new ones, keyed on file + rule + subject with the line deliberately excluded so an offence that moved is the same offence. Counts rather than a set; every run that used one names it and says how many it hid. |
| [ADR-ExclusionsAreCounted](ADR-ExclusionsAreCounted.md) | `--exclude` reports every pattern with the number of files it removed, including zero, so a tree taken out of the report is something the reader can see rather than a silence. |
| [ADR-FixOnlyWhatIsSafe](ADR-FixOnlyWhatIsSafe.md) | `--fix` repairs only what can be moved without reading it -- item order, section order, blank lines -- working from `syn` spans, and never touches imports, which are rustfmt's decision. |
| [ADR-ExitCodeContract](ADR-ExitCodeContract.md) | `0` clean, `1` could-not-run, `2` rule-broken, and the line between `1` and `2` is whether the work can still be enumerated — a bad manifest is a `1`, one unreadable file among fifty is a `2`. |
| [ADR-LibrarySurfaceIsNotAnApi](ADR-LibrarySurfaceIsNotAnApi.md) | The `stern4rust` library is not a public API -- depend on the binary. Everything is `pub` because unit tests are forbidden and integration tests can only reach what is public, so the surface is as wide as the test suite needs and moves whenever the rules require it. |
| [ADR-ManifestDataIsPerPackage](ADR-ManifestDataIsPerPackage.md) | The licence and the dependency list are read per package inside the scan loop rather than once for the run. `license` compared a set of distinct licences against a count of packages, so it could only ever answer for a single-package scan -- four members all declaring Apache-2.0 read as none declaring it. |
| [ADR-PerPackageConfiguration](ADR-PerPackageConfiguration.md) | One `stern4rust.toml` at the workspace root with a `[package.<name>]` section each, replacing rather than merging, so a whole workspace is judged in one call with rules that differ by member. `baseline` and `offence-threshold` stay root-only; a section naming no package is an error. |
| [ADR-MachineReadableReport](ADR-MachineReadableReport.md) | The table stays the default and `--format json` renders the same run as a document; every offence carries a required `correction` alongside its description, because a report worth reading is not the same as a report worth acting on. |
| [ADR-RuleSelection](ADR-RuleSelection.md) | `--rule` and `--skip`, both repeatable, with skipping winning over selecting; a run that switched rules off never claims all of them are satisfied. |
| [ADR-RulesExplainThemselves](ADR-RulesExplainThemselves.md) | Each rule says what it wants through `explanation()` on the trait, with no default body, so a rule cannot be added without answering it; `--rules` renders all twenty-one in either format from one list. |
| [ADR-WalkEveryFileInThePackage](ADR-WalkEveryFileInThePackage.md) | The walker skips `target/` and `.git/` and nothing else; a nested package with its own manifest is judged like any other directory, because a linter quietly declining to look at part of a tree is the silence this tool refuses. Supersedes the `0.2.0` nested-package skip. |
| [ADR-WorkspaceRuleSeam](ADR-WorkspaceRuleSeam.md) | `Rule` carries `check_workspace` beside `check`, because some offences are about a tree rather than a file — and the file that carries such an offence is usually the one that does not exist. Both are answered explicitly by every rule; the original defaults were removed by [R014](R014-ADR-PureTraitsRule.md). |

## Template

```markdown
# R<NNN>-ADR-<Name>   (a rule)
# ADR-<Name>          (anything else)

- **Status:** Accepted | Proposed | Superseded by <ADR>
- **Date:** YYYY-MM-DD

## Context
The forces and tension this resolves.

## Decision
The choice, in one quotable sentence.

## Forcing constraints / Evidence
Why this was forced, not freely chosen — the real evidence. `N/A` if none.

## Rejected alternatives
What we did not do, and why.

## Consequences
What it commits us to; what it costs; obligations pushed onto consumers.

## Enforcement
The specific test, gate, or structural mechanism that keeps it true.
`N/A` if purely structural.

## Related
Links to other ADRs (this repo, `crap4rust`, `twin4rust` or `iceberg4rust`)
and architecture docs.
```

Fields that do not apply are marked `N/A` rather than padded. Each ADR is a
snapshot of the decision as it stands today, not a changelog — state the
current shape as fact, don't narrate what an earlier version of this
document used to say.

## Two obligations specific to rule ADRs

An `R` ADR must say what the rule **does not** catch. A checking tool is read
as exhaustive by default, and a gap nobody wrote down is indistinguishable
from a gap nobody has yet hit.

It must also say what each of the rule's offences tells the reader to **do**.
`Offence::correction` is a required field rather than an optional one, so
every offence a rule can emit has an answer to "and now what?" — and the ADR
is where the wording of those answers is argued about rather than in a diff.

Neither obligation applies to the unnumbered ADRs, which decide how the tool
behaves rather than what it requires of a codebase.
