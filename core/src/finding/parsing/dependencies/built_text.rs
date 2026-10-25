// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use quote::ToTokens;
use syn::Expr;
use syn::Path;

// What a method built, named the way an offence names it. A constructor call is
// its type, a unit value or a path its path, a struct literal its type, a call
// its path and a chain its calls -- each without its arguments, so the name stays
// short whatever was passed. `Some`, `Box::new`, `Rc::new` and `Arc::new` only
// carry what they hold, and are left off. Anything else is its source, unspaced.
pub struct BuiltText;

impl BuiltText {
    const CARRIERS: [&'static str; 4] = ["Some", "Box::new", "Rc::new", "Arc::new"];

    pub fn of(value: &Expr) -> String {
        match value {
            Expr::Paren(inner) => Self::of(&inner.expr),
            Expr::Reference(reference) => Self::of(&reference.expr),
            Expr::Call(call) => match call.func.as_ref() {
                Expr::Path(function) => {
                    let path = Self::joined(&function.path);
                    match call.args.first() {
                        Some(carried)
                            if call.args.len() == 1 && Self::CARRIERS.contains(&path.as_str()) =>
                        {
                            Self::of(carried)
                        }
                        _ => path
                            .strip_suffix("::new")
                            .map_or_else(|| format!("{path}()"), str::to_string),
                    }
                }
                _ => Self::unspaced(value),
            },
            Expr::Path(path) => Self::joined(&path.path),
            Expr::Struct(literal) => Self::joined(&literal.path),
            Expr::MethodCall(call) => format!("{}.{}()", Self::of(&call.receiver), call.method),
            Expr::Macro(invocation) => format!("{}![..]", Self::joined(&invocation.mac.path)),
            _ => Self::unspaced(value),
        }
    }

    fn joined(path: &Path) -> String {
        path.segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>()
            .join("::")
    }

    fn unspaced(value: &Expr) -> String {
        value
            .to_token_stream()
            .to_string()
            .split_whitespace()
            .collect()
    }
}
