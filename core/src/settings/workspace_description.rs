// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::settings::package_description::PackageDescription;
use crate::source_path::SourcePath;

// The workspace as cargo describes it: the directory every member's manifest
// is named relative to, whether the root manifest declares a `[workspace]`, and
// every member.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorkspaceDescription {
    pub root: SourcePath,
    pub declares_workspace: bool,
    pub packages: Vec<PackageDescription>,
}
