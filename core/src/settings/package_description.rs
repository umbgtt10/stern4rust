// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::finding::model::manifest_dependency::ManifestDependency;
use crate::source_path::SourcePath;

// One member as cargo describes it, with every dependency its manifest
// declares, as written -- empty where the manifest could not be read.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PackageDescription {
    pub name: String,
    pub manifest_path: SourcePath,
    pub license: Option<String>,
    pub declared: Vec<ManifestDependency>,
}
