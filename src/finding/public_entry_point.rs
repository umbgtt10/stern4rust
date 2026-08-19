// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// One publicly reachable function, identified by name and arity.
//
// Arity rather than the parameter types, and that limit is the rule's whole
// honesty: at a call site `check(3, &paths)` gives two arguments and nothing
// more. Whether `3` is a `usize` and `&paths` a `&[&str]` is type inference,
// which is rustc's work and not something a syntax tree can answer. Arity is
// available, costs nothing, and separates `new()` from `new(a, b)` -- which is
// most of what a name alone confuses.
//
// The receiver does not count. `printer.with_fixed(12)` passes one argument and
// `pub fn with_fixed(self, fixed: usize)` declares one parameter besides self,
// so the two match.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PublicEntryPoint {
    pub name: String,
    pub arity: usize,
}

impl PublicEntryPoint {
    pub fn new(name: &str, arity: usize) -> Self {
        Self {
            name: name.to_string(),
            arity,
        }
    }

    // How the offence names it, so two entry points sharing a name are still
    // told apart on the page.
    pub fn signature(&self) -> String {
        format!("{}/{}", self.name, self.arity)
    }
}
