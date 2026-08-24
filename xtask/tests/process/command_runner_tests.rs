// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::cell::RefCell;
use xtask::process::command_outcome::CommandOutcome;
use xtask::process::command_runner::CommandRunner;

// The seam every gate is constructed against. Lives here, beside the trait it
// implements, so the gate tests share one fake rather than each growing their
// own.
pub struct FakeCommandRunner {
    available: bool,
    streaming_code: Option<i32>,
    stdout: String,
    calls: RefCell<Vec<Vec<String>>>,
}

impl FakeCommandRunner {
    pub fn new() -> Self {
        Self {
            available: true,
            streaming_code: Some(0),
            stdout: String::new(),
            calls: RefCell::new(Vec::new()),
        }
    }

    pub fn with_available(mut self, available: bool) -> Self {
        self.available = available;
        self
    }

    pub fn with_streaming_code(mut self, streaming_code: Option<i32>) -> Self {
        self.streaming_code = streaming_code;
        self
    }

    pub fn with_stdout(mut self, stdout: &str) -> Self {
        self.stdout = String::from(stdout);
        self
    }

    pub fn calls(&self) -> Vec<Vec<String>> {
        self.calls.borrow().clone()
    }
}

impl Default for FakeCommandRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandRunner for FakeCommandRunner {
    fn is_available(&self, _binary_name: &str) -> bool {
        self.available
    }

    fn run_streaming(&self, _program: &str, args: &[String]) -> Result<Option<i32>, String> {
        self.calls.borrow_mut().push(args.to_vec());
        Ok(self.streaming_code)
    }

    fn run_capturing(&self, _program: &str, args: &[String]) -> Result<CommandOutcome, String> {
        self.calls.borrow_mut().push(args.to_vec());
        Ok(CommandOutcome::new(
            self.streaming_code,
            self.stdout.clone(),
            String::new(),
        ))
    }
}

#[test]
fn is_available_reports_what_the_implementation_was_given() {
    // Arrange
    let runner = FakeCommandRunner::new().with_available(false);

    // Act
    let available = runner.is_available("cargo-anything");

    // Assert
    assert!(!available);
}

#[test]
fn run_capturing_returns_the_canned_stdout() {
    // Arrange
    let runner = FakeCommandRunner::new().with_stdout("payload");

    // Act
    let outcome = runner.run_capturing("cargo", &[]).expect("capturing");

    // Assert
    assert_eq!(outcome.stdout, "payload");
}

#[test]
fn run_streaming_records_the_arguments_it_was_called_with() {
    // Arrange
    let runner = FakeCommandRunner::new();
    let args = vec![String::from("first"), String::from("second")];

    // Act
    let _ = runner.run_streaming("cargo", &args);

    // Assert
    assert_eq!(runner.calls(), vec![args]);
}

#[test]
fn run_streaming_through_a_trait_object_returns_the_canned_code() {
    // Arrange
    let runner = FakeCommandRunner::new().with_streaming_code(Some(2));
    let as_trait_object: &dyn CommandRunner = &runner;

    // Act
    let code = as_trait_object
        .run_streaming("cargo", &[])
        .expect("running");

    // Assert
    assert_eq!(code, Some(2));
}
