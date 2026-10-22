// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::collections::BTreeSet;

use syn::Path;

use crate::finding::parsing::time::imported_names::ImportedNames;

// Whether the trait an impl names is a port: one a test can hand the code an
// implementation of. A trait the package declares is one, and so is a
// dependency's -- a family's core crate declares the ports its runtimes
// implement, and the adapter that reads the clock sits in the runtime. The
// standard library's traits are not: `Display`, `Iterator` and `Drop` are a
// value's behaviour, and a test swaps none of them.
//
// The trait's origin is read through the file's imports. A bare name nobody
// imported is the standard library's only if the prelude holds it; any other
// origin this cannot tell counts as a port, so the rule stays quiet rather than
// accuse an adapter.
pub struct PortTraits<'a> {
    declared: &'a BTreeSet<String>,
    names: &'a ImportedNames,
}

impl<'a> PortTraits<'a> {
    const PACKAGE_ROOTS: [&'static str; 3] = ["crate", "self", "super"];
    const PRELUDE: [&'static str; 34] = [
        "AsMut",
        "AsRef",
        "AsyncFn",
        "AsyncFnMut",
        "AsyncFnOnce",
        "Clone",
        "Copy",
        "Default",
        "DoubleEndedIterator",
        "Drop",
        "Eq",
        "ExactSizeIterator",
        "Extend",
        "Fn",
        "FnMut",
        "FnOnce",
        "From",
        "FromIterator",
        "Future",
        "Into",
        "IntoFuture",
        "IntoIterator",
        "Iterator",
        "Ord",
        "PartialEq",
        "PartialOrd",
        "Send",
        "Sized",
        "Sync",
        "ToOwned",
        "ToString",
        "TryFrom",
        "TryInto",
        "Unpin",
    ];
    const STANDARD_ROOTS: [&'static str; 3] = ["std", "core", "alloc"];

    pub fn new(declared: &'a BTreeSet<String>, names: &'a ImportedNames) -> Self {
        Self { declared, names }
    }

    pub fn is_port(&self, path: &Path) -> bool {
        let written: Vec<String> = path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect();
        let Some(name) = written.last() else {
            return false;
        };
        if self.declared.contains(name) {
            return true;
        }
        let resolved = self
            .names
            .resolve(&written)
            .into_iter()
            .next()
            .unwrap_or_default();
        let root = resolved.split("::").next().unwrap_or_default();
        if Self::STANDARD_ROOTS.contains(&root) || Self::PACKAGE_ROOTS.contains(&root) {
            return false;
        }
        resolved != *name || !Self::PRELUDE.contains(&name.as_str())
    }
}
