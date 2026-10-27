// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::support::in_memory_source_tree::NotHeld;
use crate::support::in_memory_source_tree::path_of;
use stern4rust::settings::package_description::PackageDescription;
use stern4rust::settings::workspace_description::WorkspaceDescription;
use stern4rust::settings::workspace_read_failure::WorkspaceReadFailure;
use stern4rust::traits::workspace_reader::WorkspaceReader;

// The workspace every core test resolves against, handed over whole rather than
// read from cargo.
pub struct InMemoryWorkspaceReader {
    workspace: Option<WorkspaceDescription>,
}

impl InMemoryWorkspaceReader {
    pub fn new(workspace: WorkspaceDescription) -> Self {
        Self {
            workspace: Some(workspace),
        }
    }

    pub fn failing() -> Self {
        Self { workspace: None }
    }
}

impl WorkspaceReader for InMemoryWorkspaceReader {
    fn describe(&self) -> Result<WorkspaceDescription, WorkspaceReadFailure> {
        self.workspace.clone().ok_or_else(|| WorkspaceReadFailure {
            cause: Box::new(NotHeld),
        })
    }
}

// A workspace of these packages, each at `<root>/<name>`, declaring nothing.
pub fn workspace_of(root: &str, names: &[&str]) -> WorkspaceDescription {
    WorkspaceDescription {
        root: path_of(root),
        declares_workspace: names.len() > 1,
        packages: names
            .iter()
            .map(|name| PackageDescription {
                name: (*name).to_string(),
                manifest_path: path_of(&format!("{root}/{name}/Cargo.toml")),
                license: Some(String::from("MIT")),
                declared: Vec::new(),
            })
            .collect(),
    }
}
