// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// How a test came to depend on time, in the words an offence uses for it.

use stern4rust::finding::model::time::time_call_kind::TimeCallKind;

#[test]
fn verb_of_each_kind_says_what_its_call_does() {
    // Arrange & Act
    let verbs = [
        TimeCallKind::Clock.verb(),
        TimeCallKind::Wait.verb(),
        TimeCallKind::Race.verb(),
        TimeCallKind::Random.verb(),
    ];

    // Assert
    assert_eq!(
        verbs,
        [
            "reads the clock",
            "waits on a timer",
            "races a timer",
            "draws unseeded randomness"
        ]
    );
}
