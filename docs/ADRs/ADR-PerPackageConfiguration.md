# ADR-PerPackageConfiguration

- **Status:** Proposed
- **Date:** 2026-08-21

## Context

`stern4rust.toml` configures a run. A run judges whatever `--package` selected,
and every file in it answers to the same rule set. For a single-package
repository that is the whole story, and it is the story every repository in this
family told until now.

A workspace breaks it. `etheram-raft` holds four members and they do not want the
same rules:

| package | what it holds | rules |
|---|---|---|
| `node` | the protocol | all twenty-one |
| `node-infra` | the adapters around it | all twenty-one |
| `validation` | scenario tests over a built cluster | twenty |
| `system-tests` | the same, over real processes | twenty, minus generated code |

The difference is not taste. Every test in `validation` and `system-tests` is
named for a scenario — election, fault tolerance, replication — rather than for a
source file, so `paired-test-file` has nothing to match them against. All
fourteen of its offences there were of that shape. And `system-tests` carries two
files `prost` writes and says so on their first line, which is a question of
which files are input rather than which rules are wrong.

Today that is expressible only by giving each member its own `stern4rust.toml`
and calling the tool four times. It works — `ConfigFile::load` reads from the
directory of the `--manifest-path` it is handed, so a file beside a member's
manifest is that member's configuration — but it costs four files, four
invocations, and a trap:

```
cargo stern4rust --manifest-path node/Cargo.toml            # 243 files: the whole workspace
cargo stern4rust --manifest-path node/Cargo.toml --package node   # 127 files: node
```

The manifest path selects the *configuration*; `--package` scopes the *scan*.
Passing only the first measures everything and reads as though it measured one
thing. Nothing warns.

The four files also repeat what they agree on. `header-file` is the same in all
of them and has to be written `../docs/header.txt` in each, because each resolves
from its own directory.

## Decision

**One configuration file at the workspace root, with a section per package.**

```toml
header-file = "docs/header.txt"
baseline    = "stern4rust-baseline.json"

[package.validation]
skip = ["paired-test-file"]

[package.system-tests]
skip    = ["paired-test-file"]
exclude = ["**/generated/**"]
```

A package with nothing to say needs no section. `node` and `node-infra` have
none, which is the honest rendering of "these apply everything".

### Precedence is replacement, not merging

**Command line beats package section beats root beats default**, and each level
*replaces* the one below rather than adding to it.

That is not a new principle; it is the one already written down for the command
line against the file:

> Merging would make `--rule header` mean "header plus whatever the file already
> selected", which is the opposite of what naming one rule means everywhere else
> in this tool.

The same sentence holds one level down. A package that says `skip = [...]` states
its whole skip list, and a reader who wants to know what a package skips reads one
list rather than assembling it from two.

### Two tiers of key, separated by what they are about

| root only | per package |
|---|---|
| `baseline`, `offence-threshold` | `rules`, `skip`, `exclude`, `header-file`, `max-files-per-directory`, `max-subfolders-per-directory` |

A baseline is a set of fingerprints for one run and `offence-threshold` is about
how the report ends. Neither is a property of a package, and neither would mean
anything inside a section.

This is enforced by the type rather than by a check: the per-package struct
simply has no `baseline` field, so `deny_unknown_fields` rejects it with the same
message it already gives a misspelled key.

### A section naming no package is an error

`deny_unknown_fields` cannot catch `[package.validaton]`, because the section
name is data rather than a key. It gets the same treatment as an unknown
`--rule` name, for the reason the config file already gives:

> A misspelled `exclude` that silently did nothing would look exactly like an
> exclude that worked.

A section for a package that is not in the workspace is that same failure wearing
a different hat, and it is the more dangerous one: it reads as a rule set being
applied.

### The report becomes per package

This is the part that costs something. Today the report carries one `applied:`
line, one `not applied:` line and one `summary:`. With rule sets that differ by
package, a single `rules_applied=21` is false the moment one package applies
twenty.

So the report gains a block per package, and keeps a roll-up. **The `summary:`
line keeps its shape**, because every gate script in this family parses it with a
regex and a changed summary would break all of them at once — see
[ADR-MachineReadableReport](ADR-MachineReadableReport.md).

**A block each only where they differ.** A roster that says the same thing as
every other is said once, without a package name, which is what every
single-package run and every workspace answering to one rule set prints today.
The reader pays for the workspace they have rather than the one this decision
anticipated: an ordinary run is unchanged, and only a genuinely mixed workspace
costs a block per member.

The summary keeps understating on purpose. `rules_skipped=1` where one of four
packages stood a rule down is true of the run and coarse about the members; the
blocks above it carry the detail, and the number below stays parseable.

The property being protected is the one that makes a subset honest at all: a
stand-down is only acceptable while the report names it. A per-package skip that
vanished into an aggregate number would be a blind spot with a green tick over
it.

## Consequences

**One call, one file.** `cargo stern4rust` at the workspace root judges every
member against its own rules. The four-file arrangement stops being necessary,
though it keeps working — see below.

**The `--manifest-path` trap goes away for the common case.** There is nothing to
select with it, so there is nothing to get half right.

**Relative paths resolve once.** `header-file = "docs/header.txt"` is written
once, from the root, rather than `../docs/header.txt` four times.

**Backwards compatible.** A configuration with no `[package.*]` section behaves
exactly as it does today, which is what every other repository in this family
has. Per-package files beside member manifests keep working too: they are still
the config for a run that names that manifest.

**Two configurations can now disagree.** A root file with sections and a stale
`stern4rust.toml` beside a member are both valid, and only one is read. The
answer is the one this tool always gives — the report says which file it read,
on the `config:` line it already prints.

**It does not make the rules per-file.** The unit is the package, because the
package is what a manifest describes and what `--package` already selects.
Anything finer is what `exclude` is for.

## What this does not solve

**A package whose files disagree with each other.** `system-tests` wants
`paired-test-file` for its `src/`, where the files are ordinary source, and not
for its `tests/`, where they are scenarios. The section skips it for both. The
rule set is per package because the manifest is; a directory-level answer would
need a different unit and is not proposed here.

**Choosing the sections.** Nothing in this decision says which rules a package
should stand down on. That stayed a reading of what actually fired: fourteen
offences, all one shape, against a rule with nothing to match. A section written
from a guess is a blind spot arrived at politely.
