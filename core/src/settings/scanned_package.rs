// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::string::String;
use alloc::string::ToString;

use crate::source_path::SourcePath;

// One package about to be walked, and what its manifest says about it.
//
// The scan loop already runs package by package, but until now it carried only
// a directory: by the time a file was judged there was nothing left saying
// which member it came from. Everything that comes from a manifest rather than
// from a command line was therefore read once for the whole run and applied to
// all of them.
//
// That is what made `spdx-matches-manifest` unable to speak for a workspace.
// See [ADR-ManifestDataIsPerPackage](../../docs/ADRs/ADR-ManifestDataIsPerPackage.md).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScannedPackage {
    name: String,
    root: SourcePath,
    // None where the manifest declares no `license`, which is the one case
    // `spdx-matches-manifest` cannot judge and stands down on. A virtual
    // manifest has no package and so never reaches here at all.
    license: Option<String>,
}

impl ScannedPackage {
    pub fn new(name: &str, root: SourcePath, license: Option<String>) -> Self {
        Self {
            name: name.to_string(),
            root,
            license,
        }
    }

    // The licence every scanned package declares, or None where they do not all
    // declare the same one.
    //
    // This is what the report answers for, and it is deliberately the weaker
    // question. Checking is per package; a run-wide claim can only be made where
    // there is nothing to disagree about, so it understates rather than
    // overstates.
    pub fn agreed_license(packages: &[Self]) -> Option<String> {
        let first = packages.first()?.license.as_ref()?;
        packages
            .iter()
            .all(|package| package.license.as_ref() == Some(first))
            .then(|| first.clone())
    }

    pub fn license(&self) -> Option<&str> {
        self.license.as_deref()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn root(&self) -> &SourcePath {
        &self.root
    }
}
