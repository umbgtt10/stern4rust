# R034-ADR-InjectedTimeRule

- **Status:** Proposed
- **Date:** 2026-10-21

## Context

[R033](R033-ADR-TimelessTestsRule.md) keeps a test from reading a clock or
waiting on a timer. It cannot keep the code under test from doing so, and that
is where a timeout lives: a job runner that gives up after thirty seconds, a
retry that backs off, a protocol that changes round when a timer fires. A test
of that code either waits for real time -- which R033 forbids -- or drives a
clock the code was handed. Only the second is deterministic, and it is only
possible if the code takes its time from something a test can replace.

The family's protocol cores already work that way. Their `node` crates read no
clock: time arrives as a timer event, and what to do about it is decided on
data, so the same inputs give the same outputs on any machine. That is the
property a deterministic simulation of a core, or a model checker run over it,
depends on -- and nothing keeps it from being lost by the next change that
reaches for `Instant::now()` because it is the shortest way to a timeout.

## Decision

**Time reaches production code through a port.** In `src/`, outside
`#[cfg(test)]` modules, a function reading a clock, waiting or racing a timer --
the calls [R033](R033-ADR-TimelessTestsRule.md) lists -- is an offence unless it
sits:

- **in an implementation of a trait the package declares** -- the adapter a
  test replaces with one it drives -- or
- **in a binary** -- `src/main.rs`, `src/bin/`, and the modules only they
  declare -- the composition root, where the real clock is chosen.

- **A trait the package declares**, not any trait: `impl Display` reading the
  clock is not a seam a test can swap. The traits are the package's own, read
  across its files as [R026](R026-ADR-ImplementedInOneFileRule.md)'s
  `TypeHomes` reads its types.
- **Names are resolved through the file's `use` items**, and string literals are
  not code, as in R033.
- **One offence per function**, at its name, naming the first call: *"`retry`
  reads the clock with `Instant::now`; take time through a port the package
  declares, so a test can drive it"*.
- **The correction names the shape:** a trait for the clock or timer the code
  needs, the real one implementing it in an adapter, the function taking it
  injected; or, for a timer that only marks a deadline, an event the caller
  feeds in, as the protocol cores' timers are.
- **On by default, adopted as a guard on a core.** A workspace whose libraries
  are runtime -- firmware tasks, hardware harnesses, transport and storage
  adapters, where real time is the job -- skips the rule at its root, and the
  packages that are meant to be time-free state their own `skip` list, which
  replaces the root's ([ADR-PerPackageConfiguration](ADR-PerPackageConfiguration.md)):

  ```toml
  skip = ["injected-time"]

  [package.node]
  skip = []
  ```

  No new key: a rule nobody turns on protects nothing, and a root skip says in
  the open what an off-by-default rule would leave unsaid.
- **Exempt:** `tests/`, which is R033's.

## Forcing constraints / Evidence

The census of [R033](R033-ADR-TimelessTestsRule.md), the same heuristic scanner
and caveat, over `src/` outside test modules: **428 calls in 129 files** -- 263 in
libraries, 139 in harness packages, 26 in binaries. By where they sit: **16 in a
trait impl** -- an upper bound on what this rule already admits, the scanner
not telling the package's own traits from foreign ones --, 309 in inherent
methods, 103 in free functions.

| repository | library | harness | binary |
|---|---:|---:|---:|
| `etheram-ibft-embassy` | 93 | 0 | 6 |
| `etheram-embassy` | 64 | 14 | 17 |
| `etheram-tools` | 60 | 0 | 1 |
| `etheram-raft-embassy` | 40 | 0 | 0 |
| `etheram-ibft` | 0 | 104 | 2, in its `xtask` |
| `etheram-raft` | 0 | 20 | 0 |
| `slotgate` | 5 | 0 | 0 |
| `grip4rust`, `braintax4rust` | 1, 0 | 0, 1 | 0 |
| `etheram-core`, the analysis workspace, `crap4rust`, `dry4rust`, `iceberg4rust`, `stern4rust`, `twin4rust` | **0** | **0** | **0** |

- **The cores comply.** `etheram-core`, and the `node` crates of `etheram-ibft`
  and `etheram-raft`, have no call at all: everything in those repositories is
  in a harness or an `xtask`.
- **The rest is runtime.** The embassy repositories read time in their firmware
  apps, scenarios, hardware harnesses and their UDP and flash adapters, largely
  in `embassy` tasks, which an executor requires to be free functions;
  `etheram-tools` in `embedded-gate`. Injecting a clock there would cost `no_std`
  statics and executor tasks a layer that buys little: those packages are tested
  on the rig.
- **The tools are nearly clean:** `grip4rust` measures one duration, and
  `slotgate`'s job runner, the one tool whose subject is time, has five.

## Rejected alternatives

**Off by default, behind a new key.** Every rule here is on by default, and the
configuration already says where one applies. A key naming the time-free
packages would be a second way to state what `skip` states. Rejected.

**Time-free packages: no clock anywhere, adapters included.** Stricter, and true
of today's cores, but it would forbid a core from shipping its own system-clock
adapter beside the port it declares. A core that takes time through a port stays
deterministic under any test that injects one. Rejected for the port.

**Any trait impl.** `impl Display for Elapsed` reading the clock is behind a
trait and behind no seam. Rejected for traits the package declares.

**Paths as the boundary** -- `infra/`, `adapters/`, `runtime/`. A convention of
one family; this tool's corrections rearrange paths, and the configuration
already names packages. Rejected.

## Consequences

**A core that adopts it stays time-free.** The next change that wants a timeout
in a protocol core has to declare a port for it or carry the deadline in as an
event, and the property a deterministic simulation needs is held by a gate
rather than by habit.

**R033 becomes satisfiable for timed code.** A test of a timeout injects a clock
it drives, instead of waiting for one.

**Runtime workspaces state that they are.** The embassy repositories and
`etheram-tools` skip it at their roots in one line, and re-enable it for any core
they hold; the tools fix their six or skip it where time is the subject.

### What this rule does not catch

- **Time inside a dependency** -- a library call that reads the clock or times
  out on its own.
- **Time a macro generates**, invisible to `syn`.
- **Randomness**, the other input a deterministic core must take from outside --
  a separate question, and a separate port.

## Enforcement

To be written test-first when the rule is built: each of R033's calls reported in
an inherent method and in a free function, through a full path and through an
import; the same call in an implementation of a trait the package declares left
alone, and in an implementation of a foreign trait reported; `src/main.rs`,
`src/bin/` and the modules only they declare left alone; `#[cfg(test)]` modules
and `tests/` left alone; a string literal left alone; one offence per function
naming the first call; the correction; a root skip with a package re-enabling
it; the unparseable file; `check_workspace`, `is_configured`, the name and the
requirement; `tests/rule_registry_tests.rs` updated.

## Related

- [R033-ADR-TimelessTestsRule](R033-ADR-TimelessTestsRule.md) -- the same calls
  in tests, and the census this one shares.
- [R026-ADR-ImplementedInOneFileRule](R026-ADR-ImplementedInOneFileRule.md) --
  the package-wide reading of declarations this one reuses.
- [R030-ADR-TypedErrorsRule](R030-ADR-TypedErrorsRule.md) -- the library target
  and the binary's files, judged and left alone the same way.
- [ADR-PerPackageConfiguration](ADR-PerPackageConfiguration.md) -- the
  replacement that lets a package re-enable a rule its root skips.
