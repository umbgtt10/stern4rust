// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::settings::workspace_description::WorkspaceDescription;
use crate::settings::workspace_read_failure::WorkspaceReadFailure;

// The workspace a run judges: its root, its members and what each member's
// manifest declares.
pub trait WorkspaceReader {
    fn describe(&self) -> Result<WorkspaceDescription, WorkspaceReadFailure>;
}
