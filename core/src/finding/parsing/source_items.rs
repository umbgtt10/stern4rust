// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use syn::Item;
use syn::parse_file;

use crate::reporting::offence::Offence;
use crate::source_file::SourceFile;

// The parsed items of a file under a source root, or none -- and the offences a
// rule finds in them.
//
// A rule that judges `src/` alone opens with the same two refusals: a file
// outside the root is not its business, and a file that does not parse is
// rustc's to report. `PureTraitsRule` and `ReturnedMutationsRule` carried that
// guard and parse in their `check`, then the same find-then-judge over what came
// back. Only the two halves differ -- what each looks for in the items, and what
// it says about each one found -- so those are what a rule hands in.
pub struct SourceItems;

impl SourceItems {
    pub fn offences<T, F, J>(root: &str, file: &SourceFile, find: F, judge: J) -> Vec<Offence>
    where
        F: for<'i> Fn(&'i [Item]) -> Vec<&'i T>,
        J: Fn(&SourceFile, &T) -> Vec<Offence>,
    {
        let items = Self::under(root, file);
        find(&items)
            .into_iter()
            .flat_map(|found| judge(file, found))
            .collect()
    }

    pub fn under(root: &str, file: &SourceFile) -> Vec<Item> {
        if !file.relative_path().starts_with(root) {
            return Vec::new();
        }
        parse_file(&file.contents())
            .map(|syntax| syntax.items)
            .unwrap_or_default()
    }
}
