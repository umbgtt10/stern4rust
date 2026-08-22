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

    // The subset declared in one manifest.
    //
    // `check_workspace` runs once per package, so handing every package the
    // whole workspace's declarations stated each finding once per member --
    // twenty real findings became 580 in a twenty-nine member workspace, every
    // copy identical, the count simply tracking the member count. A finding
    // about `alpha/Cargo.toml` belongs to `alpha`, and the loop walking the
    // packages already knows which one it is in.
    //
    // `None` stays `None`: that is how a package outside a workspace tells the
    // rule to stand down, and filtering must not turn it into an empty list.
    pub fn in_manifest(all: &Option<Vec<Self>>, manifest: &str) -> Option<Vec<Self>> {
        Some(
            all.as_ref()?
                .iter()
                .filter(|dependency| dependency.manifest == manifest)
                .cloned()
                .collect(),
        )
    }

    pub fn new(manifest: &str, name: &str, section: &str, takes_from_workspace: bool) -> Self {
        Self {
            manifest: manifest.to_string(),
            name: name.to_string(),
            section: section.to_string(),
            takes_from_workspace,
        }
    }
}
