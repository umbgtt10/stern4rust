// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

// What keeps a package technology-free: the only crates its manifest may name,
// in any section; the inner attributes its crate root must carry -- `no_std`
// hands the source tree to the compiler --; and, in the directories scanned,
// the standard-library modules, printing macros and file-system methods none of
// its files may reach, which matters for its tests, where `std` is linked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundaryRule {
    pub package: String,
    pub allowed_crates: Vec<String>,
    pub crate_root: PathBuf,
    pub required_attributes: Vec<String>,
    pub source_dirs: Vec<PathBuf>,
    pub forbidden_modules: Vec<String>,
    pub forbidden_macros: Vec<String>,
    pub forbidden_methods: Vec<String>,
}
