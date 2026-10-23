// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A tuple holding two elements of one type, as written: the tuple, and the type
// only position tells apart.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepeatedTuple {
    tuple: String,
    element: String,
}

impl RepeatedTuple {
    pub fn new(tuple: String, element: String) -> Self {
        Self { tuple, element }
    }

    pub fn element(&self) -> &str {
        &self.element
    }

    pub fn tuple(&self) -> &str {
        &self.tuple
    }
}
