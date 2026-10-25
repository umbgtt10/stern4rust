// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The phase or phases one marker comment names. Plain data: a marker is read,
// never asked anything.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkerPhase {
    Arrange,
    Act,
    Assert,
}
