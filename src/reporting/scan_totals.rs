// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// What a scan did, as opposed to what it found.
//
// The offences are the verdict; these are the work behind it. They travelled to
// the report as separate arguments until the per-package rosters arrived and
// took the count past the point where a reader could tell one from another.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScanTotals {
    pub files_scanned: usize,
    pub fixed: usize,
}

impl ScanTotals {
    pub fn new(files_scanned: usize, fixed: usize) -> Self {
        Self {
            files_scanned,
            fixed,
        }
    }
}
