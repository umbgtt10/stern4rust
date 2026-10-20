// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A rule that was selected but could not run, and what it was waiting for -- two
// strings a pair would leave to position, named.

use stern4rust::reporting::unconfigured_rule::UnconfiguredRule;

#[test]
fn new_keeps_the_name_and_what_it_waits_for() {
    // Arrange & Act
    let unconfigured = UnconfiguredRule::new("header", "needs --header-file");

    // Assert
    assert_eq!(unconfigured.name, "header");
    assert_eq!(unconfigured.requirement, "needs --header-file");
}
