// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use serde_json::Error;
use serde_json::Value;
use serde_json::from_str;

const TABLE: &str = "cargo-shear";
const WORKSPACE: &str = "workspace";

// The exceptions a workspace makes to cargo-shear, read from `cargo metadata`.
// A crate a package -- or the workspace -- lists in its cargo-shear `ignored`
// is one cargo-shear cannot see used: linked only for what it provides,
// declared only to switch on a feature, used only by generated code. Each
// carries its reason in the same table's `reasons`, because an exception
// nobody can explain is a dependency nobody checked.
pub struct IgnoreReasons;

impl IgnoreReasons {
    pub fn new() -> Self {
        Self
    }

    // Every exception without a reason, as `<package>: <crate>`, packages in
    // the order cargo lists them and the workspace's last.
    pub fn unexplained(&self, metadata: &str) -> Result<Vec<String>, Error> {
        let document: Value = from_str(metadata)?;
        let packages = document["packages"].as_array().cloned().unwrap_or_default();
        let mut unexplained: Vec<String> = packages
            .iter()
            .flat_map(|package| {
                Self::in_table(
                    package["name"].as_str().unwrap_or_default(),
                    &package["metadata"],
                )
            })
            .collect();
        unexplained.extend(Self::in_table(WORKSPACE, &document["metadata"]));
        Ok(unexplained)
    }

    fn in_table(owner: &str, metadata: &Value) -> Vec<String> {
        let table = &metadata[TABLE];
        table["ignored"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .iter()
            .filter_map(Value::as_str)
            .filter(|ignored| {
                table["reasons"][*ignored]
                    .as_str()
                    .is_none_or(|reason| reason.trim().is_empty())
            })
            .map(|ignored| format!("{owner}: {ignored}"))
            .collect()
    }
}

impl Default for IgnoreReasons {
    fn default() -> Self {
        Self::new()
    }
}
