// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use syn::ReturnType;
use syn::Signature;
use syn::Type;

use crate::finding::parsing::signatures::authored_signatures::AuthoredSignatures;
use crate::finding::parsing::signatures::mutable_reference_search::MutableReferenceSearch;
use crate::finding::parsing::source_items::SourceItems;
use crate::reporting::offence::Offence;
use crate::reporting::rule_explanation::RuleExplanation;
use crate::rule::Rule;
use crate::source_file::SourceFile;

// A function does not hand out a mutable reference.
//
// `&mut self` is the one channel through which the house standard lets a type
// change. A function returning `&mut T` -- or a type holding one -- opens a
// second, owned by whoever holds the reference: the caller changes the type's
// internals without going through any of its methods, and every invariant the
// type keeps is kept only while nobody uses what it lent. `returned-mutations`
// stops a mutation coming in through a parameter; this stops one going out.
//
// `&'static mut` is left alone. It is a static handed over once -- the shape an
// embedded executor needs for its resources -- with no type behind it to lend.
// A trait impl is left alone, its signature being the trait's; a trait of the
// author's own is reported at its declaration. A returned callback's contract,
// `impl FnMut(&mut T)`, says what the caller's closure receives, not what this
// function lends, and is not descended into.
//
// tests/ is exempt.
pub struct ImmutableReturnsRule;

impl ImmutableReturnsRule {
    pub const SOURCE_ROOT: &'static str = "src/";
    const NAME: &'static str = "immutable-returns";

    pub fn new() -> Self {
        Self
    }

    // A `&mut Self` chain keeps a mutable borrow across every call; taking and
    // returning `self` keeps the chain and drops the loan.
    fn correction(output: &Type) -> &'static str {
        if Self::is_mut_self(output) {
            "take and return `self` by value instead of `&mut Self`"
        } else {
            "return the value, or give the type a method that makes the change"
        }
    }

    fn is_mut_self(output: &Type) -> bool {
        match output {
            Type::Reference(reference) if reference.mutability.is_some() => {
                matches!(&*reference.elem, Type::Path(path) if path.path.is_ident("Self"))
            }
            _ => false,
        }
    }

    fn offences(file: &SourceFile, signature: &Signature) -> Vec<Offence> {
        let ReturnType::Type(_, output) = &signature.output else {
            return Vec::new();
        };
        if !MutableReferenceSearch::new(true).finds_in(output) {
            return Vec::new();
        }
        let function = signature.ident.to_string();
        vec![
            Offence::new(
                file.relative_path(),
                signature.ident.span().start().line,
                Self::NAME,
                format!(
                    "`{function}` returns a mutable reference, so its caller can change what it \
                     lends without going through its methods"
                ),
                Self::correction(output).to_string(),
            )
            .with_subject(&function),
        ]
    }
}

impl Default for ImmutableReturnsRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for ImmutableReturnsRule {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn check(&self, file: &SourceFile) -> Vec<Offence> {
        SourceItems::offences(
            Self::SOURCE_ROOT,
            file,
            AuthoredSignatures::of,
            Self::offences,
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
        RuleExplanation::new(
            self.name(),
            "A function does not hand out a mutable reference.",
            "impl Meter {\n    pub fn total_mut(&mut self) -> &mut u64 {\n        &mut self.total\n    }\n}",
            "impl Meter {\n    pub fn add(&mut self, amount: u64) {\n        self.total += amount;\n    }\n}",
        )
    }
}
