// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::string::ToString;

use syn::Attribute;
use syn::Expr;
use syn::ExprLit;
use syn::Item;
use syn::Lit;
use syn::Macro;
use syn::Meta;
use syn::PatIdent;
use syn::Path;
use syn::Signature;
use syn::Token;
use syn::Type;
use syn::UseName;
use syn::UseRename;
use syn::parse_file;
use syn::punctuated::Punctuated;
use syn::visit::Visit;
use syn::visit::visit_attribute;
use syn::visit::visit_pat_ident;
use syn::visit::visit_path;

use crate::finding::parsing::identifiers::Identifiers;
use crate::source_file::SourceFile;

// The names a file reads: every segment of every path it writes -- in an
// expression, a type, an attribute --, every name it imports, every bare name a
// pattern matches, which is how a `match` reads a constant, and every identifier
// in a macro's tokens, which is where an assertion reads one, and every path a
// serde attribute spells as a string, which is how `#[serde(with = "...")]`
// reads its mirror. The names it declares are not paths, so a file declaring a
// constant of its own does not read another file's of the same name.
pub struct ReadNames {
    names: BTreeSet<String>,
}

impl ReadNames {
    const SERDE: &'static str = "serde";
    const SPELLED_PATHS: [&'static str; 10] = [
        "default",
        "deserialize_with",
        "from",
        "getter",
        "into",
        "remote",
        "serialize_with",
        "skip_serializing_if",
        "try_from",
        "with",
    ];

    fn empty() -> Self {
        Self {
            names: BTreeSet::new(),
        }
    }

    // Each name these items read, for a question about part of a file.
    pub fn in_items<'a>(items: impl IntoIterator<Item = &'a Item>) -> BTreeSet<String> {
        items
            .into_iter()
            .fold(Self::empty(), |mut reader, item| {
                reader.visit_item(item);
                reader
            })
            .names
    }

    // Each name a signature reads: its parameters' and return types, its generics.
    pub fn in_signature(signature: &Signature) -> BTreeSet<String> {
        let mut reader = Self::empty();
        reader.visit_signature(signature);
        reader.names
    }

    pub fn in_type(declared: &Type) -> BTreeSet<String> {
        let mut reader = Self::empty();
        reader.visit_type(declared);
        reader.names
    }

    // Each name any of these files reads, with the paths of the files reading
    // it, as the files name themselves.
    pub fn index(files: &[SourceFile]) -> BTreeMap<String, BTreeSet<String>> {
        let mut index: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for file in files {
            for name in Self::of(file) {
                index
                    .entry(name)
                    .or_default()
                    .insert(file.relative_path().to_string());
            }
        }
        index
    }

    pub fn of(file: &SourceFile) -> BTreeSet<String> {
        let Ok(syntax) = parse_file(&file.contents()) else {
            return BTreeSet::new();
        };
        let mut reader = Self::empty();
        reader.visit_file(&syntax);
        reader.names
    }

    fn spelled_type(meta: Meta) -> Option<Type> {
        let Meta::NameValue(pair) = meta else {
            return None;
        };
        let Expr::Lit(ExprLit {
            lit: Lit::Str(spelled),
            ..
        }) = pair.value
        else {
            return None;
        };
        Self::SPELLED_PATHS
            .iter()
            .any(|key| pair.path.is_ident(key))
            .then(|| spelled.parse().ok())
            .flatten()
    }
}

impl<'ast> Visit<'ast> for ReadNames {
    fn visit_attribute(&mut self, attribute: &'ast Attribute) {
        visit_attribute(self, attribute);
        if !attribute.path().is_ident(Self::SERDE) {
            return;
        }
        let spelled = attribute
            .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
            .into_iter()
            .flatten()
            .filter_map(Self::spelled_type);
        for declared in spelled {
            self.visit_type(&declared);
        }
    }

    fn visit_macro(&mut self, mac: &'ast Macro) {
        self.visit_path(&mac.path);
        self.names.extend(Identifiers::of(mac.tokens.clone()));
    }

    fn visit_pat_ident(&mut self, pattern: &'ast PatIdent) {
        self.names.insert(pattern.ident.to_string());
        visit_pat_ident(self, pattern);
    }

    fn visit_path(&mut self, path: &'ast Path) {
        self.names.extend(
            path.segments
                .iter()
                .map(|segment| segment.ident.to_string()),
        );
        visit_path(self, path);
    }

    fn visit_use_name(&mut self, name: &'ast UseName) {
        self.names.insert(name.ident.to_string());
    }

    fn visit_use_rename(&mut self, rename: &'ast UseRename) {
        self.names.insert(rename.ident.to_string());
    }
}
