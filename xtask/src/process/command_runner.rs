// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::process::command_outcome::CommandOutcome;

pub trait CommandRunner {
    fn is_available(&self, binary_name: &str) -> bool;

    fn run_streaming(&self, program: &str, args: &[String]) -> Result<Option<i32>, String>;

    fn run_capturing(&self, program: &str, args: &[String]) -> Result<CommandOutcome, String>;
}
