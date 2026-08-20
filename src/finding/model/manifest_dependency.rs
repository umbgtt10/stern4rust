// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// One dependency as a member manifest declares it.
//
// Plain data, read from the TOML rather than from `cargo metadata`, because the
// question is *how it was written* and not what it resolves to. `cargo metadata`
// answers the second and erases the first: a dependency taken from the workspace
// and one spelled out in the member look identical once resolved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestDependency {
    pub manifest: String,
    pub name: String,
    pub section: String,
    pub takes_from_workspace: bool,
}

impl ManifestDependency {
    pub const SECTIONS: [&'static str; 3] =
        ["dependencies", "dev-dependencies", "build-dependencies"];

    pub fn new(manifest: &str, name: &str, section: &str, takes_from_workspace: bool) -> Self {
        Self {
            manifest: manifest.to_string(),
            name: name.to_string(),
            section: section.to_string(),
            takes_from_workspace,
        }
    }
}
