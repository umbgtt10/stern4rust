// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::support::in_memory_source_tree::NotHeld;
use std::cell::RefCell;
use std::collections::BTreeMap;
use stern4rust::settings::adoption::baseline::Baseline;
use stern4rust::settings::adoption::baseline_failure::BaselineFailure;
use stern4rust::source_path::SourcePath;
use stern4rust::traits::baseline_store::BaselineStore;

// Baselines kept in memory by path: what a run loads, and what it saved.
pub struct InMemoryBaselineStore {
    baselines: RefCell<BTreeMap<SourcePath, BTreeMap<String, usize>>>,
}

impl InMemoryBaselineStore {
    pub fn new() -> Self {
        Self {
            baselines: RefCell::new(BTreeMap::new()),
        }
    }

    pub fn saved(&self, path: &SourcePath) -> Option<BTreeMap<String, usize>> {
        self.baselines.borrow().get(path).cloned()
    }

    pub fn with_baseline(self, path: &SourcePath, counts: BTreeMap<String, usize>) -> Self {
        self.baselines.borrow_mut().insert(path.clone(), counts);
        self
    }
}

impl Default for InMemoryBaselineStore {
    fn default() -> Self {
        Self::new()
    }
}

impl BaselineStore for InMemoryBaselineStore {
    fn load(&self, path: &SourcePath) -> Result<Baseline, BaselineFailure> {
        self.baselines
            .borrow()
            .get(path)
            .cloned()
            .map(Baseline::new)
            .ok_or_else(|| BaselineFailure::Unreadable {
                path: path.clone(),
                cause: Box::new(NotHeld),
            })
    }

    fn save(&self, path: &SourcePath, baseline: &Baseline) -> Result<(), BaselineFailure> {
        self.baselines
            .borrow_mut()
            .insert(path.clone(), baseline.counts().clone());
        Ok(())
    }
}
