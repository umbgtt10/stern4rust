# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Publishable crate skeleton. Packages as `cargo-stern4rust` with a `stern4rust`
  library, so cargo resolves `cargo stern4rust`, matching `cargo-crap4rust`,
  `cargo-twin4rust` and `cargo-iceberg4rust`.
- `Args::without_cargo_subcommand`, the argv fixup every cargo subcommand needs:
  cargo runs `cargo stern4rust ...` as `cargo-stern4rust stern4rust ...`, so the
  name arrives twice, while running the binary directly does not repeat it. The
  strip is conditional and positional, so a package that happens to be named
  `stern4rust` survives.
- `--manifest-path` and `--package`, the two flags the whole family shares.
- Test harness: `autotests = false` with a single `[[test]] all_tests`, one test
  file per source file.

### Not yet implemented

- The rules themselves. A run reports that nothing is implemented and exits 0.
  The first two are specified in README.md: AAA structure in tests, and at most
  one struct with an impl block per file. The set is open by design.
