// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// How a function came to depend on time: by reading a clock, waiting on a timer, racing one, or drawing randomness
// nobody seeded -- the same failure by another door.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeCallKind {
    Clock,
    Wait,
    Race,
    Random,
}

impl TimeCallKind {
    const VERBS: [&'static str; 4] = [
        "reads the clock",
        "waits on a timer",
        "races a timer",
        "draws unseeded randomness",
    ];

    pub fn verb(self) -> &'static str {
        Self::VERBS[self as usize]
    }
}
