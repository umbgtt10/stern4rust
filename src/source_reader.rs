// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::fs;
use std::path::Path;

use crate::manifest_resolver::ManifestResolver;
use crate::offence::Offence;
use crate::rules::readable_source_rule::ReadableSourceRule;
use crate::source_file::SourceFile;

// Reads a file the walker found, and turns a failure into a finding rather than
// into the end of the run.
//
// A file that cannot be read used to abort everything, so one unreadable file
// hid every offence already found in every other file. That is the wrong trade.
// A bad manifest is genuinely a could-not-run condition -- without it nothing
// can be enumerated -- but a single unreadable file is a fact about the tree,
// and the rest of the tree is still worth reporting on.
pub struct SourceReader;

impl SourceReader {
    pub fn read(root: &Path, path: &Path) -> Result<SourceFile, Offence> {
        let relative = ManifestResolver::relative_to(root, path);
        match fs::read_to_string(path) {
            Ok(contents) => Ok(SourceFile::new(&relative, &contents)),
            Err(error) => Err(Offence::new(
                &relative,
                1,
                ReadableSourceRule::NAME,
                format!("file could not be read: {error}"),
            )
            .with_subject(&relative)),
        }
    }
}
