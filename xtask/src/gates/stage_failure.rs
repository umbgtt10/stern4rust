// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::fmt;

use crate::gates::gate_failure::GateFailure;

// The gate that stopped stage 2, by its label, and why it failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageFailure {
    label: String,
    failure: GateFailure,
}

impl StageFailure {
    pub fn new(label: String, failure: GateFailure) -> Self {
        Self { label, failure }
    }

    pub fn failure(&self) -> &GateFailure {
        &self.failure
    }

    pub fn label(&self) -> &str {
        &self.label
    }
}

impl fmt::Display for StageFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} ({})", self.label, self.failure)
    }
}
