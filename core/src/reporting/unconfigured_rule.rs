// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A rule that was selected but could not run, and what it was waiting for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnconfiguredRule {
    pub name: &'static str,
    pub requirement: &'static str,
}

impl UnconfiguredRule {
    pub fn new(name: &'static str, requirement: &'static str) -> Self {
        Self { name, requirement }
    }
}
