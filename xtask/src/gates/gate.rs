// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::gates::gate_failure::GateFailure;

pub trait Gate {
    fn label(&self) -> String;

    fn run(&self) -> Result<(), GateFailure>;
}
