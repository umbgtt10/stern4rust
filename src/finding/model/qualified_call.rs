// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A call reached through a path the file never imported.
//
// The correction is the interesting part. Every such path can be repaired by
// importing enough of it that at most one segment is left at the call site, and
// the two shapes differ: `syn::parse_file` has nothing worth keeping as a
// qualifier, so the whole path is imported and the call becomes `parse_file`,
// while `std::env::args` keeps `env` because it says something -- `use std::env`
// and `env::args`.
pub struct QualifiedCall {
    pub path: String,
    pub line: usize,
}

impl QualifiedCall {
    pub fn new(path: &str, line: usize) -> Self {
        Self {
            path: path.to_string(),
            line,
        }
    }

    // What the call site reads as afterwards: the last segment on its own, or
    // the last two when a module qualifier is worth keeping.
    pub fn call(&self) -> String {
        let segments = self.segments();
        if segments.len() <= 2 {
            return segments.last().copied().unwrap_or_default().to_string();
        }
        segments[segments.len() - 2..].join("::")
    }

    // What to import so that at most one imported segment is left at the call.
    pub fn import(&self) -> String {
        let segments = self.segments();
        if segments.len() <= 2 {
            return self.path.clone();
        }
        segments[..segments.len() - 1].join("::")
    }

    fn segments(&self) -> Vec<&str> {
        self.path.split("::").collect()
    }
}
