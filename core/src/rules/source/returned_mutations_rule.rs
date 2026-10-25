// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::ptr;

use syn::FnArg;
use syn::ImplItem;
use syn::Item;
use syn::Pat;
use syn::Signature;
use syn::spanned::Spanned;

use crate::finding::parsing::impl_blocks::ImplBlocks;
use crate::finding::parsing::shapes::method_classifier::MethodClassifier;
use crate::finding::parsing::shapes::method_shape::MethodShape;
use crate::finding::parsing::signatures::authored_signatures::AuthoredSignatures;
use crate::finding::parsing::signatures::mutable_reference_search::MutableReferenceSearch;
use crate::finding::parsing::source_items::SourceItems;
use crate::reporting::offence::Offence;
use crate::reporting::rule_explanation::RuleExplanation;
use crate::rule::Rule;
use crate::source_file::SourceFile;

// A function returns its output; it does not write it through a `&mut`
// parameter.
//
// A `&mut T` argument is an output dressed as an input: the caller hands in a
// place for the function to write, and the function's real result is the edit it
// leaves behind rather than the value it returns. The reader of the signature
// cannot tell what comes back without reading the body, and the compiler cannot
// stop the caller passing a half-built value in. Returning the value instead
// makes the output a thing the signature names and the caller owns.
//
// The `&mut self` receiver is left alone. It is the one mutating channel the
// house standard sanctions, and a method that mutates the thing it belongs to is
// the point of a method.
//
// A trait impl is left alone. Its signature is the trait's decision, not the
// implementor's -- `Display::fmt` takes `&mut Formatter` because `Display` says
// so, and no implementor can change that. Where the trait is the author's own,
// the `&mut` is reported at the declaration, which is the one place changing it
// is both possible and enough. Reporting it again at every implementor would
// name an edit that cannot be made there.
//
// A callback that takes a `&mut` describes the callback's contract rather than
// this function's own parameter list, so `impl FnMut(&mut T)` and `fn(&mut T)`
// are left alone.
//
// tests/ is exempt. A test file hands a fake a `&mut` to stand in for a real
// collaborator, and a stand-in shaped that way is what those fakes are for.
pub struct ReturnedMutationsRule;

impl ReturnedMutationsRule {
    pub const SOURCE_ROOT: &'static str = "src/";
    const NAME: &'static str = "returned-mutations";

    pub fn new() -> Self {
        Self
    }

    // The receiver carries no type here, so `&mut self` falls out as a matter of
    // shape rather than a special case.
    fn mutable_parameter(argument: &FnArg) -> Option<(String, usize)> {
        let FnArg::Typed(typed) = argument else {
            return None;
        };
        if !MutableReferenceSearch::new(false).finds_in(&typed.ty) {
            return None;
        }
        Some((
            Self::parameter_name(&typed.pat),
            typed.pat.span().start().line,
        ))
    }

    fn offence(file: &SourceFile, function: &str, parameter: (String, usize)) -> Offence {
        let (name, line) = parameter;
        let subject = format!("{function}::{name}");
        Offence::new(
            file.relative_path(),
            line,
            Self::NAME,
            format!(
                "`{subject}` is a `&mut` parameter, so its result is written through an argument \
                 instead of being returned"
            ),
            format!("return the new value instead of writing it through `&mut {name}`"),
        )
        .with_subject(&subject)
    }

    // Every offending parameter is reported, because turning each one into a
    // return value is a separate edit.
    fn offences(file: &SourceFile, signature: &Signature) -> Vec<Offence> {
        let function = signature.ident.to_string();
        signature
            .inputs
            .iter()
            .filter_map(Self::mutable_parameter)
            .map(|parameter| Self::offence(file, &function, parameter))
            .collect()
    }

    fn parameter_name(pattern: &Pat) -> String {
        match pattern {
            Pat::Ident(binding) => binding.ident.to_string(),
            _ => "_".to_string(),
        }
    }

    fn plain_constructors(items: &[Item]) -> Vec<&Signature> {
        ImplBlocks::inherent(items)
            .into_iter()
            .flat_map(|block| {
                let name = ImplBlocks::target(&block.self_ty).unwrap_or_default();
                block.items.iter().filter_map(move |item| match item {
                    ImplItem::Fn(method)
                        if MethodClassifier::shape(method, &name)
                            == MethodShape::PlainConstructor =>
                    {
                        Some(&method.sig)
                    }
                    _ => None,
                })
            })
            .collect()
    }
}

impl Default for ReturnedMutationsRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for ReturnedMutationsRule {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn check(&self, file: &SourceFile) -> Vec<Offence> {
        let items = SourceItems::under(Self::SOURCE_ROOT, file);
        let plain = Self::plain_constructors(&items);
        AuthoredSignatures::of(&items)
            .into_iter()
            .filter(|signature| !plain.iter().any(|stored| ptr::eq(*stored, *signature)))
            .flat_map(|signature| Self::offences(file, signature))
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
            summary: "A function returns its output; it does not write it through a `&mut` parameter.",
            breaks: "fn scale(factor: u32, out: &mut Vec<u32>) {\n    out.push(factor);\n}",
            instead: "fn scale(factor: u32) -> Vec<u32> {\n    vec![factor]\n}",
        }
    }
}
