// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::format;
use alloc::string::ToString;
use alloc::vec::Vec;

use syn::Signature;

use crate::finding::model::repeated_tuple::RepeatedTuple;
use crate::finding::parsing::signatures::authored_signatures::AuthoredSignatures;
use crate::finding::parsing::signatures::repeated_tuple_finder::RepeatedTupleFinder;
use crate::finding::parsing::source_items::SourceItems;
use crate::reporting::offence::Offence;
use crate::reporting::rule_explanation::RuleExplanation;
use crate::rule::Rule;
use crate::source_file::SourceFile;

// A tuple in a return type repeats no element type.
//
// `-> (u64, u64)` returns two numbers and says nothing about which is which, and
// a caller that swaps them still compiles. Most tuples are not like that -- in
// `(PathBuf, u64)` the types say which element is which and a swap does not
// compile -- so the rule asks only about the narrow case: two elements of one
// type, where position is all that tells them apart. A struct with named fields
// says it instead.
//
// The tuple is found wherever it sits in the return type -- at the top, inside a
// generic, behind a reference -- in a private function as much as a public one,
// since a private function's callers can swap its results as easily. Types are
// compared as written, lifetimes ignored and nothing resolved, so an alias that
// names a meaning is the cure rather than a loophole. A trait impl is left alone,
// its signature being the trait's; a returned callback's parameter list is its
// contract, not a tuple.
//
// tests/ is exempt.
pub struct DistinctTupleElementsRule;

impl DistinctTupleElementsRule {
    pub const SOURCE_ROOT: &'static str = "src/";
    const NAME: &'static str = "distinct-tuple-elements";

    pub fn new() -> Self {
        Self
    }

    fn offence(file: &SourceFile, signature: &Signature, found: RepeatedTuple) -> Offence {
        let function = signature.ident.to_string();
        Offence::new(
            file.relative_path(),
            signature.ident.span().start().line,
            Self::NAME,
            format!(
                "`{function}` returns `{}`, whose `{}`s only their position tells apart",
                found.tuple, found.element
            ),
            format!(
                "return a struct whose fields say which `{}` is which",
                found.element
            ),
        )
        .with_subject(&function)
    }
}

impl Default for DistinctTupleElementsRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for DistinctTupleElementsRule {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn check(&self, file: &SourceFile) -> Vec<Offence> {
        SourceItems::in_returns(
            Self::SOURCE_ROOT,
            file,
            AuthoredSignatures::of,
            RepeatedTupleFinder::first_in,
            Self::offence,
        )
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
            summary: "A tuple in a return type repeats no element type.",
            breaks: "fn bounds(values: &[u64]) -> (u64, u64) {\n    (values[0], values[1])\n}",
            instead: "struct Bounds {\n    low: u64,\n    high: u64,\n}\n\nfn bounds(values: &[u64]) -> Bounds {\n    \
             Bounds { low: values[0], high: values[1] }\n}",
        }
    }
}
