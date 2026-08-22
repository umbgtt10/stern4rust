// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! Holds a Rust workspace to its house coding rules and names every file that
//! breaks one.
//!
//! **This library is not a public API. Depend on the binary,
//! `cargo stern4rust`.**
//!
//! Module paths, type names and signatures may move in any release, including a
//! patch, and no change to them is treated as breaking. Everything here is
//! `pub` for one reason: this repository forbids unit tests, so every test lives
//! in `tests/` and can only reach what is public. The surface is exactly as wide
//! as the test suite needs, and it is reshaped whenever the rules require it --
//! `directory-file-count` has already forced two restructures that moved public
//! paths.
//!
//! What *is* promised is the binary: its exit codes (`0` clean, `1` could not
//! run, `2` rule broken) and the shape of its `--format json` report. See
//! `docs/ADRs/ADR-LibrarySurfaceIsNotAnApi.md`.

pub mod adoption;
pub mod finding;
pub mod reporting;
pub mod rule;
pub mod rule_registry;
pub mod rules;
pub mod runner;
pub mod settings;
pub mod source_file;
pub mod source_reader;
pub mod source_walker;
pub mod test_file_rewriter;
