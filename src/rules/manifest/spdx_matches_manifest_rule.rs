// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::reporting::offence::Offence;
use crate::reporting::rule_explanation::RuleExplanation;
use crate::rule::Rule;
use crate::source_file::SourceFile;

// Every file's SPDX identifier says what the manifest says.
//
// `header` compares a header against a text file and nothing else, so a file
// whose header is perfectly formed can still declare a licence the package does
// not. The two statements of the same fact -- `license` in `Cargo.toml` and
// `SPDX-License-Identifier` in every header -- had nothing holding them
// together.
//
// This rule takes its expected value from the manifest rather than from a flag,
// which is what makes it different from `header` in the way that matters: it
// needs no `--header-file` to hold. It is the first rule whose configuration
// comes from the package being judged instead of from the command line, and
// `ManifestResolver` reads it once for the run.
//
// A manifest that declares no licence leaves the rule with **nothing to work
// from**, so it answers `is_configured` with false and the registry drops it --
// the same third state the header rule uses, and the report names it as not
// applied rather than passing it in silence.
//
// That was measured rather than chosen. Reporting the silent manifest as an
// offence instead fired once per package root: `braintax4rust`, a workspace of
// twenty packages, produced twenty identical lines against `Cargo.toml`.
//
// The header is the comment block a file opens with -- everything before the
// first line that is neither blank nor a `//` comment. An SPDX line below that
// is prose or code, and does not declare anything.
pub struct SpdxMatchesManifestRule {
    license: Option<String>,
}

impl SpdxMatchesManifestRule {
    pub const MANIFEST: &'static str = "Cargo.toml";
    pub const MARKER: &'static str = "SPDX-License-Identifier:";

    pub fn new(license: Option<String>) -> Self {
        Self { license }
    }

    // Everything before the first line that is neither blank nor a comment.
    fn header_of(file: &SourceFile) -> Vec<&String> {
        file.lines()
            .iter()
            .take_while(|line| {
                let text = line.trim();
                text.is_empty() || text.starts_with("//")
            })
            .collect()
    }

    fn declared_in(file: &SourceFile) -> Option<String> {
        Self::header_of(file).into_iter().find_map(|line| {
            line.split_once(Self::MARKER)
                .map(|(_, found)| found.trim().to_string())
        })
    }

    fn missing(&self, file: &SourceFile, expected: &str) -> Offence {
        Offence::new(
            file.relative_path(),
            1,
            self.name(),
            format!(
                "{} carries no `{}`, so nothing ties it to the `{expected}` the manifest declares",
                file.relative_path(),
                Self::MARKER
            ),
            format!(
                "add `// {} {expected}` to the header, or correct the manifest",
                Self::MARKER
            ),
        )
        .with_subject(file.relative_path())
        .with_expected(expected)
    }

    fn differs(&self, file: &SourceFile, found: &str, expected: &str) -> Offence {
        Offence::new(
            file.relative_path(),
            1,
            self.name(),
            format!(
                "{} declares `{found}` where the manifest declares `{expected}`",
                file.relative_path()
            ),
            format!(
                "change the header to `// {} {expected}`, or correct the manifest",
                Self::MARKER
            ),
        )
        .with_subject(file.relative_path())
        .with_expected(expected)
    }
}

impl Rule for SpdxMatchesManifestRule {
    fn name(&self) -> &'static str {
        "spdx-matches-manifest"
    }

    fn check(&self, file: &SourceFile) -> Vec<Offence> {
        let Some(expected) = &self.license else {
            return Vec::new();
        };
        match Self::declared_in(file) {
            None => vec![self.missing(file, expected)],
            Some(found) if &found != expected => vec![self.differs(file, &found, expected)],
            Some(_) => Vec::new(),
        }
    }

    fn check_workspace(&self, _files: &[SourceFile]) -> Vec<Offence> {
        Vec::new()
    }

    // The manifest is what configures this rule, so a manifest with no `license`
    // leaves it nothing to check against.
    fn requirement(&self) -> Option<&'static str> {
        Some("needs a `license` field in Cargo.toml")
    }

    fn is_configured(&self) -> bool {
        self.license.is_some()
    }

    fn explanation(&self) -> RuleExplanation {
        RuleExplanation::new(
            self.name(),
            "Every file's SPDX identifier says what the manifest says.",
            "// SPDX-License-Identifier: Apache-2.0   -- manifest says MIT",
            "// SPDX-License-Identifier: MIT",
        )
    }
}
