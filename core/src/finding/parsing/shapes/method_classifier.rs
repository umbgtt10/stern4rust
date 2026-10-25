// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use quote::ToTokens;
use syn::Expr;
use syn::FnArg;
use syn::ImplItemFn;
use syn::Pat;
use syn::Path;
use syn::Receiver;
use syn::ReturnType;
use syn::Stmt;
use syn::Type;
use syn::UnOp;

use crate::finding::parsing::identifiers::Identifiers;
use crate::finding::parsing::shapes::method_shape::MethodShape;

pub struct MethodClassifier;

impl MethodClassifier {
    const FIELD_READS: [&'static str; 9] = [
        "as_deref",
        "as_path",
        "as_ref",
        "as_slice",
        "as_str",
        "clone",
        "to_owned",
        "to_string",
        "to_vec",
    ];

    pub fn shape(method: &ImplItemFn, type_name: &str) -> MethodShape {
        let body = Self::single_expression(method);
        if Self::is_getter(method, body) {
            MethodShape::Getter
        } else if Self::is_plain_constructor(method, body, type_name) {
            MethodShape::PlainConstructor
        } else if Self::is_step(method, body, type_name) {
            MethodShape::Step
        } else if Self::is_build(method, type_name) {
            MethodShape::Build
        } else {
            MethodShape::Behaviour
        }
    }

    fn is_bare_parameter(expression: &Expr, parameters: &[String]) -> bool {
        matches!(expression, Expr::Path(path)
            if path.path.get_ident().is_some_and(|ident| parameters.contains(&ident.to_string())))
    }

    fn is_build(method: &ImplItemFn, type_name: &str) -> bool {
        let ReturnType::Type(_, output) = &method.sig.output else {
            return false;
        };
        let named = Identifiers::of(output.to_token_stream());
        method.sig.ident == "build"
            && Self::receiver(method).is_some_and(|receiver| receiver.reference.is_none())
            && !named.contains("Self")
            && !named.contains(type_name)
    }

    fn is_getter(method: &ImplItemFn, body: Option<&Expr>) -> bool {
        let shared = Self::receiver(method)
            .is_some_and(|receiver| receiver.reference.is_some() && receiver.mutability.is_none());
        shared && method.sig.inputs.len() == 1 && body.is_some_and(Self::reads_one_field)
    }

    fn is_plain_constructor(method: &ImplItemFn, body: Option<&Expr>, type_name: &str) -> bool {
        if Self::receiver(method).is_some() || !Self::returns_own_type(method, type_name) {
            return false;
        }
        let parameters = Self::parameters(method);
        match body {
            Some(Expr::Struct(literal)) => {
                literal.rest.is_none()
                    && Self::names_own_type(&literal.path, type_name)
                    && literal
                        .fields
                        .iter()
                        .all(|field| Self::is_bare_parameter(&field.expr, &parameters))
            }
            Some(Expr::Call(call)) => {
                matches!(&*call.func, Expr::Path(path) if Self::names_own_type(&path.path, type_name))
                    && call
                        .args
                        .iter()
                        .all(|argument| Self::is_bare_parameter(argument, &parameters))
            }
            Some(Expr::Path(path)) => {
                parameters.is_empty() && Self::names_own_type(&path.path, type_name)
            }
            _ => false,
        }
    }

    fn is_step(method: &ImplItemFn, body: Option<&Expr>, type_name: &str) -> bool {
        let by_value = Self::receiver(method).is_some_and(|receiver| receiver.reference.is_none());
        let parameters = Self::parameters(method);
        let Some(Expr::Struct(literal)) = body else {
            return false;
        };
        let rest_is_self = literal
            .rest
            .as_deref()
            .is_some_and(|rest| matches!(rest, Expr::Path(path) if path.path.is_ident("self")));
        by_value
            && parameters.len() == 1
            && rest_is_self
            && Self::names_own_type(&literal.path, type_name)
            && literal.fields.len() == 1
            && literal
                .fields
                .iter()
                .all(|field| Self::is_bare_parameter(&field.expr, &parameters))
    }

    fn names_own_type(path: &Path, type_name: &str) -> bool {
        path.segments.len() == 1 && (path.is_ident("Self") || path.segments[0].ident == type_name)
    }

    fn parameters(method: &ImplItemFn) -> Vec<String> {
        method
            .sig
            .inputs
            .iter()
            .filter_map(|input| match input {
                FnArg::Typed(typed) => match &*typed.pat {
                    Pat::Ident(binding) => Some(binding.ident.to_string()),
                    _ => None,
                },
                FnArg::Receiver(_) => None,
            })
            .collect()
    }

    fn reads_one_field(expression: &Expr) -> bool {
        match expression {
            Expr::Paren(inner) => Self::reads_one_field(&inner.expr),
            Expr::Reference(inner) => {
                inner.mutability.is_none() && Self::reads_one_field(&inner.expr)
            }
            Expr::Unary(inner) => {
                matches!(inner.op, UnOp::Deref(_)) && Self::reads_one_field(&inner.expr)
            }
            Expr::Field(field) => {
                matches!(&*field.base, Expr::Path(path) if path.path.is_ident("self"))
            }
            Expr::MethodCall(call) => {
                call.args.is_empty()
                    && Self::FIELD_READS.contains(&call.method.to_string().as_str())
                    && Self::reads_one_field(&call.receiver)
            }
            _ => false,
        }
    }

    fn receiver(method: &ImplItemFn) -> Option<&Receiver> {
        match method.sig.inputs.first() {
            Some(FnArg::Receiver(receiver)) => Some(receiver),
            _ => None,
        }
    }

    fn returns_own_type(method: &ImplItemFn, type_name: &str) -> bool {
        matches!(&method.sig.output, ReturnType::Type(_, output)
            if matches!(&**output, Type::Path(path) if path.qself.is_none() && Self::names_own_type(&path.path, type_name)))
    }

    fn single_expression(method: &ImplItemFn) -> Option<&Expr> {
        match method.block.stmts.as_slice() {
            [Stmt::Expr(expression, None)] => Some(expression),
            _ => None,
        }
    }
}
