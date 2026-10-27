// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::string::String;

use crate::source_file::SourceFile;

// One fixer, the rule it repairs, and nothing else. A fixer runs only where the
// package applies its rule -- a rule a package skips, or a run did not select,
// governs nothing, and a fixer must never edit a file no rule governs -- so it
// says which rule that is rather than the runner knowing.
pub trait SourceRewriter {
    fn rule(&self) -> &'static str;

    // The file as this fixer leaves it, or None when it is not this fixer's to
    // touch or has nothing to change, so a caller never writes a file it did
    // not need to.
    fn rewrite(&self, file: &SourceFile) -> Option<String>;
}
