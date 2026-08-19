# Architecture

`cargo-stern4rust` reads every `.rs` file in a package, asks each registered
rule what is wrong with it, and prints the answers in one table — or one JSON
document — sorted by file.

There is no metric. Unlike `crap4rust`, `iceberg4rust` and `grip4rust`, nothing
here is scored or thresholded against a formula: a rule is either satisfied or
it is not. That is why this repository has a `RULES.md` where those have a
`FORMULA.md`.

## Pipeline

```
Args ──▶ Config ──▶ RuleRegistry
                         │
ManifestResolver ──▶ package roots
                         │
                   SourceWalker  (skips target/, .git/, nested packages)
                         │
                   SourceReader  ──▶ SourceFile   or  Offence
                         │
              ┌──────────┴──────────┐
       rule.check(file)     rule.check_workspace(&files)
              └──────────┬──────────┘
                     Vec<Offence>
                         │
                   sort by (file, line, rule)
                         │
              ReportPrinter │ JsonPrinter   (OffenceThreshold caps what prints)
                         │
                    RunOutcome ──▶ exit code
```

The whole package is read before any of it is judged. Two of the rules answer
questions about the *tree* rather than about a file, and one of them reports
offences against files that do not exist — see
[ADR-WorkspaceRuleSeam](ADRs/ADR-WorkspaceRuleSeam.md). Reading first also
removes an order dependency: results cannot vary with the order the walker
happened to return files in.

## Components

| Component | Responsibility |
|---|---|
| `Args` | clap surface, plus the argv fixup every cargo subcommand needs |
| `Config` | what a run was asked to do, decoupled from how it was asked |
| `ManifestResolver` | package name → source root, via `cargo_metadata`; also relativises paths |
| `SourceWalker` | every `.rs` path under a root, minus `target/`, `.git/` and nested packages |
| `SourceReader` | path → `SourceFile`, or an `Offence` if it cannot be read |
| `SourceFile` | normalised contents: BOM stripped, `\r` stripped, path forward-slashed |
| `Rule` | the seam: `check` for one file, `check_workspace` for the set |
| `RuleRegistry` | the one place that knows which rules exist |
| `Offence` | the currency every rule reports in |
| `OffenceThreshold` | how much of the report is printed |
| `ReportPrinter` / `JsonPrinter` | the two renderings |
| `RunOutcome` | clean or rules-broken, turned into an exit code only by `main` |

Three parsers sit beside the rules, each turning a file into the items one rule
reasons about: `TestFileParser` (sections and ordering), `RegistryParser`
(strays in an `all_tests.rs` or `mod.rs`), and `UnitTestFinder` (tests and test
machinery in the source tree).

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

1. a file under `src/rules/`, implementing `Rule`
2. one line in `RuleRegistry::from_config`
3. a mirrored test file under `tests/rules/`
4. an `R<NNN>-ADR-<Name>.md` in `docs/ADRs/`

Nothing else in the tool changes. A rule does not walk, does not print, and does
not know which other rules exist.

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

`main` does three things: parse argv, call `Runner::run`, and turn the
`RunOutcome` into an exit code. It is the only place that knows `2` means
"rule broken" rather than "tool failed", which is what keeps the whole run
reachable from a test without a process boundary.

## Related

- [RULES.md](RULES.md) — what each rule requires, and what it does not catch
- [ADRs/](ADRs/README.md) — the load-bearing decisions, `R` for rules
- [ADR-MachineReadableReport](ADRs/ADR-MachineReadableReport.md) — the report's shape
- [ADR-ExitCodeContract](ADRs/ADR-ExitCodeContract.md) — `0` / `1` / `2`
