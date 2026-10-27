// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use serde_json::Error;
use serde_json::Value;
use serde_json::from_str;

// The crates one package's manifest names, read from `cargo metadata --no-deps`:
// every section -- normal, dev and build -- since a technology-free package's
// tests are held to the same line as its code. A crate declared in two
// sections is named once, where it first appears.
pub struct PackageDependencies;

impl PackageDependencies {
    pub fn new() -> Self {
        Self
    }

    // `None` when the package is not in the document: a renamed package must
    // fail the gate rather than pass it by having no dependencies to judge.
    pub fn of(&self, metadata: &str, package: &str) -> Result<Option<Vec<String>>, Error> {
        let document: Value = from_str(metadata)?;
        Ok(document["packages"]
            .as_array()
            .and_then(|packages| {
                packages
                    .iter()
                    .find(|candidate| candidate["name"] == package)
            })
            .map(|found| {
                let mut names: Vec<String> = Vec::new();
                for name in found["dependencies"]
                    .as_array()
                    .cloned()
                    .unwrap_or_default()
                    .iter()
                    .filter_map(|dependency| dependency["name"].as_str().map(String::from))
                {
                    if !names.contains(&name) {
                        names.push(name);
                    }
                }
                names
            }))
    }
}

impl Default for PackageDependencies {
    fn default() -> Self {
        Self::new()
    }
}
