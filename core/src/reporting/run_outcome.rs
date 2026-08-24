// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// What a run concluded, kept separate from how the process reports it.
//
// Runner returns this rather than calling exit, so the whole run is reachable
// from a test. Only main turns it into an exit code, which is also the only
// place that needs to know 2 means "rule broken" rather than "tool failed".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunOutcome {
    Clean,
    RulesBroken,
}

impl RunOutcome {
    pub fn of(offence_count: usize) -> Self {
        if offence_count == 0 {
            Self::Clean
        } else {
            Self::RulesBroken
        }
    }

    pub fn exit_code(self) -> i32 {
        match self {
            Self::Clean => 0,
            Self::RulesBroken => 2,
        }
    }
}
