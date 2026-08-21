// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// What a scan did, as opposed to what it found.
//
// Four numbers travelled to the report as four arguments, and adding the
// per-package rosters took it past the point where a reader could tell them
// apart. They describe one thing between them -- the work, not the verdict --
// so they travel as one thing.

use stern4rust::reporting::scan_totals::ScanTotals;

#[test]
fn new_keeps_the_counts_it_was_given() {
    // Arrange & Act
    let totals = ScanTotals::new(250, 2);

    // Assert
    assert_eq!(totals.files_scanned, 250);
    assert_eq!(totals.fixed, 2);
}

// A run that repaired nothing is the ordinary one, and the report says nothing
// about --fix unless it did something.
#[test]
fn new_without_repairs_records_none() {
    // Arrange & Act
    let totals = ScanTotals::new(127, 0);

    // Assert
    assert_eq!(totals.fixed, 0);
}
