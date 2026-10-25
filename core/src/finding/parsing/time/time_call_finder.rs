// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use syn::Block;
use syn::Expr;
use syn::ExprCall;
use syn::ExprMethodCall;
use syn::ExprPath;
use syn::Macro;
use syn::Token;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::visit::visit_expr_method_call;
use syn::visit::visit_expr_path;

use crate::finding::model::time::time_call::TimeCall;
use crate::finding::model::time::time_call_kind::TimeCallKind;
use crate::finding::parsing::time::imported_names::ImportedNames;

// The first call in a body through which it depends on time: a clock read -- `Instant::now` and its kin, or an
// `elapsed` read off one, which reads `now` inside --, a sleep, a timeout raced, or a generator nobody seeded. A path
// is matched once resolved through the file's imports, so a function of the file's own that happens to be called
// `sleep` is not one; a string spelling a call is not code. On Tokio's paused clock, `tokio::time` is virtual and let
// through, and the readings off its `Instant` with it; the system's clock is not. A macro's arguments are read where
// they parse as expressions -- `assert!`, `format!` and their kin --, since that is where a test puts its stopwatch.
pub struct TimeCallFinder<'a> {
    names: &'a ImportedNames,
    paused: bool,
    calls: Vec<TimeCall>,
}

impl<'a> TimeCallFinder<'a> {
    const CLOCKS: [&'static str; 10] = [
        "std::time::Instant::now",
        "std::time::SystemTime::now",
        "chrono::Utc::now",
        "chrono::Local::now",
        "chrono::offset::Utc::now",
        "chrono::offset::Local::now",
        "time::OffsetDateTime::now_utc",
        "time::OffsetDateTime::now_local",
        "tokio::time::Instant::now",
        "embassy_time::Instant::now",
    ];
    const PAUSED_CLOCK: &'static str = "tokio::time::";
    const RACES: [&'static str; 4] = [
        "tokio::time::timeout",
        "tokio::time::timeout_at",
        "embassy_time::with_timeout",
        "embassy_time::with_deadline",
    ];
    const RANDOMS: [&'static str; 5] = [
        "rand::thread_rng",
        "rand::rng",
        "rand::random",
        "rand::rngs::OsRng",
        "rand_core::OsRng",
    ];
    const READING: &'static str = "elapsed";
    const WAITS: [&'static str; 6] = [
        "std::thread::sleep",
        "tokio::time::sleep",
        "tokio::time::sleep_until",
        "async_std::task::sleep",
        "embassy_time::Timer::after",
        "embassy_time::Timer::at",
    ];

    pub fn new(names: &'a ImportedNames, paused: bool) -> Self {
        Self {
            names,
            paused,
            calls: Vec::new(),
        }
    }

    // Every call, in the order written: a rule that leaves one kind alone picks among them.
    pub fn calls_in(mut self, body: &Block) -> Vec<TimeCall> {
        self.visit_block(body);
        let mut calls = self.calls;
        calls.sort_by_key(|call| (call.line(), call.column()));
        calls
    }

    pub fn first_in(self, body: &Block) -> Option<TimeCall> {
        self.calls_in(body).into_iter().next()
    }

    fn kind_of(&self, full: &str) -> Option<TimeCallKind> {
        if self.paused && full.starts_with(Self::PAUSED_CLOCK) {
            return None;
        }
        [
            (Self::CLOCKS.as_slice(), TimeCallKind::Clock),
            (Self::WAITS.as_slice(), TimeCallKind::Wait),
            (Self::RACES.as_slice(), TimeCallKind::Race),
            (Self::RANDOMS.as_slice(), TimeCallKind::Random),
        ]
        .into_iter()
        .find_map(|(catalogue, kind)| catalogue.contains(&full).then_some(kind))
    }

    // A bare name that is not called is a value -- a binding, or a field written
    // in shorthand -- and reads no time, whatever the file imports under that
    // name; a generator is the one value that does.
    fn record(&mut self, node: &ExprPath, called: bool) {
        let written: Vec<String> = node
            .path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect();
        let found = self
            .names
            .resolve(&written)
            .iter()
            .find_map(|full| self.kind_of(full))
            .filter(|kind| called || written.len() > 1 || *kind == TimeCallKind::Random);
        if let Some(kind) = found {
            let start = node.span().start();
            self.calls.push(TimeCall::new(
                kind,
                &written.join("::"),
                start.line,
                start.column,
            ));
        }
    }
}

impl<'ast> Visit<'ast> for TimeCallFinder<'_> {
    fn visit_expr_call(&mut self, node: &'ast ExprCall) {
        match node.func.as_ref() {
            Expr::Path(callee) => {
                self.record(callee, true);
                visit_expr_path(self, callee);
            }
            callee => self.visit_expr(callee),
        }
        for argument in &node.args {
            self.visit_expr(argument);
        }
    }

    fn visit_expr_path(&mut self, node: &'ast ExprPath) {
        self.record(node, false);
        visit_expr_path(self, node);
    }

    fn visit_macro(&mut self, node: &'ast Macro) {
        if let Ok(arguments) = node.parse_body_with(Punctuated::<Expr, Token![,]>::parse_terminated)
        {
            for argument in &arguments {
                self.visit_expr(argument);
            }
        }
    }

    fn visit_expr_method_call(&mut self, node: &'ast ExprMethodCall) {
        if !self.paused && node.method == Self::READING && node.args.is_empty() {
            let start = node.method.span().start();
            self.calls.push(TimeCall::new(
                TimeCallKind::Clock,
                Self::READING,
                start.line,
                start.column,
            ));
        }
        visit_expr_method_call(self, node);
    }
}
