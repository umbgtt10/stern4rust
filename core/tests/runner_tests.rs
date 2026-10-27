// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The wiring from an invocation to a verdict. What a run finds is the command
// line's to test, end to end; what the core owes it is a run that could not
// judge anything coming back as an Err rather than as a verdict.

use crate::support::in_memory_baseline_store::InMemoryBaselineStore;
use crate::support::in_memory_config_source::InMemoryConfigSource;
use crate::support::in_memory_source_tree::InMemorySourceTree;
use crate::support::in_memory_source_tree::path_of;
use crate::support::in_memory_workspace_reader::InMemoryWorkspaceReader;
use stern4rust::runner::Runner;
use stern4rust::settings::invocation::Invocation;

#[test]
fn run_reporting_with_a_workspace_that_cannot_be_read_fails() {
    // Arrange
    let workspace = InMemoryWorkspaceReader::failing();
    let invocation = Invocation {
        manifest_directory: path_of("no/such/workspace"),
        ..Invocation::default()
    };

    // Act
    let failure = Runner::new(
        &workspace,
        &InMemorySourceTree::new(),
        &InMemoryConfigSource::absent(),
        &InMemoryBaselineStore::new(),
    )
    .run_reporting(&invocation)
    .expect_err("an unreadable workspace");

    // Assert
    assert_eq!(failure.to_string(), "not held");
}
