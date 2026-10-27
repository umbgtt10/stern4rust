// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::collections::BTreeMap;
use std::fs::read_to_string;
use std::fs::write;

use serde::Deserialize;
use serde::Serialize;
use serde_json::from_str;
use serde_json::to_string_pretty;
use stern4rust::settings::adoption::baseline::Baseline;
use stern4rust::settings::adoption::baseline_failure::BaselineFailure;
use stern4rust::source_path::SourcePath;
use stern4rust::traits::baseline_store::BaselineStore;

// A baseline as the JSON file a repository checks in: every fingerprint with
// the number of times it was recorded, sorted, because a diff that reorders
// itself between runs is a diff nobody reviews.
#[derive(Debug, Clone, Copy, Default)]
pub struct JsonBaselineStore;

#[derive(Debug, Default, Deserialize, Serialize)]
struct JsonBaseline {
    pub offences: BTreeMap<String, usize>,
}

impl JsonBaselineStore {
    pub fn new() -> Self {
        Self
    }
}

impl BaselineStore for JsonBaselineStore {
    fn load(&self, path: &SourcePath) -> Result<Baseline, BaselineFailure> {
        let text = read_to_string(path.as_str()).map_err(|cause| BaselineFailure::Unreadable {
            path: path.clone(),
            cause: Box::new(cause),
        })?;
        let stored: JsonBaseline = from_str(&text).map_err(|cause| BaselineFailure::Invalid {
            path: path.clone(),
            cause: Box::new(cause),
        })?;
        Ok(Baseline::new(stored.offences))
    }

    fn save(&self, path: &SourcePath, baseline: &Baseline) -> Result<(), BaselineFailure> {
        let stored = JsonBaseline {
            offences: baseline.counts().clone(),
        };
        let text = to_string_pretty(&stored).map_err(|cause| BaselineFailure::Unrenderable {
            path: path.clone(),
            cause: Box::new(cause),
        })?;
        write(path.as_str(), text).map_err(|cause| BaselineFailure::Unwritable {
            path: path.clone(),
            cause: Box::new(cause),
        })
    }
}
