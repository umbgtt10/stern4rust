// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::settings::adoption::baseline::Baseline;
use crate::settings::adoption::baseline_failure::BaselineFailure;
use crate::source_path::SourcePath;

// Where a baseline is kept between runs, in whatever form it is written down.
pub trait BaselineStore {
    fn load(&self, path: &SourcePath) -> Result<Baseline, BaselineFailure>;

    fn save(&self, path: &SourcePath, baseline: &Baseline) -> Result<(), BaselineFailure>;
}
