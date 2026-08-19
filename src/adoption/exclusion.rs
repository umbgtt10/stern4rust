// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use anyhow::Context;
use anyhow::Result;
use globset::Glob;
use globset::GlobMatcher;
use std::path::Path;

// One `--exclude` pattern and the paths it covers.
//
// Matched against the package-relative path, so a pattern can be written down
// in a repository and mean the same thing on every machine that checks it out.
// Separators are normalised to `/` before matching for the same reason: the
// walker hands back backslashes on Windows, and an exclusion that worked on one
// developer's machine and quietly stopped working on another would be worse
// than no exclusion at all.
//
// An unusable pattern is an error rather than a pattern matching nothing.
// Matching nothing is a legitimate outcome -- a tree that has since been
// deleted -- and the report says so; the two must not look alike.
pub struct Exclusion {
    pattern: String,
    matcher: GlobMatcher,
}

impl Exclusion {
    pub fn new(pattern: &str) -> Result<Self> {
        let matcher = Glob::new(pattern)
            .with_context(|| format!("`{pattern}` is not a usable exclude pattern"))?
            .compile_matcher();
        Ok(Self {
            pattern: pattern.to_string(),
            matcher,
        })
    }

    pub fn matches(&self, relative: &Path) -> bool {
        self.matcher.is_match(Self::normalised(relative))
    }

    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    fn normalised(relative: &Path) -> String {
        relative.to_string_lossy().replace('\\', "/")
    }
}
