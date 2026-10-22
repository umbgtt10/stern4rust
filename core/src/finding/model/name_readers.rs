// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::sync::Arc;

// Which files of the workspace read each name, asked from inside one package.
//
// Built once for the whole workspace and handed to every package, because a
// constant read only by a sibling crate is shared all the same: counted by
// package, every constant a sibling reads looked unread. A file asks by its
// package-relative path, the way a rule sees it, and is left out of its own
// answer; the readers come back workspace-relative, so a correction can name a
// file in another member.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NameReaders {
    package: String,
    readers: Arc<BTreeMap<String, BTreeSet<String>>>,
}

impl NameReaders {
    pub fn new(readers: BTreeMap<String, BTreeSet<String>>) -> Self {
        Self {
            package: String::new(),
            readers: Arc::new(readers),
        }
    }

    pub fn in_package(&self, package: &str) -> Self {
        Self {
            package: package.to_string(),
            readers: Arc::clone(&self.readers),
        }
    }

    pub fn others(&self, name: &str, file: &str) -> Vec<String> {
        let asking = self.in_workspace(file);
        self.readers
            .get(name)
            .map(|files| {
                files
                    .iter()
                    .filter(|reader| **reader != asking)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn reads(&self, name: &str, file: &str) -> bool {
        self.readers
            .get(name)
            .is_some_and(|files| files.contains(&self.in_workspace(file)))
    }

    fn in_workspace(&self, file: &str) -> String {
        if self.package.is_empty() {
            file.to_string()
        } else {
            format!("{}/{file}", self.package)
        }
    }
}
