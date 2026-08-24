// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use syn::Item;
use syn::ItemUse;
use syn::parse_file;
use syn::spanned::Spanned;

use crate::finding::model::import_path::ImportPath;
use crate::reporting::offence::Offence;
use crate::reporting::rule_explanation::RuleExplanation;
use crate::rule::Rule;
use crate::source_file::SourceFile;

// Imports in `src/` run in alphabetic order.
//
// `test-file-structure` has asked this of `tests/` since the first release and
// nothing asked it of the source tree, which is where `imported-paths`
// routinely *adds* lines -- 201 of them across the sibling tools -- with nothing
// saying where a new one lands. The result is a file whose first import block is
// sorted and whose second is whatever order things were needed in.
//
// This is a scope change rather than a new question, and the stand-downs come
// with it. `cargo fmt` runs first in the gate and orders `self`, `super`,
// `crate` and uppercase-initial paths by rules of its own, so a rule demanding
// the alphabet there would write a file no edit could make green -- each run
// undoing the last. `ImportPath` already decides which pairs those are, and this
// rule asks it rather than deciding again.
//
// The consequence worth stating is that the stand-down does far more work here
// than it ever has in `tests/`. A test file imports the crate under test, so
// `crate::` never appears; a source file usually leads with a block of it.
// Measured on this crate, **56% of adjacent import pairs in `src/` stand down**
// -- so more than half of what the rule appears to check, it does not.
//
// A block ends where the lines stop being consecutive: a blank line or a comment
// between two imports separates them, and the first import of a block is
// compared with nothing.
pub struct OrderedImportsRule;

impl OrderedImportsRule {
    pub const SOURCE_ROOT: &'static str = "src/";

    pub fn new() -> Self {
        Self
    }

    fn applies_to(file: &SourceFile) -> bool {
        file.relative_path().starts_with(Self::SOURCE_ROOT)
    }

    fn imports(items: &[Item]) -> Vec<&ItemUse> {
        items
            .iter()
            .filter_map(|item| match item {
                Item::Use(import) => Some(import),
                _ => None,
            })
            .collect()
    }

    // The text as written, taken from the line rather than rebuilt from the
    // syntax tree, so the offence quotes what the reader will search for.
    //
    // Anchored on the `use` keyword, not on the item. An item's span begins at
    // its first attribute, so a gated import read from `span().start()` was
    // judged -- and quoted -- by its `#[cfg(...)]` line instead of its path.
    //
    // The corrections that produced were not merely noisy, they were wrong.
    // `embassy-logging` gates four imports on three features, and following
    // every correction would have ordered them by feature name: `rtt_sink`
    // above `qemu_sink` above `host_sink`, the paths in reverse, by the rule
    // that exists to alphabetise them. It cut both ways -- two imports genuinely
    // out of order went unreported because their attributes happened to sort.
    //
    // `follows` deliberately keeps the item span. An attributed import sitting
    // directly beneath another is consecutive, and anchoring that on the `use`
    // line too would put the attribute in the gap and stop the pair being
    // compared at all.
    fn text_of(file: &SourceFile, import: &ItemUse) -> String {
        file.lines()
            .get(import.use_token.span().start().line - 1)
            .map(|line| line.trim().to_string())
            .unwrap_or_default()
    }

    // Consecutive lines, so a blank line or a comment between two imports ends
    // the block and the pair is never compared.
    fn follows(previous: &ItemUse, import: &ItemUse) -> bool {
        import.span().start().line == previous.span().end().line + 1
    }

    fn offence(&self, file: &SourceFile, previous: &str, import: &str, line: usize) -> Offence {
        Offence::new(
            file.relative_path(),
            line,
            self.name(),
            format!("`{import}` is out of alphabetic order; it follows `{previous}`"),
            format!("move `{import}` above `{previous}`"),
        )
        .with_subject(import)
    }
}

impl Default for OrderedImportsRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for OrderedImportsRule {
    fn name(&self) -> &'static str {
        "ordered-imports"
    }

    fn check(&self, file: &SourceFile) -> Vec<Offence> {
        if !Self::applies_to(file) {
            return Vec::new();
        }
        let Ok(syntax) = parse_file(&file.contents()) else {
            return Vec::new();
        };
        let imports = Self::imports(&syntax.items);
        imports
            .windows(2)
            .filter(|pair| Self::follows(pair[0], pair[1]))
            .filter_map(|pair| {
                let previous = Self::text_of(file, pair[0]);
                let import = Self::text_of(file, pair[1]);
                if ImportPath::decides_order(&previous, &import)
                    || ImportPath::is_ordered(&previous, &import)
                {
                    return None;
                }
                Some(self.offence(file, &previous, &import, pair[1].span().start().line))
            })
            .collect()
    }

    fn check_workspace(&self, _files: &[SourceFile]) -> Vec<Offence> {
        Vec::new()
    }

    fn requirement(&self) -> Option<&'static str> {
        None
    }

    fn is_configured(&self) -> bool {
        true
    }

    fn explanation(&self) -> RuleExplanation {
        RuleExplanation::new(
            self.name(),
            "Imports in src/ run in alphabetic order.",
            "use zzz::Zed;\nuse aaa::Alpha;",
            "use aaa::Alpha;\nuse zzz::Zed;",
        )
    }
}
