# Architecture

`cargo-stern4rust` reads every `.rs` file in a package, asks each registered
rule what is wrong with it, and prints the answers in one table — or one JSON
document — sorted by file.

There is no metric. Unlike `crap4rust`, `iceberg4rust` and `grip4rust`, nothing
here is scored or thresholded against a formula: a rule is either satisfied or
it is not. That is why this repository has a `RULES.md` where those have a
`FORMULA.md`.

## Two crates

The rules and everything that decides a verdict live in `stern4rust-core`
(`core/`, library `stern4rust`): `#![no_std]`, depending on globset,
proc-macro2, quote and syn only, and reaching the machine through five ports it
declares in `core/src/traits/`. The command, `cargo-stern4rust` (`cli/`, library
`stern4rust_cli`), implements them, parses the arguments, prints and decides the
exit code. See [ADR-TechnologyFreeCore](ADRs/ADR-TechnologyFreeCore.md).

| port (`core`) | what it answers | implemented by (`cli`) |
|---|---|---|
| `SourceTree` | the `.rs` files under a root, in path order, `target/` and `.git/` left out; a file's text; whether a path exists; writing a repaired file back | `WalkdirSourceTree` |
| `WorkspaceReader` | the workspace, once a run: its root, whether it declares a workspace, each member's name, manifest, licence and declared dependencies | `CargoWorkspaceReader` (`cargo metadata --no-deps`, each member manifest read as TOML through `DependencyTables`) |
| `ConfigSource` | the `stern4rust.toml` beside a manifest, if any | `TomlConfigSource` |
| `BaselineStore` | a baseline's counts, and keeping them | `JsonBaselineStore` |
| `ReportRenderer` | the report and the rule listing as text | `TextReportRenderer` (in `core`), `JsonReportRenderer` |

## Pipeline

```
cli: main ──▶ EntryPoint: Args ──▶ Invocation, the adapters, the renderer --format picks
                         │
core: Runner::run(&Invocation)
                         │
      ConfigSource ──▶ ConfigFile, PackageSections ──▶ Config ──▶ RuleRegistry
                         │
      WorkspaceReader ──▶ WorkspaceDescription ──▶ ManifestResolver ──▶ packages
                         │
      SourceTree::rust_files  (skips target/ and .git/, nothing else)
                         │
      SourceReader  ──▶ SourceFile   or  Offence
                         │
              ┌──────────┴──────────┐
       rule.check(file)     rule.check_workspace(&files)
              └──────────┬──────────┘
                     Vec<Offence>
                         │
          dedupe by content, sort by (file, line, rule), BaselineStore
                         │
      RunReport ──▶ ReportRenderer   (OffenceThreshold caps what prints)
                         │
cli: (RunOutcome, text) ──▶ println, exit code
```

The whole package is read before any of it is judged. Six of the rules answer
questions about the *tree* rather than about a file, and one of them reports
offences against files that do not exist — see
[ADR-WorkspaceRuleSeam](ADRs/ADR-WorkspaceRuleSeam.md). Reading first also
removes an order dependency: results cannot vary with the order the walker
happened to return files in.

## Components

| Component | Responsibility |
|---|---|
| `Args` (cli) | clap surface, plus the argv fixup every cargo subcommand needs; builds the `Invocation` |
| `EntryPoint` (cli) | the composition root: parses, chooses the adapters and the renderer, runs, prints |
| `Invocation` | what the command line asked, in the core's terms |
| `Config` | what a run was asked to do, decoupled from how it was asked |
| `ManifestResolver` | package name → source root, over the `WorkspaceDescription`; also relativises paths |
| `DependencyTables` (cli) | one manifest's dependency entries as written -- plain and `[target.<cfg>]` tables, with each entry's workspace, version, features, default features and optional |
| `WalkdirSourceTree` (cli) | every `.rs` path under a root, minus `target/` and `.git/`; reads and writes files |
| `SourcePath` | a path as text, `/` its only separator -- the core's only path |
| `SourceReader` | path → `SourceFile`, or an `Offence` if it cannot be read |
| `SourceFile` | normalised contents: BOM stripped, `\r` stripped, path forward-slashed |
| `Rule` | the seam: `name`, `check` for one file, `check_workspace` for the set, `is_configured`, `requirement` for why it could not run, `explanation` for what it wants. No method has a default body, so every rule answers all six |
| `RuleRegistry` | the one place that knows which rules exist |
| `Offence` | the currency every rule reports in |
| `OffenceThreshold` | how much of the report is printed |
| `RunReport` | everything a run has to say, before anybody chooses how to say it |
| `ReportPrinter`, `JsonPrinter` (cli) | the two renderings of a run, behind `TextReportRenderer` and `JsonReportRenderer` |
| `RuleExplanation` | what one rule wants, in the rule's own words |
| `RuleListing` / `JsonRuleListing` (cli) | the rule set as text or as a document -- no run behind it |
| `RunOutcome` | clean or rules-broken, turned into an exit code only by `main` |
| `RunFailure` | why a run could not judge anything, worded as the run has always worded it |

`src/finding/` is grouped into `model/` (what a rule reasons about) and
`parsing/` (what turns a file into it), with `tests/finding/` mirroring exactly.
`directory-file-count` decided that split too: the directory stood at 20 files.

`ItemNaming` in `parsing/` holds the identifier an item declares and the
source-line fallback for items that declare none. `TestFileParser`,
`RegistryParser` and `UnitTestFinder` each carried both; what they legitimately
differ on -- the wording around the name -- stays with each of them, because
every one of those strings sits inside an offence description that baselines are
keyed on.

A set of finders and parsers sits beside the rules in `src/finding/`, each
turning a file into the items one rule reasons about: `TestFileParser` (sections
and ordering), `RegistryParser` (strays in an `all_tests.rs` or `mod.rs`),
`UnitTestFinder` (tests and test machinery in the source tree),
`ImplementedTypeFinder`, `DeclaredItemFinder`, `FreeFunctionFinder`,
`ImplBlocks`, `QualifiedCallFinder`, `ModuleDeclarationFinder`,
`PublicEntryPointFinder` and `CallSiteFinder`. `PackageTree` models the
directory shape the tree-wide rules ask about.

A rule needs one only when the finding is worth testing on its own. The simplest
rules — `pure-traits`, `test-naming` — parse the file directly and keep the
whole answer in one place.

## Data model

`Offence` is the only currency. Every rule reports in it, so the report is one
table rather than one section per rule, and a new rule costs nothing in the
printer.

```rust
pub struct Offence {
    pub file: String,
    pub line: usize,
    pub rule: &'static str,
    pub description: String,      // what is wrong
    pub correction: String,       // what to do about it -- required
    pub subject: Option<String>,  // the thing it is about, named
    pub expected: Option<String>, // the correct text, where the rule knows it
}
```

`correction` is required while `subject` and `expected` are not, and the
asymmetry is deliberate: a rule that can say what is broken can always say how
to fix it, so an `Option` there would only ever be used to skip the half of the
report worth acting on. `subject` and `expected` are genuinely absent for some
offences — there is no "expected text" for a missing folder.

## Adding a rule

1. a file under `src/rules/<group>/`, implementing `Rule` — all six methods,
   since none of them has a default body. That includes `explanation`, so a
   rule cannot join the set without saying what it wants and what breaking it
   looks like. The groups are `source/`, `layout/`,
   `testing/`, `manifest/` and `types/`; `tests/rules/` mirrors them exactly
2. a `pub mod` line in that group's `mod.rs`
3. an import and one entry in `RuleCatalog::all`, the single list both
   `selected` and `known_names` read
4. a mirrored test file under `tests/rules/<group>/`, declared in that group`s `mod.rs`
5. an `R<NNN>-ADR-<Name>.md` in `docs/ADRs/`, and a section in `RULES.md`

Nothing else in the tool changes. A rule does not walk, does not print, and does
not know which other rules exist. The rule-name lists in
`tests/rule_catalog_tests.rs` are the one place that has to agree, and they fail
loudly when it does not.

## Analysis is AST-structural

Every rule works from `syn`'s syntax tree or from the normalised lines, never
from type resolution. The tool runs on any syntactically valid source whether or
not it compiles, and needs no build. The one consequence worth stating: plain
`//` comments never reach the AST, so `TestFileParser` folds the comment lines
above an item back into that item by hand — otherwise every documented test
reads as a spacing offence.

## Paths

Paths are relative to the package root and forward-slashed on every platform,
normalised once in `SourceFile` so no rule has to remember. The same
normalisation strips a leading BOM and the `\r` of CRLF line endings, which is
what lets the header rule compare exactly without failing on every file in a
Windows working copy.

## CLI layer

`main` does two things: hand argv to `EntryPoint`, and turn the `RunOutcome`
into an exit code -- or an `Err` into `stern4rust: {error:#}` and `1`. It is the
only place that knows `2` means "rule broken" rather than "tool failed", which
is what keeps the whole run reachable from a test without a process boundary.
`EntryPoint` parses the arguments, builds the `Invocation`, chooses the
adapters and the renderer, runs the core and prints what it says; the core
itself never prints.

`--rules` is the one path that returns before the pipeline above runs at all:
nothing is walked, read or judged, so it works in a checkout with no manifest
worth reading and cannot fail the way a run can.

## Related

- [RULES.md](RULES.md) — what each rule requires, and what it does not catch
- [ADRs/](ADRs/README.md) — the load-bearing decisions, `R` for rules
- [ADR-MachineReadableReport](ADRs/ADR-MachineReadableReport.md) — the report's shape
- [ADR-ExitCodeContract](ADRs/ADR-ExitCodeContract.md) — `0` / `1` / `2`
- [ADR-RulesExplainThemselves](ADRs/ADR-RulesExplainThemselves.md) — why the
  explanation lives on the rule
