// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::cmp::Ordering;
use core::fmt;

// A path held as text, because the core has no `std::path`. `/` is its only
// separator: the edge that hands paths in normalises the platform's own to it.
// It compares by component, so `/a/b` and `/a/b/` are one path; `.` and `..`
// are compared as written, which `std::path` would normalise -- cargo never
// hands either over.
#[derive(Debug, Clone, Default)]
pub struct SourcePath {
    text: String,
}

impl SourcePath {
    pub fn new(text: String) -> Self {
        Self { text }
    }

    pub fn as_str(&self) -> &str {
        &self.text
    }

    pub fn extension(&self) -> Option<&str> {
        let name = self.file_name()?;
        match name.rsplit_once('.') {
            Some((stem, extension)) if !stem.is_empty() => Some(extension),
            _ => None,
        }
    }

    pub fn file_name(&self) -> Option<&str> {
        self.components()
            .last()
            .copied()
            .filter(|name| *name != "..")
    }

    pub fn file_stem(&self) -> Option<&str> {
        let name = self.file_name()?;
        match name.rsplit_once('.') {
            Some((stem, _)) if !stem.is_empty() => Some(stem),
            _ => Some(name),
        }
    }

    pub fn join(&self, name: &str) -> Self {
        if self.text.is_empty() || name.starts_with(Self::is_separator) {
            return Self::new(String::from(name));
        }
        if self.text.ends_with(Self::is_separator) {
            return Self::new(format!("{}{name}", self.text));
        }
        Self::new(format!("{}/{name}", self.text))
    }

    pub fn parent(&self) -> Option<Self> {
        if self.components().is_empty() {
            return None;
        }
        let trimmed = self.text.trim_end_matches(Self::is_separator);
        let cut = trimmed.rfind(Self::is_separator).unwrap_or(0);
        let head = trimmed[..cut].trim_end_matches(Self::is_separator);
        if head.is_empty() && self.has_root() {
            return Some(Self::new(String::from(&self.text[..1])));
        }
        Some(Self::new(String::from(head)))
    }

    pub fn strip_prefix(&self, base: &Self) -> Option<&str> {
        if self.has_root() != base.has_root() {
            return None;
        }
        let mut rest = self.text.as_str();
        for expected in base.components() {
            rest = rest.trim_start_matches(Self::is_separator);
            let end = rest.find(Self::is_separator).unwrap_or(rest.len());
            if &rest[..end] != expected {
                return None;
            }
            rest = &rest[end..];
        }
        Some(rest.trim_start_matches(Self::is_separator))
    }

    fn components(&self) -> Vec<&str> {
        self.text
            .split(Self::is_separator)
            .filter(|component| !component.is_empty())
            .collect()
    }

    fn has_root(&self) -> bool {
        self.text.starts_with(Self::is_separator)
    }

    fn is_separator(character: char) -> bool {
        character == '/'
    }
}

impl fmt::Display for SourcePath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.text)
    }
}

impl PartialEq for SourcePath {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for SourcePath {}

impl PartialOrd for SourcePath {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SourcePath {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .has_root()
            .cmp(&self.has_root())
            .then_with(|| self.components().cmp(&other.components()))
    }
}
