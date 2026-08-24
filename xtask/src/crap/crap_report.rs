// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::crap::crap_function::CrapFunction;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct CrapReport {
    pub total_functions: u32,
    pub crappy_functions: u32,
    pub crappy_percent: f64,
    pub functions: Vec<CrapFunction>,
}

impl CrapReport {
    pub fn is_clean(&self) -> bool {
        self.crappy_functions == 0
    }

    pub fn summary(&self) -> String {
        format!(
            "crap4rust: {}/{} functions crappy ({:.1}%)",
            self.crappy_functions, self.total_functions, self.crappy_percent
        )
    }

    pub fn offenders(&self) -> Vec<&CrapFunction> {
        self.functions
            .iter()
            .filter(|function| !function.is_clean())
            .collect()
    }
}
