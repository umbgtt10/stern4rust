# R024-ADR-OwnedFunctionsRule

- **Status:** Accepted
- **Date:** 2026-10-19

## Context

[R007](R007-ADR-SingleImplementedTypeRule.md) gives a source file one subject,
and [R023](R023-ADR-NamedSubjectRule.md) names the file after it. Neither sees a
free function standing beside that subject. A file declares `Widget`, gives it an
impl, and then carries `fn border()` below it, called from `draw`; or a
`pub fn parse_log_line()` beside `LogRecord`; or a global accessor beside the
type whose static it reads. Each is a second subject without a name, and the
usual way one arrives is an impl that grew: crap4rust flags a method, and a
helper is pulled out as a free function instead of a struct being extracted.

The house standard already names this: *prefer named structs with methods over
free functions*, and *when crap4rust flags a function as too complex, reduce it
by extracting internal structs with methods -- not by extracting standalone
helper functions*. It was a line in a style guide.

## Decision

**A file with a subject owns its functions.** A source file that declares a
struct, an enum or a trait, or holds an impl block, holds no free function.

- **The subject** is the implemented type named for the file (`FileStem::names`,
  as R007 and R023 read it), else the first implemented type; else the type an
  impl block is for, which is what a file continuing another file's impl is
  about; else a declared type, the one named for the file first; else a trait.
- **A free function** is any `fn` item outside an impl block or a trait: at
  module level, inside an inline module, and nested in another function's body,
  so that nesting a helper inside a method is not a way out.
- **Left out:** a function something other than the author requires to be free --
  an attribute whose last path segment is `alloc_error_handler`, `entry`,
  `exception`, `export_name`, `interrupt`, `main`, `no_mangle`,
  `panic_handler`, `pre_init`, `proc_macro`, `proc_macro_attribute`,
  `proc_macro_derive` or `task`, also inside `#[unsafe(...)]`; an
  `extern "..."` function; and `main` in an entry file (`main.rs`, `src/bin/`,
  `examples/`, `benches/`). Anything under `#[cfg(test)]` is test-free-source's.
- **Exempt files:** `tests/`, where helper functions are what
  test-file-structure asks for; `lib.rs` and `mod.rs`, whose functions
  module-registry already reports; and `build.rs`, a Cargo script.
- **One offence per function**, at its name, with the function as the subject:
  *"`border` is a free function in a file whose subject is `Widget`"*. The
  correction reads the file. A function the subject's impl calls, or whose body
  names the subject, is offered the impl: *"move `border` into `impl Widget` as
  an associated function"*. One that touches neither is offered a file of its
  own first: *"move `checksum` into a file of its own, or into `impl Widget` if
  it belongs there"*. Beside a trait only a file of its own is offered: holding
  the function would take a default body, and
  [R014](R014-ADR-PureTraitsRule.md) forbids one.

A file of functions alone has no subject to compete with and is left alone.
Moving a helper into such a file separates it from the type, which is the
point; requiring a struct there is a separate question, below.

## Forcing constraints / Evidence

Measured before the rule was written, with a census built on stern4rust's own
finders, on the sixteen repositories in the family, each scanned the way its own
stern gate scans it: 2,004 files. The shipped rule, run read-only on the same
trees, reports the same 170 offences, repository by repository.

| repository | offences | files |
|---|---:|---:|
| `etheram-ibft` | 36 | 12 |
| `dry4rust` | 26 | 12 |
| `etheram-raft` | 22 | 7 |
| `etheram-embassy` | 20 | 10 |
| `etheram-tools` | 18 | 11 |
| `etheram-ibft-embassy` | 14 | 6 |
| `etheram-raft-embassy` | 11 | 3 |
| `crap4rust`, `grip4rust` | 8 each | 2, 4 |
| `iceberg4rust` | 4 | 4 |
| `braintax4rust` | 3 | 2 |
| `stern4rust`, `etheram-core`, `slotgate`, `twin4rust`, `beggar` | **0** | |

Of the 170, 43 are private helpers the subject's own impl calls -- the shape the
rule is for, and a mechanical move -- and 96 are `pub`: a second API beside the
type, from `run`/`run_from_args` beside a tool's `App` to accessors over a
static. The exemptions are not decoration: 152 functions across the family are
free because something requires it -- 63 `main`, 45 `#[embassy_executor::task]`,
29 `#[panic_handler]`, 15 `#[exception]` -- and without them no firmware
repository could adopt any version of the rule.

## Rejected alternatives

**No free functions anywhere outside tests/.** It would close every file, not only
those with a subject, and reported 648: the 170 plus 478 in 180 files of
functions alone. Rejected on what the extra 478 would buy. Their fix is a
stateless struct used as a namespace -- `struct Replication; impl Replication {
fn handle_heartbeat(ctx) }` -- which changes the spelling and not the design: a
public free function is as reachable from `tests/` as an associated one, the
module already is the namespace, raft's protocol written as functions over a
context snapshot is a functional core worth keeping, and an accessor over a
static stays one. It is a uniformity choice rather than a quality one, and if it
is wanted it is a rule of its own that a repository can adopt or not.

**Exempting entry files wholesale**, as R023 does. Rejected: a `main.rs` that
declares a type is as much a file with a subject as any other. It costs nothing
today -- no entry file in the family holds an implemented type -- and an entry
file of functions alone is untouched anyway, which keeps every `xtask` clean.

**Reporting only the functions the impl calls.** Rejected: a `pub fn` beside the
type is the larger share and the louder second subject; whether the impl calls
it decides the correction, not the offence.

**Making the exemptions configurable.** Rejected for now: the list is the
language's and the frameworks', not a repository's taste, and a knob is a way to
switch the rule off one attribute at a time. It grows by release.

## Consequences

**A helper beside a type fails the build.** On by default, with the correction
naming where it goes. 170 offences across eleven repositories on the day it
shipped; five are clean.

**The three rules read one file the same way.** R007 bounds the subject, R023
names the file after it, and this rule keeps everything in the file the
subject's. `ImplementedTypeFinder` now takes the types its impl blocks are for
from `ImplBlocks`, which this rule reads too, and the descent into inline
modules that `ImplBlocks` and `pure-traits` each carried is
`InlineModule::flatten`.

### What this rule does not catch

- **A file of functions alone**, by decision, above.
- **Whether the fix is a good one.** A helper moved into the impl as a private
  associated function satisfies the rule exactly as one extracted into a struct
  of its own; the correction steers, and cannot decide.
- **Functions generated by a macro**, invisible to `syn`, and closures stored in
  statics or constants.
- **Anything under `#[cfg(test)]`**, which is test-free-source's.

## Enforcement

`tests/rules/source/owned_functions_rule_tests.rs` -- twenty-one tests covering
the helper the impl calls, a function that never touches the type, one that
names it, one beside plain data, beside a trait, beside an impl for a type
declared elsewhere, inside an inline module and nested in a method, every
function reported once, the attributes and ABI that require a function free,
`main` and a helper in an entry file, the test-only function, associated
functions and trait items, a file of functions alone, the exempt files, the
unparseable file, `check_workspace`, `is_configured`, the name and the
requirement.

`tests/finding/parsing/free_function_finder_tests.rs`, `impl_blocks_tests.rs`,
`identifiers_tests.rs` and `tests/finding/model/free_function_tests.rs` pin the
finder, the impl blocks, the identifier walk and the model on their own.

`tests/rule_registry_tests.rs` -- the four hardcoded rule-name lists include
`owned-functions`.

`tests/runner_tests.rs::run_against_this_crate_with_its_own_header_is_clean` and
stage 2 both run the tool against this crate and require zero offences.

## Related

- [R007-ADR-SingleImplementedTypeRule](R007-ADR-SingleImplementedTypeRule.md),
  [R023-ADR-NamedSubjectRule](R023-ADR-NamedSubjectRule.md) -- one subject per
  file, the file named after it; this rule keeps the rest of the file its.
- [R014-ADR-PureTraitsRule](R014-ADR-PureTraitsRule.md) -- why a trait cannot
  take a free function.
- [R022-ADR-ReturnedMutationsRule](R022-ADR-ReturnedMutationsRule.md) -- the same
  move: a written house standard turned into a gate.
