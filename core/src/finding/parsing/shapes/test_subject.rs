// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestSubject {
    pub name: String,
    pub line: usize,
    pub records: Vec<String>,
    pub exercises_behaviour: bool,
}
