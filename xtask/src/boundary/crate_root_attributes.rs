// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use syn::{File, Meta, Path};

// A crate root's inner attributes as they read: `no_std`, `forbid(unsafe_code)`
// -- a list attribute with its nested names in order, comma-separated.
pub struct CrateRootAttributes;

impl CrateRootAttributes {
    pub fn new() -> Self {
        Self
    }

    pub fn of(&self, file: &File) -> Vec<String> {
        file.attrs
            .iter()
            .map(|attribute| {
                let name = Self::path_text(attribute.path());
                match &attribute.meta {
                    Meta::List(_) => {
                        let mut nested = Vec::new();
                        let _ = attribute.parse_nested_meta(|meta| {
                            nested.push(Self::path_text(&meta.path));
                            Ok(())
                        });
                        format!("{name}({})", nested.join(", "))
                    }
                    _ => name,
                }
            })
            .collect()
    }

    fn path_text(path: &Path) -> String {
        path.segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>()
            .join("::")
    }
}

impl Default for CrateRootAttributes {
    fn default() -> Self {
        Self::new()
    }
}
