// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// What one scoring tool reported of core: its own release, and the total it
// was asked for.
#[derive(Debug, Clone, PartialEq)]
pub struct ToolScore {
    pub version: String,
    pub total: f64,
}
