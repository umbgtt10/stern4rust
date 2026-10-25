// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// One call through which a function depends on time: its kind, as written, and where.

use stern4rust::finding::model::time::time_call::TimeCall;
use stern4rust::finding::model::time::time_call_kind::TimeCallKind;

#[test]
fn new_keeps_the_call_as_it_was_written() {
    // Arrange & Act
    let call = TimeCall::new(TimeCallKind::Wait, "thread::sleep", 7, 9);

    // Assert
    assert_eq!(call.written(), "thread::sleep");
    assert_eq!((call.line(), call.column()), (7, 9));
    assert_eq!(call.kind(), TimeCallKind::Wait);
}
