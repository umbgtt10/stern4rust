// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use syn::parse_file;

use crate::finding::model::public_entry_point::PublicEntryPoint;
use crate::finding::model::restricted_method::RestrictedMethod;
use crate::finding::parsing::call_site_finder::CallSiteFinder;
use crate::finding::parsing::restricted_method_finder::RestrictedMethodFinder;
use crate::finding::parsing::type_homes::TypeHomes;
use crate::reporting::offence::Offence;
use crate::reporting::rule_explanation::RuleExplanation;
use crate::rule::Rule;
use crate::source_file::SourceFile;

// A function in an inherent impl is `pub` or private.
//
// `fn` is the type's implementation and `pub fn` its contract, which
// tested-public-api holds to a test. `pub(crate)`, `pub(super)`, `pub(self)` and
// `pub(in ...)` are a third state between the gates: not private to the file,
// not an entry point tested-public-api counts, and out of reach of a test in
// tests/, which compiles as another crate.
//
// Each restricted method in an inherent impl under src/ is reported at its
// name. The correction reads the package's call sites, matched on name and
// arity as tested-public-api matches them: private where no other file calls
// it, `pub` with a test where one does -- and, where the type's impl is spread
// over files, implemented-in-one-file, the restriction being what the spread
// needs. `pub(self)` already means private. Trait impls carry no visibility of
// their own, and tests/ is exempt.
pub struct PublicOrPrivateRule;

impl PublicOrPrivateRule {
    pub const NAME: &'static str = "public-or-private";
    pub const SOURCE_ROOT: &'static str = "src/";

    pub fn new() -> Self {
        Self
    }

    fn called_elsewhere(
        calls: &BTreeMap<String, BTreeSet<PublicEntryPoint>>,
        file: &str,
        method: &RestrictedMethod,
    ) -> bool {
        calls
            .iter()
            .any(|(caller, called)| caller != file && called.contains(method.call()))
    }

    fn calls(files: &[SourceFile]) -> BTreeMap<String, BTreeSet<PublicEntryPoint>> {
        files
            .iter()
            .filter(|file| file.relative_path().starts_with(Self::SOURCE_ROOT))
            .filter_map(|file| {
                CallSiteFinder::find(file).map(|called| (file.relative_path().to_string(), called))
            })
            .collect()
    }

    fn correction(
        homes: &TypeHomes,
        calls: &BTreeMap<String, BTreeSet<PublicEntryPoint>>,
        file: &str,
        method: &RestrictedMethod,
    ) -> String {
        if method.is_private_already() {
            return String::from("make it private: `fn`");
        }
        if homes.spread().iter().any(|home| home == method.owner()) {
            return format!(
                "implement `{}` in one file, as `implemented-in-one-file` asks: the restriction \
                 serves its spread over {} files",
                method.owner(),
                homes.implementing_files(method.owner()).len()
            );
        }
        if Self::called_elsewhere(calls, file, method) {
            return String::from(
                "make it `pub` and give it a test, or move the caller's need behind the type's \
                 public contract",
            );
        }
        String::from("make it private: `fn`")
    }

    fn offences_in(
        file: &SourceFile,
        homes: &TypeHomes,
        calls: &BTreeMap<String, BTreeSet<PublicEntryPoint>>,
    ) -> Vec<Offence> {
        let path = file.relative_path();
        if !path.starts_with(Self::SOURCE_ROOT) {
            return Vec::new();
        }
        let Ok(syntax) = parse_file(&file.contents()) else {
            return Vec::new();
        };
        RestrictedMethodFinder::find(&syntax.items)
            .iter()
            .map(|method| {
                Offence::new(
                    path,
                    method.line(),
                    Self::NAME,
                    format!(
                        "`{}` is `{}` in `impl {}`; a method is `pub` or private",
                        method.call().name(),
                        method.visibility(),
                        method.owner()
                    ),
                    Self::correction(homes, calls, path, method),
                )
                .with_subject(method.call().name())
            })
            .collect()
    }
}

impl Default for PublicOrPrivateRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for PublicOrPrivateRule {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn check(&self, _file: &SourceFile) -> Vec<Offence> {
        Vec::new()
    }

    fn check_workspace(&self, files: &[SourceFile]) -> Vec<Offence> {
        let homes = TypeHomes::of(Self::SOURCE_ROOT, files);
        let calls = Self::calls(files);
        files
            .iter()
            .flat_map(|file| Self::offences_in(file, &homes, &calls))
            .collect()
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
            "A function in an inherent impl is `pub` or private.",
            "impl Store {\n    pub(crate) fn grow(&mut self) {\n        self.size += 1;\n    }\n}",
            "impl Store {\n    fn grow(&mut self) {\n        self.size += 1;\n    }\n}",
        )
    }
}
