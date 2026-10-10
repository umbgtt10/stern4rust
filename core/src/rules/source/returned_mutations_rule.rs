// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use syn::FnArg;
use syn::GenericArgument;
use syn::ImplItem;
use syn::Item;
use syn::ItemImpl;
use syn::ItemTrait;
use syn::Pat;
use syn::PathArguments;
use syn::PathSegment;
use syn::Signature;
use syn::TraitItem;
use syn::Type;
use syn::spanned::Spanned;

use crate::finding::parsing::inline_module::InlineModule;
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

    // The signatures whose `&mut` arguments are the author's to remove: free
    // functions, inherent methods and trait declarations. Trait impls are the
    // deliberate omission -- their shape is the trait's.
    fn signatures(items: &[Item]) -> Vec<&Signature> {
        items.iter().flat_map(Self::signatures_in_item).collect()
    }

    fn signatures_in_item(item: &Item) -> Vec<&Signature> {
        match item {
            Item::Fn(function) => vec![&function.sig],
            Item::Impl(block) => Self::signatures_in_impl(block),
            Item::Trait(declaration) => Self::signatures_in_trait(declaration),
            Item::Mod(module) => InlineModule::items(module)
                .map(Self::signatures)
                .unwrap_or_default(),
            _ => Vec::new(),
        }
    }

    fn signatures_in_impl(block: &ItemImpl) -> Vec<&Signature> {
        if block.trait_.is_some() {
            return Vec::new();
        }
        block
            .items
            .iter()
            .filter_map(|item| match item {
                ImplItem::Fn(method) => Some(&method.sig),
                _ => None,
            })
            .collect()
    }

    fn signatures_in_trait(declaration: &ItemTrait) -> Vec<&Signature> {
        declaration
            .items
            .iter()
            .filter_map(|item| match item {
                TraitItem::Fn(method) => Some(&method.sig),
                _ => None,
            })
            .collect()
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

    // The receiver carries no type here, so `&mut self` falls out as a matter of
    // shape rather than a special case.
    fn mutable_parameter(argument: &FnArg) -> Option<(String, usize)> {
        let FnArg::Typed(typed) = argument else {
            return None;
        };
        if !Self::references_mut(&typed.ty) {
            return None;
        }
        Some((
            Self::parameter_name(&typed.pat),
            typed.pat.span().start().line,
        ))
    }

    fn parameter_name(pattern: &Pat) -> String {
        match pattern {
            Pat::Ident(binding) => binding.ident.to_string(),
            _ => "_".to_string(),
        }
    }

    // A mutable reference anywhere a parameter's type structurally contains it is
    // the argument the caller writes through -- but a callback's own contract,
    // written `FnMut(&mut T)` or `fn(&mut T)`, is not descended into.
    fn references_mut(ty: &Type) -> bool {
        match ty {
            Type::Reference(reference) => {
                reference.mutability.is_some() || Self::references_mut(&reference.elem)
            }
            Type::Tuple(tuple) => tuple.elems.iter().any(Self::references_mut),
            Type::Paren(inner) => Self::references_mut(&inner.elem),
            Type::Group(inner) => Self::references_mut(&inner.elem),
            Type::Array(array) => Self::references_mut(&array.elem),
            Type::Slice(slice) => Self::references_mut(&slice.elem),
            Type::Path(path) => path.path.segments.iter().any(Self::segment_references_mut),
            _ => false,
        }
    }

    fn segment_references_mut(segment: &PathSegment) -> bool {
        match &segment.arguments {
            PathArguments::AngleBracketed(arguments) => {
                arguments.args.iter().any(Self::argument_references_mut)
            }
            _ => false,
        }
    }

    fn argument_references_mut(argument: &GenericArgument) -> bool {
        match argument {
            GenericArgument::Type(ty) => Self::references_mut(ty),
            _ => false,
        }
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
        SourceItems::offences(Self::SOURCE_ROOT, file, Self::signatures, Self::offences)
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
            "A function returns its output; it does not write it through a `&mut` parameter.",
            "fn scale(factor: u32, out: &mut Vec<u32>) {\n    out.push(factor);\n}",
            "fn scale(factor: u32) -> Vec<u32> {\n    vec![factor]\n}",
        )
    }
}
