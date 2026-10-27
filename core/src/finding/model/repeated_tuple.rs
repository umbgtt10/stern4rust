// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A tuple holding two elements of one type, as written: the tuple, and the type
// only position tells apart.
use alloc::string::String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepeatedTuple {
    pub tuple: String,
    pub element: String,
}
