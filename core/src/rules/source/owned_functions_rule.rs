// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use syn::Item;
use syn::parse_file;

use crate::finding::model::file_stem::FileStem;
use crate::finding::model::free_function::FreeFunction;
use crate::finding::parsing::declared_item_finder::DeclaredItemFinder;
use crate::finding::parsing::free_function_finder::FreeFunctionFinder;
use crate::finding::parsing::impl_blocks::ImplBlocks;
use crate::finding::parsing::implemented_type_finder::ImplementedTypeFinder;
use crate::reporting::offence::Offence;
use crate::reporting::rule_explanation::RuleExplanation;
use crate::rule::Rule;
use crate::source_file::SourceFile;

// A file with a subject owns its functions.
//
// single-implemented-type gives a file one subject and named-subject names the
// file after it; neither sees a free function standing beside it. That function
// is a second subject without a name, and usually the residue of an impl that
// grew: a helper pulled out to quiet crap4rust instead of a struct being
// extracted. The house standard prefers named structs with methods over free
// functions; this rule holds the files that have a subject to it.
//
// A file has a subject when it declares a type or a trait, or holds an impl
// block. The subject is the implemented type named for the file, else the first;
// else the type an impl block is for; else a declared type, then a trait. A file
// of functions alone has no subject to compete with and is left alone -- moving a
// helper into such a file separates it, which is the point.
//
// The correction reads the file: a function the impl calls, or whose body names
// the subject, is offered the impl; one that touches neither is offered a file of
// its own first. Beside a trait only a file of its own is offered, since holding
// the function would take a default body and pure-traits forbids one.
//
// tests/ is exempt, where helper functions are the shape test-file-structure asks
// for; so are lib.rs and mod.rs, whose functions module-registry already reports,
// and build.rs, a Cargo script. In an entry file `main` is free by definition.
pub struct OwnedFunctionsRule;

impl OwnedFunctionsRule {
    const TESTS_ROOT: &'static str = "tests/";
    const EXEMPT_FILES: [&'static str; 3] = ["lib.rs", "mod.rs", "build.rs"];
    const ENTRY_FILE: &'static str = "main.rs";
    const ENTRY_ROOTS: [&'static str; 3] = ["src/bin/", "examples/", "benches/"];

    pub fn new() -> Self {
        Self
    }

    fn applies_to(file: &SourceFile) -> bool {
        !file.relative_path().starts_with(Self::TESTS_ROOT)
            && !Self::EXEMPT_FILES.contains(&file.file_name())
    }

    fn correction(
        function: &FreeFunction,
        subject: &str,
        ownable: bool,
        impls: &ImplBlocks,
    ) -> String {
        let name = &function.name();
        if !ownable {
            return format!("move `{name}` into a file of its own");
        }
        if impls.names(name) || function.names(subject) {
            return format!("move `{name}` into `impl {subject}` as an associated function");
        }
        format!(
            "move `{name}` into a file of its own, or into `impl {subject}` if it belongs there"
        )
    }

    fn is_entry_file(file: &SourceFile) -> bool {
        file.file_name() == Self::ENTRY_FILE
            || Self::ENTRY_ROOTS
                .iter()
                .any(|root| file.relative_path().starts_with(root))
    }

    fn named(stem: &FileStem, names: &[String]) -> Option<String> {
        names
            .iter()
            .find(|name| stem.names(name))
            .or_else(|| names.first())
            .cloned()
    }

    // The subject, and whether it can take a function: a type can, through an
    // impl block; a trait cannot.
    fn subject(file: &SourceFile, items: &[Item], impls: &ImplBlocks) -> Option<(String, bool)> {
        let stem = FileStem::of(file.relative_path());
        let implemented: Vec<String> = ImplementedTypeFinder::among(items)
            .into_iter()
            .map(|found| found.name().to_string())
            .collect();
        let declared: Vec<String> = DeclaredItemFinder::types(items)
            .into_iter()
            .map(|found| found.name().to_string())
            .collect();
        let traits = DeclaredItemFinder::traits(items);
        Self::named(&stem, &implemented)
            .or_else(|| impls.targets().first().cloned())
            .or_else(|| Self::named(&stem, &declared))
            .map(|name| (name, true))
            .or_else(|| {
                traits
                    .first()
                    .map(|found| (found.name().to_string(), false))
            })
    }
}

impl Default for OwnedFunctionsRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for OwnedFunctionsRule {
    fn name(&self) -> &'static str {
        "owned-functions"
    }

    fn check(&self, file: &SourceFile) -> Vec<Offence> {
        if !Self::applies_to(file) {
            return Vec::new();
        }
        let Ok(syntax) = parse_file(&file.contents()) else {
            return Vec::new();
        };
        let impls = ImplBlocks::of(&syntax.items);
        let Some((subject, ownable)) = Self::subject(file, &syntax.items, &impls) else {
            return Vec::new();
        };
        FreeFunctionFinder::among(&syntax.items, Self::is_entry_file(file))
            .iter()
            .map(|function| {
                Offence::new(
                    file.relative_path(),
                    function.line(),
                    self.name(),
                    format!(
                        "`{}` is a free function in a file whose subject is `{subject}`",
                        function.name()
                    ),
                    Self::correction(function, &subject, ownable, &impls),
                )
                .with_subject(function.name())
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
        RuleExplanation {
            name: self.name(),
            summary: "A file with a subject owns its functions: none stands free beside a type, a trait or an impl block.",
            breaks: "pub struct Widget;\nimpl Widget {\n    pub fn area(&self) -> u32 { square(2) }\n}\n\nfn square(side: u32) -> u32 { side * side }",
            instead: "pub struct Widget;\nimpl Widget {\n    pub fn area(&self) -> u32 { Self::square(2) }\n\n    fn square(side: u32) -> u32 { side * side }\n}",
        }
    }
}
