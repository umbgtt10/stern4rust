// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use toml::Table;
use toml::Value;

use crate::finding::model::manifest_dependency::ManifestDependency;

// Every dependency one manifest declares, as it is written: the three plain
// tables, then each of them under `[target.<cfg>]`, and for every entry what it
// asks of the crate it names.
//
// Read from the TOML rather than from `cargo metadata`, because the manifest
// rules ask how a dependency was written, and resolution erases that. A bare
// string is a version requirement; a table may take from the workspace, state a
// version, list features, turn default features off -- spelled
// `default-features`, or `default_features` as older manifests do -- and be
// optional.
pub struct DependencyTables;

impl DependencyTables {
    pub fn declared(manifest: &Table, shown: &str) -> Vec<ManifestDependency> {
        let targeted = manifest
            .get("target")
            .and_then(Value::as_table)
            .into_iter()
            .flatten()
            .filter_map(|(target, tables)| {
                tables
                    .as_table()
                    .map(|tables| Self::in_tables(tables, shown, Some(target.as_str())))
            })
            .flatten();
        Self::in_tables(manifest, shown, None)
            .into_iter()
            .chain(targeted)
            .collect()
    }

    fn entry(
        shown: &str,
        section: &str,
        target: Option<&str>,
        name: &str,
        value: &Value,
    ) -> ManifestDependency {
        ManifestDependency::new(
            shown,
            name,
            section,
            Self::flag(value, "workspace").unwrap_or(false),
        )
        .with_target(target)
        .with_version(
            value
                .as_str()
                .or_else(|| value.get("version").and_then(Value::as_str)),
        )
        .with_features(Self::features(value))
        .with_default_features(
            Self::flag(value, "default-features")
                .or_else(|| Self::flag(value, "default_features"))
                .unwrap_or(true),
        )
        .with_optional(Self::flag(value, "optional").unwrap_or(false))
    }

    fn features(value: &Value) -> Vec<String> {
        value
            .get("features")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect()
    }

    fn flag(value: &Value, key: &str) -> Option<bool> {
        value.get(key).and_then(Value::as_bool)
    }

    fn in_tables(tables: &Table, shown: &str, target: Option<&str>) -> Vec<ManifestDependency> {
        ManifestDependency::SECTIONS
            .into_iter()
            .flat_map(|section| {
                tables
                    .get(section)
                    .and_then(Value::as_table)
                    .into_iter()
                    .flatten()
                    .map(move |(name, value)| Self::entry(shown, section, target, name, value))
            })
            .collect()
    }
}
