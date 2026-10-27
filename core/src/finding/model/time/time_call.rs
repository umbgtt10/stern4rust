// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::string::String;

use crate::finding::model::time::time_call_kind::TimeCallKind;

// One call through which a function depends on time: its kind, the path or method as the source wrote it, and where
// it stands, so the first of several can be named.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeCall {
    kind: TimeCallKind,
    written: String,
    line: usize,
    column: usize,
}

impl TimeCall {
    pub fn new(kind: TimeCallKind, written: &str, line: usize, column: usize) -> Self {
        Self {
            kind,
            written: String::from(written),
            line,
            column,
        }
    }

    pub fn column(&self) -> usize {
        self.column
    }

    pub fn kind(&self) -> TimeCallKind {
        self.kind
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn written(&self) -> &str {
        &self.written
    }
}
