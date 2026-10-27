// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use syn::File;
use syn::Ident;
use syn::parse_file;

use crate::finding::model::time::time_call::TimeCall;
use crate::finding::model::time::time_call_kind::TimeCallKind;
use crate::finding::parsing::declared_item_finder::DeclaredItemFinder;
use crate::finding::parsing::modules::library_target::LibraryTarget;
use crate::finding::parsing::time::imported_names::ImportedNames;
use crate::finding::parsing::time::port_traits::PortTraits;
use crate::finding::parsing::time::production_functions::ProductionFunctions;
use crate::finding::parsing::time::time_call_finder::TimeCallFinder;
use crate::reporting::offence::Offence;
use crate::reporting::rule_explanation::RuleExplanation;
use crate::rule::Rule;
use crate::source_file::SourceFile;

// Time reaches production code through a port.
//
// timeless-tests keeps a test from reading a clock, which leaves code with a
// timeout testable only through a clock it was handed. So in the library target
// a function reading a clock, waiting on a timer or racing one -- the calls
// timeless-tests lists, through the file's imports -- is reported once, at its
// name, naming the first call, unless it implements a port: a trait the package
// declares, or one a dependency does -- a family's core crate declares the ports
// its runtimes implement. That is the adapter a test replaces. The standard
// library's traits are not enough; `impl Display` reading the clock is behind no
// seam.
//
// A binary's files -- `src/main.rs`, `src/bin/`, the modules only they declare --
// are the composition root, where the real clock is chosen, and are left alone,
// as is a package with no library. So are test code and randomness, the other
// input a deterministic core takes from outside, by another port. A workspace
// whose libraries are runtime skips the rule at its root, and the packages meant
// to be time-free state their own skip list.
pub struct InjectedTimeRule;

impl InjectedTimeRule {
    pub const NAME: &'static str = "injected-time";
    pub const SOURCE_ROOT: &'static str = "src/";
    const CORRECTION: &'static str = "declare a trait for the clock or timer it needs, implement it \
                                      with the real one in an adapter and inject it -- or, for a timer \
                                      that only marks a deadline, take the deadline in as an event the \
                                      caller feeds";

    pub fn new() -> Self {
        Self
    }

    fn in_file(file: &SourceFile, syntax: &File, declared: &BTreeSet<String>) -> Vec<Offence> {
        let names = ImportedNames::of(&syntax.items);
        ProductionFunctions::of(&syntax.items, &PortTraits::new(declared, &names))
            .into_iter()
            .filter_map(|(function, body)| {
                TimeCallFinder::new(&names, false)
                    .calls_in(body)
                    .into_iter()
                    .find(|call| call.kind() != TimeCallKind::Random)
                    .map(|call| Self::offence(file, function, &call))
            })
            .collect()
    }

    fn offence(file: &SourceFile, function: &Ident, call: &TimeCall) -> Offence {
        let name = function.to_string();
        Offence::new(
            file.relative_path(),
            function.span().start().line,
            Self::NAME,
            format!(
                "`{name}` {} with `{}`; take time through a port the package declares, so a test can \
                 drive it",
                call.kind().verb(),
                call.written()
            ),
            String::from(Self::CORRECTION),
        )
        .with_subject(&name)
    }
}

impl Default for InjectedTimeRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for InjectedTimeRule {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn check(&self, _file: &SourceFile) -> Vec<Offence> {
        Vec::new()
    }

    fn check_workspace(&self, files: &[SourceFile]) -> Vec<Offence> {
        let parsed: Vec<(&SourceFile, File)> = files
            .iter()
            .filter(|file| file.is_under(Self::SOURCE_ROOT, &[]))
            .filter_map(|file| {
                parse_file(&file.contents())
                    .ok()
                    .map(|syntax| (file, syntax))
            })
            .collect();
        let declared: BTreeSet<String> = parsed
            .iter()
            .flat_map(|(_, syntax)| DeclaredItemFinder::traits(&syntax.items))
            .map(|declared| declared.name().to_string())
            .collect();
        let library = LibraryTarget::of(files);
        parsed
            .iter()
            .filter(|(file, _)| library.contains(file.relative_path()))
            .flat_map(|(file, syntax)| Self::in_file(file, syntax, &declared))
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
            summary: "Time reaches production code through a port.",
            breaks: "pub fn expired(deadline: std::time::Instant) -> bool {\n    std::time::Instant::now() > deadline\n}",
            instead: "pub trait Clock {\n    fn now(&self) -> std::time::Instant;\n}\n\npub fn expired(clock: &dyn Clock, \
             deadline: std::time::Instant) -> bool {\n    clock.now() > deadline\n}",
        }
    }
}
