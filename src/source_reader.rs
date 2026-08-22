// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::fs;
use std::path::Path;

use crate::reporting::offence::Offence;
use crate::rules::source::readable_source_rule::ReadableSourceRule;
use crate::settings::manifest_resolver::ManifestResolver;
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
    // The offence is boxed because it is much larger than the SourceFile it is
    // returned instead of, and an unboxed Err variant that size makes every
    // successful read pay for the failing one.
    pub fn read(root: &Path, path: &Path) -> Result<SourceFile, Box<Offence>> {
        let relative = ManifestResolver::relative_to(root, path);
        match fs::read_to_string(path) {
            Ok(contents) => Ok(SourceFile::new(&relative, &contents)),
            Err(error) => Err(Box::new(
                Offence::new(
                    &relative,
                    1,
                    ReadableSourceRule::NAME,
                    format!("file could not be read: {error}"),
                    "check that the file exists and that its permissions allow reading it"
                        .to_string(),
                )
                .with_subject(&relative),
            )),
        }
    }
}
