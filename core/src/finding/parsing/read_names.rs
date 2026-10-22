// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use syn::Macro;
use syn::PatIdent;
use syn::Path;
use syn::UseName;
use syn::UseRename;
use syn::parse_file;
use syn::visit::Visit;
use syn::visit::visit_pat_ident;
use syn::visit::visit_path;

use crate::finding::parsing::identifiers::Identifiers;
use crate::source_file::SourceFile;

// The names a file reads: every segment of every path it writes -- in an
// expression, a type, an attribute --, every name it imports, every bare name a
// pattern matches, which is how a `match` reads a constant, and every identifier
// in a macro's tokens, which is where an assertion reads one. The names it
// declares are not paths, so a file declaring a constant of its own does not
// read another file's of the same name.
pub struct ReadNames {
    names: BTreeSet<String>,
}

impl ReadNames {
    // Each name any of these files reads, with the paths of the files reading
    // it, as the files name themselves.
    pub fn index(files: &[SourceFile]) -> BTreeMap<String, BTreeSet<String>> {
        let mut index: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for file in files {
            for name in Self::of(file) {
                index
                    .entry(name)
                    .or_default()
                    .insert(file.relative_path().to_string());
            }
        }
        index
    }

    pub fn of(file: &SourceFile) -> BTreeSet<String> {
        let Ok(syntax) = parse_file(&file.contents()) else {
            return BTreeSet::new();
        };
        let mut reader = Self {
            names: BTreeSet::new(),
        };
        reader.visit_file(&syntax);
        reader.names
    }
}

impl<'ast> Visit<'ast> for ReadNames {
    fn visit_macro(&mut self, mac: &'ast Macro) {
        self.visit_path(&mac.path);
        self.names.extend(Identifiers::of(mac.tokens.clone()));
    }

    fn visit_pat_ident(&mut self, pattern: &'ast PatIdent) {
        self.names.insert(pattern.ident.to_string());
        visit_pat_ident(self, pattern);
    }

    fn visit_path(&mut self, path: &'ast Path) {
        self.names.extend(
            path.segments
                .iter()
                .map(|segment| segment.ident.to_string()),
        );
        visit_path(self, path);
    }

    fn visit_use_name(&mut self, name: &'ast UseName) {
        self.names.insert(name.ident.to_string());
    }

    fn visit_use_rename(&mut self, rename: &'ast UseRename) {
        self.names.insert(rename.ident.to_string());
    }
}
