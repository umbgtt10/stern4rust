// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use syn::Signature;

use crate::finding::parsing::modules::library_target::LibraryTarget;
use crate::finding::parsing::signatures::authored_signatures::AuthoredSignatures;
use crate::finding::parsing::signatures::stringly_error_finder::StringlyErrorFinder;
use crate::finding::parsing::source_items::SourceItems;
use crate::reporting::offence::Offence;
use crate::reporting::rule_explanation::RuleExplanation;
use crate::rule::Rule;
use crate::source_file::SourceFile;

// A library's public functions fail with an error type of their own.
//
// `Result<T, String>` makes a failure a sentence: a caller can print it and do
// nothing else, and a test pinning a failure path pins its wording. `&str` and
// `Box<dyn Error>` are the same in different clothes. An error type of the
// crate's own lets a test match the failure it means and a caller handle one and
// pass the rest -- which matters most in a library, whose callers are code.
//
// A public function of the library target -- a `pub fn`, a `pub` method of an
// inherent impl, a method of a `pub` trait, reported at the trait -- whose return
// type holds a `Result` failing with a `String`, a `&str` or a boxed
// `dyn ...Error` is reported at its name. A binary's files, `src/main.rs`,
// `src/bin/` and what only they declare, end on a terminal and are left alone;
// trait impls are the trait's; a package that is a library only in form -- a
// harness, an `xtask` -- skips the rule in `stern4rust.toml`. tests/ is exempt.
pub struct TypedErrorsRule;

impl TypedErrorsRule {
    pub const NAME: &'static str = "typed-errors";
    pub const SOURCE_ROOT: &'static str = "src/";

    pub fn new() -> Self {
        Self
    }

    fn offence(file: &SourceFile, signature: &Signature, error: String) -> Offence {
        let function = signature.ident.to_string();
        Offence::new(
            file.relative_path(),
            signature.ident.span().start().line,
            Self::NAME,
            format!("`{function}` fails with a `{error}`"),
            String::from("give the crate an error enum its callers and tests can match"),
        )
        .with_subject(&function)
    }
}

impl Default for TypedErrorsRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for TypedErrorsRule {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn check(&self, _file: &SourceFile) -> Vec<Offence> {
        Vec::new()
    }

    fn check_workspace(&self, files: &[SourceFile]) -> Vec<Offence> {
        let library = LibraryTarget::of(files);
        files
            .iter()
            .filter(|file| library.contains(file.relative_path()))
            .flat_map(|file| {
                SourceItems::in_returns(
                    Self::SOURCE_ROOT,
                    file,
                    AuthoredSignatures::public,
                    StringlyErrorFinder::first_in,
                    Self::offence,
                )
            })
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
            "A library's public functions fail with an error type of their own.",
            "pub fn read(path: &str) -> Result<u8, String> {\n    Err(format!(\"cannot read {path}\"))\n}",
            "pub enum ReadError {\n    Missing,\n}\n\npub fn read(path: &str) -> Result<u8, ReadError> {\n    \
             Err(ReadError::Missing)\n}",
        )
    }
}
