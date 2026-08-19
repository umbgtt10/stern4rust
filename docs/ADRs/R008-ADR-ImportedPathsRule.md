# R008-ADR-ImportedPathsRule

- **Status:** Accepted
- **Date:** 2026-08-19

## Context

A file's `use` statements are its list of dependencies. A path written inline at
the call site is a dependency that never reaches that list.

That is the cost, and it is paid by the reader rather than the writer.
`syn::parse_file(...)` compiles perfectly well with nothing in the file
mentioning `syn`, so someone scanning the imports to find out what this file
needs is quietly given a wrong answer. `std::env::args()` is worse in a
different way: it spells out at the call site a route that belongs at the top,
and spells it out again at every other call.

The house standard already says so — *"do not use fully qualified paths; use
`use` imports instead."* This rule is that sentence made checkable.

## Decision

A call is either unqualified, qualified by a type, or qualified by exactly one
segment that this file imported. Anything else is reported.

Three shapes are deliberately left alone.

**An unqualified call** has nothing to import.

**A type qualifier** — `Widget::new()`, `Self::inner()`, `Duration::from_secs(1)`
— is not a path standing in for an import. The type itself was imported, and the
qualifier says which type is being constructed. Uppercase-initial first segments
are read as types: a convention, not a resolution, since this tool has no type
information.

**One imported segment** — `use std::fs;` followed by `fs::read_to_string(...)`
— is the point of the rule rather than an exception to it. It names the route
once at the top and still tells the reader at the call site which module the
function came from. A bare `read_to_string(...)` would satisfy a stricter rule
while saying strictly less.

What is left is a path doing an import's job, and the correction says exactly
which import to add. Two shapes, because they want different splits:

| offence | correction |
| --- | --- |
| `syn::parse_file(...)` | add `use syn::parse_file;` and call `parse_file` |
| `std::env::args()` | add `use std::env;` and call `env::args` |

The two-segment case imports the whole path, because `use syn;` would be legal
and leave the call site unchanged. The longer case imports all but the last
segment, keeping `env` because `env::args()` reads better than a bare `args()`.

Both productive and test files are checked. Unlike
[R007](R007-ADR-SingleImplementedTypeRule.md), `tests/` gets no exemption: a
test file has the same reader and the same list of dependencies at its top.

## Forcing constraints / Evidence

The rule found **15 offences in this tool's own source** the moment it was
registered, including five `syn::parse_file` calls — one of them inside the
finder that implements this very rule. `etheram-core`, the repository the
standard comes from, is at **0**. The six sibling tools hold 201 between them.

That split is the evidence: the standard is real and kept where it was written
down, and drifts everywhere it was not enforced.

Two facts were measured rather than assumed, and both changed the work.

**Macros are excluded.** `serde_json::json!(...)` is an `ExprMacro`, not an
`ExprCall`, so it never reaches this rule. A macro is not a function and its
path resolves under different rules; including it was not worth the risk of
being wrong about attribute and derive macros.

**A regex estimate was 39% low.** A pre-survey put the family at 123; the built
rule found 201. The gap is almost entirely turbofish — `from_str::<Value>(...)`
does not match a pattern expecting `(` after the path. The same shape then broke
the script that fixed this tool's own offences, which asserted the path was
present but replaced `path + "("`, so two calls were silently skipped. Both were
caught only because the rule re-checked the result.

## Consequences

**It exposed a latent bug in [R002](R002-ADR-TestFileStructureRule.md).** Adding
`use serde_json::from_str;` beside an existing `use serde_json::Value;` created a
file that could not be made green: `cargo fmt` wrote one order and
`test-file-structure` demanded the other. The stand-down that exists to prevent
exactly this was keyed on an import's **first** segment, and these two paths
share theirs and diverge at the second. It is now decided per pair —
`ImportPath::decides_order` — standing down where the two paths first differ and
the segments there are of different case.

Measuring rustfmt to fix it turned up something worth writing down: **it treats
case as significant in opposite directions at the two levels.** An
uppercase-initial crate sorts *behind* every lowercase one (`Bbb::gamma` after
`zzz::last`), while an uppercase-initial segment later in a path sorts *ahead* of
its lowercase siblings (`serde_json::Value` before `serde_json::from_str`). Also
note that `cargo fmt` and a standalone `rustfmt <file>` disagree here; only
`cargo fmt` matters, because that is what stage 1 runs.

**What this rule does not catch.** Paths outside call position: a
`let x: std::path::PathBuf` or a `std::fmt::Result` return type passes
untouched, because the standard as written is about function and method
qualifiers. Macro paths, as above. And it cannot tell a module from a type
without type information, so a lowercase-named type or an uppercase-named module
would be judged by its case rather than by what it is — a cost accepted for
staying AST-only.

**It does not check `src/` import ordering.** That belongs to R002, which is
scoped to `tests/`. This rule adds imports to productive files without any rule
governing where they land; `cargo fmt` is the only authority there.

## Rejected alternatives

**Forbid any lowercase qualifier, so every call is unqualified.** Rejected, and
this was the live decision — measured at 168 offences against 123 for the rule
as built. It would turn `fs::read_to_string(path)` into `read_to_string(path)`,
which is less informative at the call site, and it reads "fully qualified path"
as meaning something narrower than it does. A one-segment module qualifier is
not a fully qualified path.

**Require the qualifier to be imported, but allow multi-segment paths whose root
is imported.** Rejected: `use std::env;` followed by `std::env::args()` would
pass, which is the exact shape the rule exists to remove.

**Treat every dependency as implicitly in scope.** In edition 2018 and later
every crate in `Cargo.toml` is resolvable from the root, so `syn::parse_file`
*is* "in scope" in a compiler sense. Rejected because it would make the rule
vacuous: it would allow every path it was written to catch. The rule is about
what the file says, not about what resolves.

**Include macro invocations.** Rejected for now — see above. If a
`serde_json::json!` inline path turns out to bother a reader as much as a
function call does, it is a small change to `visit_expr_macro`.

## Enforcement

`tests/rules/imported_paths_rule_tests.rs` — 7 tests covering an imported
qualifier, a two-segment path, a multi-segment path and its correction, a test
file getting no exemption, an unparseable file, and that every offending call is
reported rather than the first.

`tests/qualified_call_finder_tests.rs` — 13 tests pinning each stand-down
independently: unqualified, type-qualified, `Self`-qualified, imported module,
renamed import, grouped import, descent into function bodies, and macros.

`tests/qualified_call_tests.rs` — 5 tests on the two correction shapes.

`run_stage_2.ps1` runs the compiled binary against this crate's own tree, which
is what surfaced all 15 of its own offences.

## Related

- [R002-ADR-TestFileStructureRule](R002-ADR-TestFileStructureRule.md) — the
  ordering stand-down this rule forced from per-import to per-pair.
- [R006-ADR-ModuleRegistryRule](R006-ADR-ModuleRegistryRule.md) — the other rule
  about what may appear at the top of a file.
