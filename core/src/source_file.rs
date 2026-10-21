// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// One source file, already normalised so rules never have to think about how it
// reached the disk.
//
// Two normalisations, both of which would otherwise make every file on a Windows
// checkout fail a rule that is really about content:
//
//   - a trailing carriage return is stripped, because git's autocrlf rewrites
//     line endings on checkout and a byte-for-byte comparison would fail on
//     every line of every file
//   - a leading UTF-8 byte order mark is stripped, because editors add one
//     invisibly and it would otherwise sit in front of the first character of
//     line 1
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    relative_path: String,
    lines: Vec<String>,
}

impl SourceFile {
    pub fn new(relative_path: &str, contents: &str) -> Self {
        let contents = contents.strip_prefix('\u{feff}').unwrap_or(contents);
        Self {
            relative_path: relative_path.replace('\\', "/"),
            lines: contents
                .split('\n')
                .map(|line| line.strip_suffix('\r').unwrap_or(line).to_string())
                .collect(),
        }
    }

    // The first line of the block an item starting at `start` heads. A comment
    // introducing an item belongs to it, so the block reaches up over the
    // comment lines directly above -- but never past `floor`, the line after the
    // item above, whose own lines can look like comments: a string literal's
    // continuation lines may start with `//`.
    pub fn block_start(&self, start: usize, floor: usize) -> usize {
        let mut first = start;
        while first > floor && self.is_comment(first - 1) {
            first -= 1;
        }
        first
    }

    // Rejoined from the normalised lines rather than kept alongside them, so a
    // parser and a line-counting rule can never disagree about what the file
    // says.
    pub fn contents(&self) -> String {
        self.lines.join(
            "
",
        )
    }

    // An empty file splits into one empty line, which is not the same as having
    // a line of content.
    pub fn is_empty(&self) -> bool {
        self.lines.iter().all(|line| line.trim().is_empty())
    }

    // Whether this file sits under `root` and is not named one of `except`: the
    // registries a rule about content leaves to module-registry, at any depth.
    pub fn is_under(&self, root: &str, except: &[&str]) -> bool {
        self.relative_path.starts_with(root)
            && !self
                .relative_path
                .rsplit('/')
                .next()
                .is_some_and(|name| except.contains(&name))
    }

    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    pub fn relative_path(&self) -> &str {
        &self.relative_path
    }

    fn is_comment(&self, line: usize) -> bool {
        self.lines
            .get(line.saturating_sub(1))
            .is_some_and(|text| text.trim_start().starts_with("//"))
    }
}
