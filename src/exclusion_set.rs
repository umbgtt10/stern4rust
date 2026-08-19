// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use anyhow::Result;
use std::path::Path;
use std::path::PathBuf;

use crate::exclusion::Exclusion;
use crate::exclusion_outcome::ExclusionOutcome;

// Every `--exclude` pattern this run was given, applied to the walked paths.
//
// Exclusion happens after the walk rather than by pruning it. Pruning would be
// faster on a large vendored tree and would cost the one thing that makes an
// exclusion acceptable at all: knowing how many files each pattern removed. A
// tree that is never entered cannot be counted, and an exclusion nobody can
// count is the silent skip this tool removed from the walker in 0.4.0.
//
// A path is attributed to the **first** pattern that covers it, so two
// overlapping patterns do not both claim the same file and inflate the total.
pub struct ExclusionSet {
    exclusions: Vec<Exclusion>,
}

impl ExclusionSet {
    pub fn new(patterns: &[String]) -> Result<Self> {
        let exclusions = patterns
            .iter()
            .map(|pattern| Exclusion::new(pattern))
            .collect::<Result<Vec<_>>>()?;
        Ok(Self { exclusions })
    }

    pub fn is_empty(&self) -> bool {
        self.exclusions.is_empty()
    }

    pub fn apply(&self, paths: Vec<PathBuf>, root: &Path) -> ExclusionOutcome {
        let mut counts = vec![0usize; self.exclusions.len()];
        let mut kept = Vec::new();
        for path in paths {
            match self.covering(&path, root) {
                Some(index) => counts[index] += 1,
                None => kept.push(path),
            }
        }
        let excluded = self
            .exclusions
            .iter()
            .zip(counts)
            .map(|(exclusion, count)| (exclusion.pattern().to_string(), count))
            .collect();
        ExclusionOutcome::new(kept, excluded)
    }

    fn covering(&self, path: &Path, root: &Path) -> Option<usize> {
        let relative = path.strip_prefix(root).unwrap_or(path);
        self.exclusions
            .iter()
            .position(|exclusion| exclusion.matches(relative))
    }
}
