# R034-ADR-InjectedTimeRule

- **Status:** Accepted
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

- **in an implementation of a port** -- a trait the package declares, or one a
  dependency declares; the adapter a test replaces with one it drives -- or
- **in a binary** -- `src/main.rs`, `src/bin/`, and the modules only they
  declare -- the composition root, where the real clock is chosen.

- **A port**, not any trait: `impl Display` reading the clock is not a seam a
  test can swap. A port is a trait the package declares -- read across its
  `src/` files as [R026](R026-ADR-ImplementedInOneFileRule.md)'s `TypeHomes`
  reads its types, and matched by an impl's last segment, so
  `impl crate::ports::Clock` is `Clock` -- or a trait from a dependency, its
  origin read through the file's imports: a family's core crate declares the
  ports its runtimes implement, as `etheram-core` declares `TimerOutput`. The
  standard library's traits are not ports -- `std`, `core`, `alloc`, and a bare
  name nobody imported that the prelude holds, as `Iterator` or `Drop`. An
  origin the rule cannot tell counts as a port, so it stays quiet rather than
  accuse an adapter. A port's own default bodies are left alone too: a test's
  implementation overrides them.
- **Production code** is a free function, an inherent method or a standard
  trait's method, through inline modules; a test function and anything under
  `#[cfg(test)]` is test code, and R033's. Randomness, which R033 also lists, is
  not this rule's -- see below.
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
  the open what an off-by-default rule would leave unsaid. As built first, a
  section's `skip = []` fell back to the root's list -- an absent key and an
  empty one read alike -- so the example above re-enabled nothing; a section's
  stated `skip`, empty included, is now its whole list, as
  [ADR-PerPackageConfiguration](ADR-PerPackageConfiguration.md) says.
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

**Re-run with this tool on 2026-10-22: 173 functions in 83 files**, counted
per function as the rule reports them -- 130 reading a clock first, 36 waiting
on a timer, 7 racing one. The heuristic's 428 calls counted every call, its 26
in binaries among them, which the rule leaves alone.

| repository | functions | files | clock | wait | race |
|---|---:|---:|---:|---:|---:|
| `etheram-tools` | 42 | 21 | 32 | 7 | 3 |
| `etheram-ibft` | 39 | 10 | 28 | 11 | 0 |
| `etheram-ibft-embassy`, three workspaces | 40 | 21 | 32 | 8 | 0 |
| `etheram-embassy` | 20 | 15 | 13 | 7 | 0 |
| `etheram-raft-embassy` | 17 | 6 | 16 | 1 | 0 |
| `etheram-raft` | 13 | 8 | 7 | 2 | 4 |
| `slotgate` | 2 | 2 | 2 | 0 | 0 |
| `etheram-core`, the analysis workspace, `braintax4rust`, `crap4rust`, `dry4rust`, `grip4rust`, `iceberg4rust`, `stern4rust`, `twin4rust` | **0** | | | | |

- **The cores comply, measured.** Every one of `etheram-ibft`'s 39 and
  `etheram-raft`'s 13 is in its `system-tests` package, which a runtime
  workspace's root skip covers; `node` and every other member have none.
- **Two of the heuristic's findings were not time.** `grip4rust`'s "one
  duration" is a file's modification time turned into seconds,
  `mtime.duration_since(UNIX_EPOCH)`, which reads no clock -- R033's catalogue
  had counted `duration_since` as a reading, and no longer does. And
  `slotgate`'s `JobRunner::new` was charged with racing a timer for building
  `Self { timeout, .. }`: a bare name not called is a value, and both rules now
  read it as one.
- **`slotgate` has two**, the job runner's `run` -- its stopwatch, its
  `timeout` race -- and the job order's `seed_from_clock`, which seeds a shuffle
  from the system clock.

## Rejected alternatives

**Off by default, behind a new key.** Every rule here is on by default, and the
configuration already says where one applies. A key naming the time-free
packages would be a second way to state what `skip` states. Rejected.

**Time-free packages: no clock anywhere, adapters included.** Stricter, and true
of today's cores, but it would forbid a core from shipping its own system-clock
adapter beside the port it declares. A core that takes time through a port stays
deterministic under any test that injects one. Rejected for the port.

**Any trait impl.** `impl Display for Elapsed` reading the clock is behind a
trait and behind no seam. Rejected for ports.

**Only the traits the package declares**, as first built. It accused the
adapters the architecture asks for: `etheram-raft`'s `system-tests` implements
`etheram-core`'s `TimerOutput` with a desktop timer, and `schedule` reading
`Instant::now()` there is exactly where the real clock belongs -- the port is
upstream, so the package declares nothing. Rejected on 2026-10-22, for ports a
dependency declares as well. The re-run frees three functions: that `schedule`,
`etheram-embassy`'s host time driver -- `now` in its `embassy_time_driver::Driver`
impl, the port embassy's executor takes time through -- and `etheram-tools`'
`ClusterLensApp::update`, an `eframe::App` callback. The last is the price: a
framework's trait counts as a port too, and a GUI frame reading the clock goes
unreported. With `slotgate`'s two since moved behind ports of its own, 168
functions remain.

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

`tests/rules/source/injected_time_rule_tests.rs` -- seventeen tests: a free
function reading the clock reported at its name, its description and
correction pinned word for word; a race in an inherent method; a standard
trait's impl reported; one offence for a function with two calls, at the
first, and the clock named where randomness comes first; the adapter of a port
the package declares, the adapter of a port a dependency declares, a binary's
files -- `src/main.rs`, a module only it
declares, `src/bin/` --, a package without a library, test code in `src/` and
`tests/`, unseeded randomness, a string spelling a clock and an unparseable file
left alone; `check` answering nothing per file; the explanation breaking the
rule and its mend not; `is_configured`, the name and the requirement.

`tests/finding/parsing/time/production_functions_tests.rs` (seven) pins what is
judged: free functions and inherent methods, a standard trait's methods, an
inline module's functions; and not a test, a `#[cfg(test)]` module, an impl of
a port -- the package's, written bare or as a path, or a dependency's -- or a
port's default bodies. `tests/finding/parsing/time/port_traits_tests.rs`
(seven) pins what a port is: a dependency's trait by its full path and through
an import, a trait the package declares; and not a prelude trait, a standard
one through an import or a module, or a `crate::` path to a trait the package
does not declare.
`TimeCallFinder::calls_in` has a test of its own, beside the catalogue R033
shares.

`tests/runner_tests.rs`' `run_reporting_over_a_workspace_skipping_injected_time_judges_the_member_that_takes_it_back`
builds the adoption above -- the root skipping the rule, one member stating
`skip = []` -- and finds that member's clock read and not the other's.
`tests/settings/config_file_tests.rs` pins a section's stated empty `skip`
apart from one saying nothing. The four rule-name lists in
`tests/rule_registry_tests.rs` include `injected-time`, and the `--rules`
listing holds thirty-four. Stage 2 runs this tool, built from the checkout,
against this crate with every rule applied and zero offences.

## Related

- [R033-ADR-TimelessTestsRule](R033-ADR-TimelessTestsRule.md) -- the same calls
  in tests, and the census this one shares.
- [R026-ADR-ImplementedInOneFileRule](R026-ADR-ImplementedInOneFileRule.md) --
  the package-wide reading of declarations this one reuses.
- [R030-ADR-TypedErrorsRule](R030-ADR-TypedErrorsRule.md) -- the library target
  and the binary's files, judged and left alone the same way.
- [ADR-PerPackageConfiguration](ADR-PerPackageConfiguration.md) -- the
  replacement that lets a package re-enable a rule its root skips.
