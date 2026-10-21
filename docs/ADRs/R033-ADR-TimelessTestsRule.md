# R033-ADR-TimelessTestsRule

- **Status:** Proposed
- **Date:** 2026-10-21

## Context

A test whose verdict depends on how fast the machine is measures the machine.
`slotgate`'s `run_all_bounds_wall_clock_time_by_max_parallel` ran four
half-second sleeps at `max_parallel = 4` and asserted they finished within five
seconds. On a Windows runner starting processes slowly they took 8.7, and the
test failed a change that had only reordered functions (`slotgate` `f9cc8fb`).
Its replacement asks the question directly -- four jobs that can only finish
together, each leaving a mark and waiting for the others -- and time bounds only
its failure, never its pass (`965b36a`).

Time reaches a test in a handful of shapes, each a way for the same test to pass
on one machine and fail on another:

- **a stopwatch** -- `Instant::now()`, then an assertion on `elapsed()`;
- **a sleep** standing in for "until it has happened";
- **a timeout proving that something blocks** -- "it did not finish within 200
  ms";
- **a clock as a source of uniqueness** -- a temp directory named from
  `SystemTime::now()`;
- **randomness nobody seeded**, the same failure by another door.

Whether a test is deterministic is not something a syntax tree can answer: a
test that reads no clock can still wait on one inside the code it calls. What a
syntax tree can answer is whether the test itself reads a clock or waits on a
timer -- and that is the part a test's author controls.

## Decision

**A test reads no clock and waits on no timer.** In test code -- `tests/`,
`benches/`, and `#[cfg(test)]` modules in `src/` -- a function calling any of
these is an offence:

- **a clock:** `Instant::now`, `SystemTime::now`, `chrono`'s `Utc::now` and
  `Local::now`, `time`'s `OffsetDateTime::now_utc` and `now_local`, and the
  `.elapsed()` and `.duration_since()` read off them;
- **a wait:** `std::thread::sleep`, `tokio::time::sleep` and `sleep_until`,
  `async_std::task::sleep`, `embassy_time::Timer::after` and `at`;
- **a race:** `tokio::time::timeout` and `timeout_at`;
- **unseeded randomness:** `rand::thread_rng`, `rand::rng`, `rand::random`,
  `OsRng`.

- **Names are resolved through the file's `use` items**, so a bare `sleep(...)`
  imported from `tokio::time` is Tokio's, and a helper of the test's own called
  `sleep` is not.
- **String literals are not code.** The analysis tools' tests hand
  `Instant::now()` to the tool under test as text; that is data.
- **Allowed:** a test on Tokio's paused clock --
  `#[tokio::test(start_paused = true)]` -- may use `tokio::time`'s `sleep`,
  `timeout`, `advance` and `Instant`: its time is virtual, and moves only when
  the test is idle or says so. A `Duration` is a value and never an offence:
  configuring a timeout that ends a hang, as the rendezvous test does, waits on
  nothing. A seeded generator, `StdRng::seed_from_u64`, is deterministic.
- **One offence per function**, test or helper, at its name, naming the first
  call: *"`run_all_bounds_wall_clock_time_by_max_parallel` reads the clock with
  `Instant::now`; a test's verdict must not depend on how fast the machine
  is"*.
- **The correction names the ways out**, most preferred first: wait for the
  event itself -- a channel, a barrier, a rendezvous of the parties --; run on
  Tokio's paused clock; inject a clock into the code under test and drive it
  from the test; seed the generator; name a temp directory after the test, or
  let a temp-directory crate choose.
- **A harness skips the rule** in `stern4rust.toml` -- a system-test suite
  running real clusters over real sockets, a hardware validation package --
  where real time is the subject rather than an accident, as harnesses skip
  [R030](R030-ADR-TypedErrorsRule.md).
- **Adoption is by baseline** ([ADR-Baselines](ADR-Baselines.md)).

## Forcing constraints / Evidence

A census on 2026-10-21 over the sixteen repositories of
[R025](R025-ADR-PublicOrPrivateRule.md), with a heuristic scanner -- comments and
string literals stripped, every call attributed to the function it sits in, a
bare `sleep` or `timeout` resolved through the file's imports -- to be re-run
with this tool's finders before the rule ships. A first pass with `grep` counted
the analysis tools' fixture strings as calls; stripping literals took
`braintax4rust` and `grip4rust` to zero, which is why literals are not code here.

**127 test functions and 44 test helpers** read a clock or wait on a timer, in
**261 calls**: 151 in libraries' tests, 110 in harness packages.

| repository | test functions | helpers | calls |
|---|---:|---:|---:|
| `etheram-tools` | 52 | 12 | 83 |
| `etheram-ibft` | 35 | 15 | 90, all in harness packages |
| `etheram-embassy` | 13 | 6 | 25, 8 in harness packages |
| `etheram-ibft-embassy` | 12 | 3 | 24 |
| `etheram-raft-embassy` | 6 | 1 | 11 |
| `slotgate` | 6 | 0 | 9 |
| `iceberg4rust` | 2 | 1 | 6 |
| `twin4rust` | 0 | 2 | 4 |
| `etheram-raft` | 1 | 4 | 9, all in harness packages |
| `etheram-core`, the analysis workspace, `braintax4rust`, `crap4rust`, `dry4rust`, `grip4rust`, `stern4rust` | | | **0** |

**None runs on Tokio's paused clock.** Read one by one, the shapes above are all
there: `slotgate`'s `job_runner_tests` asserts a timed-out job returns within ten
seconds, its `slot_pool_tests` prove that `acquire` blocks by waiting 200 ms for
it, `etheram-tools` sleeps while hardware settles, and `iceberg4rust` and
`twin4rust` name temp directories from `SystemTime::now()`.

## Rejected alternatives

**"Tests are deterministic" as the rule.** It cannot be checked from syntax, and
a rule promising it would pass every test that waits on a clock through the code
it calls. Rejected for a rule that says what it checks.

**Flagging every `Duration`.** A `Duration` is a value. A test that configures a
two-minute timeout to end a hang waits on nothing while it passes. Rejected.

**Exempting a timeout that only guards against a hang.** A timeout whose `Err`
fails the test bounds only the failure; one whose `Err` is the expected result
-- "it blocks" -- makes the pass wait on the machine. Telling them apart means
reading which result the test asserts. Deferred, not rejected: the census of
guards against assertions has not been taken.

**Detecting flakiness by running the tests** -- twice, under load, on a slowed
clock. A different tool's job; this one reads source.

## Consequences

**A test that cannot wait on the machine cannot fail because of it.** Tests
synchronise on events, run on a paused clock, or drive an injected one, and the
rendezvous in `slotgate`'s `executor_tests` is the shape to copy where the
parties are real processes.

**The cost lands where tests wait on real processes and hardware** --
`etheram-tools`' 52 test functions, the protocol repositories' system tests --
and harnesses that exist to exercise real time skip the rule by name.

**It pushes production code towards injected time.** A test that may not read a
clock can only test a timeout through a clock it controls, so the code under
test has to accept one. That is the companion below.

### What this rule does not catch

- **Time read inside the code a test calls** -- the companion's question.
- **Real concurrency:** thread scheduling, process start-up, file-system order.
- **Other nondeterminism:** `HashMap` iteration order, fixed temp paths and
  ports shared between runs, the network.
- **Time handed to a shell** -- `sleep 0.5` in a job's script -- and time a macro
  generates.

## Assessed: a companion for `src/`

What the census says a rule about production code could and could not do --
proposed since as [R034](R034-ADR-InjectedTimeRule.md), in the shape this
section arrives at.

**What it would ask.** Time enters production code through a port: a clock
read, a sleep or a timeout sits only in an implementation of a trait the crate
declares -- the adapter a test replaces -- or in a binary, the composition root.
Detectable as this rule's calls are, plus the enclosing impl, which
[R026](R026-ADR-ImplementedInOneFileRule.md)'s `TypeHomes` already reads.

**What the census found** -- the same scanner, over `src/` outside test modules:
**428 calls** in 129 files: 263 in libraries, 139 in harness packages, 26 in
binaries.

- **The protocol libraries already comply.** `etheram-core` has none, and every
  call in `etheram-ibft` and `etheram-raft` is in a harness or its `xtask`: their
  `node` crates
  take time in as timer events and decide on it as data, the functional core the
  family's design aims at.
- **The rest is runtime, where time is the job.** `etheram-embassy` (95),
  `etheram-ibft-embassy` (99) and `etheram-raft-embassy` (40) read it in their
  firmware apps, scenarios, hardware harnesses and infrastructure -- UDP
  transport, flash -- and `etheram-tools` (61) almost all in `embedded-gate`.
  The tools have one (`grip4rust`) and five (`slotgate`'s job runner).
- **Almost none of it is behind a port:** 16 calls sit in a trait impl, 309 in
  inherent methods and 103 in free functions -- many of them `embassy` tasks,
  which an executor requires free.

**What follows.** On by default, it would report some 250 calls in libraries
whose subject is real time, and its correction -- inject a clock -- would cost
`no_std` firmware statics and executor tasks a layer that buys little: their
tests are the rig. What it would protect, it protects cheaply where it holds
today. So the shape that fits is **opt-in, as a guard on a core**: the rule
skipped at the root of a workspace and applied by the packages that declare
themselves time-free -- a package section stating its own `skip` list replaces
the root's ([ADR-PerPackageConfiguration](ADR-PerPackageConfiguration.md)), so
this needs no new configuration. `node` and `types` in the protocol
repositories, and the cores the family extracts from them, would adopt it with
no offence and keep the property from then on -- the determinism a model checker
of a pure core, or a deterministic simulation, depends on.

## Enforcement

To be written test-first when the rule is built: each clock, wait, race and
generator reported in a test function and in a helper, through a full path and
through an import; a local function of the same name left alone; a string
literal left alone; a `Duration` left alone; a paused-clock test allowed
`tokio::time` and nothing else; a seeded generator left alone; `#[cfg(test)]`
modules judged and the rest of `src/` not; one offence per function naming the
first call; the correction; a package skipping it; the unparseable file;
`check_workspace`, `is_configured`, the name and the requirement;
`tests/rule_registry_tests.rs` updated.

## Related

- [R030-ADR-TypedErrorsRule](R030-ADR-TypedErrorsRule.md) -- how a harness skips
  a rule by name.
- [R026-ADR-ImplementedInOneFileRule](R026-ADR-ImplementedInOneFileRule.md) --
  the impl reading the companion would reuse.
- [ADR-PerPackageConfiguration](ADR-PerPackageConfiguration.md) -- why an opt-in
  rule needs no new key.
- [R034-ADR-InjectedTimeRule](R034-ADR-InjectedTimeRule.md) -- the companion for
  `src/`: time reaches production code through a port.
