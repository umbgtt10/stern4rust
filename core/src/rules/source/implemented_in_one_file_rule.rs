// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use crate::finding::parsing::type_homes::TypeHomes;
use crate::reporting::offence::Offence;
use crate::reporting::rule_explanation::RuleExplanation;
use crate::rule::Rule;
use crate::source_file::SourceFile;

// A type's inherent impl lives in one file.
//
// single-implemented-type bounds a file to one implemented type and
// named-subject names the file after it; nothing bounded a type to one file.
// Rust accepts an inherent impl for a type in any file of its crate, so a type
// that grew could be continued file after file, each passing both rules and the
// whole a subject no file names. The cost shows up as visibility: privacy is per
// module, so whatever those files share has to be `pub(crate)` or `pub(super)`.
//
// A type with inherent impls in two or more files of the package is reported
// once, at its declaration, naming every file. A name declared in two or more
// files is read as alternatives, each to be implemented beside its own
// declaration. The correction offers a trait when the impls are split by
// platform, and otherwise the declaring file or a split into types of their own.
// Trait impls are left alone, and tests/ is exempt.
pub struct ImplementedInOneFileRule;

impl ImplementedInOneFileRule {
    pub const NAME: &'static str = "implemented-in-one-file";
    pub const SOURCE_ROOT: &'static str = "src/";

    pub fn new() -> Self {
        Self
    }

    fn correction(homes: &TypeHomes, name: &str) -> String {
        if homes.split_by_platform(name) {
            return String::from(
                "put the contract in a trait in `traits/` and give each platform its own \
                 implementing type",
            );
        }
        match homes.declaring_files(name).as_slice() {
            [] => format!(
                "move every `impl {name}` into one file, or split `{name}` into types of their own"
            ),
            [home] => format!(
                "move every `impl {name}` into `{home}`, or split `{name}` into types of their own"
            ),
            _ => format!(
                "move each `impl {name}` beside the declaration it is for, or give the types that \
                 share the name names of their own"
            ),
        }
    }

    fn description(homes: &TypeHomes, name: &str) -> String {
        let declaring = homes.declaring_files(name).len();
        if declaring < 2 {
            let implementing = homes.implementing_files(name);
            return format!(
                "`{name}` is implemented in {} files: {}",
                implementing.len(),
                Self::listed(&implementing)
            );
        }
        let outside = homes.outside_declarations(name);
        let declares = if outside.len() == 1 {
            "declares"
        } else {
            "declare"
        };
        let which = if declaring == 2 {
            "neither"
        } else {
            "none of them"
        };
        format!(
            "`{name}` is declared in {declaring} files and implemented in {} that {declares} \
             {which}: {}",
            outside.len(),
            Self::listed(&outside)
        )
    }

    fn listed(files: &[String]) -> String {
        files
            .iter()
            .map(|file| format!("`{file}`"))
            .collect::<Vec<String>>()
            .join(", ")
    }

    fn offence(homes: &TypeHomes, name: &str) -> Offence {
        Offence::new(
            &homes.anchor_file(name),
            homes.anchor_line(name),
            Self::NAME,
            Self::description(homes, name),
            Self::correction(homes, name),
        )
        .with_subject(name)
    }
}

impl Default for ImplementedInOneFileRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for ImplementedInOneFileRule {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn check(&self, _file: &SourceFile) -> Vec<Offence> {
        Vec::new()
    }

    fn check_workspace(&self, files: &[SourceFile]) -> Vec<Offence> {
        let homes = TypeHomes::of(Self::SOURCE_ROOT, files);
        homes
            .spread()
            .iter()
            .map(|name| Self::offence(&homes, name))
            .collect()
    }

    fn requirement(&self) -> Option<&'static str> {
        None
    }

    fn is_configured(&self) -> bool {
        true
    }

    fn explanation(&self) -> RuleExplanation {
        RuleExplanation {
            name: self.name(),
            summary: "A type's inherent impl lives in one file.",
            breaks: "// src/store.rs\npub struct Store;\n\nimpl Store {\n    pub fn new() -> Self {\n        \
             Self\n    }\n}\n\n// src/store_queries.rs\nimpl Store {\n    pub fn len(&self) -> \
             usize {\n        0\n    }\n}",
            instead: "// src/store.rs\npub struct Store;\n\nimpl Store {\n    pub fn new() -> Self {\n        \
             Self\n    }\n\n    pub fn len(&self) -> usize {\n        0\n    }\n}",
        }
    }
}
