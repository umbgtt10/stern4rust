// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use syn::ImplItem;
use syn::ImplItemFn;
use syn::ItemImpl;
use syn::ReturnType;
use syn::Type;
use syn::Visibility;
use syn::ext::IdentExt;
use syn::parse_file;
use syn::spanned::Spanned;

use crate::finding::model::layout::fix_refusal::FixRefusal;
use crate::finding::model::layout::impl_layout::ImplLayout;
use crate::finding::model::layout::impl_member::ImplMember;
use crate::finding::model::layout::member_kind::MemberKind;
use crate::finding::parsing::impl_blocks::ImplBlocks;
use crate::finding::parsing::item_naming::ItemNaming;
use crate::finding::parsing::signatures::type_walk::TypeWalk;
use crate::source_file::SourceFile;

// Each inherent impl block of a file, inline modules included, and its members
// as written, with the lines each would move with.
//
// A member is a constant, a type or a function. A constructor is a function
// with no receiver whose return type names `Self` or the impl's type, directly
// or wrapped -- `Result<Self, E>`, `Option<Store>`, `(Self, Vec<u8>)` -- whatever
// its visibility. A `pub` function is any written `pub`, `const`, `async` or
// `unsafe` alike; everything else is the rest. A macro invocation is no member:
// its expansion cannot be seen.
//
// Whether --fix could reorder the block is decided here, as a fact about it: a
// macro among the items, two members on one line or on a brace's, or a comment
// standing free between two members, which has no item to travel with.
pub struct ImplLayoutParser;

impl ImplLayoutParser {
    pub fn is_constructor(method: &ImplItemFn, owner: &str) -> bool {
        method.sig.receiver().is_none() && Self::returns(&method.sig.output, owner)
    }

    pub fn parse(file: &SourceFile) -> Vec<ImplLayout> {
        let Ok(syntax) = parse_file(&file.contents()) else {
            return Vec::new();
        };
        ImplBlocks::inherent(&syntax.items)
            .into_iter()
            .map(|block| Self::layout(file, block))
            .collect()
    }

    fn crowded(members: &[ImplMember], open_line: usize, close_line: usize) -> bool {
        let edges = members
            .first()
            .is_some_and(|first| first.first_line() <= open_line)
            || members
                .last()
                .is_some_and(|last| last.last_line() >= close_line);
        edges
            || members
                .windows(2)
                .any(|pair| pair[1].first_line() <= pair[0].last_line())
    }

    fn free_comment(file: &SourceFile, members: &[ImplMember]) -> bool {
        members.windows(2).any(|pair| {
            (pair[0].last_line() + 1..pair[1].first_line()).any(|line| {
                file.lines()
                    .get(line - 1)
                    .is_some_and(|text| !text.trim().is_empty())
            })
        })
    }

    fn kind(method: &ImplItemFn, owner: &str) -> MemberKind {
        if Self::is_constructor(method, owner) {
            MemberKind::Constructor
        } else if matches!(method.vis, Visibility::Public(_)) {
            MemberKind::Public
        } else {
            MemberKind::Other
        }
    }

    fn layout(file: &SourceFile, block: &ItemImpl) -> ImplLayout {
        let owner = ItemNaming::implemented_type(&block.self_ty);
        let open_line = block.brace_token.span.open().start().line;
        let close_line = block.brace_token.span.close().start().line;
        let mut floor = open_line + 1;
        let mut members = Vec::new();
        for item in &block.items {
            let first_line = file.block_start(item.span().start().line, floor);
            let last_line = item.span().end().line;
            members.extend(Self::member(item, &owner, first_line, last_line));
            floor = last_line + 1;
        }
        let refusal = Self::refusal(file, block, &members, open_line, close_line);
        ImplLayout::new(
            owner,
            block.impl_token.span.start().line,
            open_line,
            close_line,
            members,
            refusal,
        )
    }

    fn member(
        item: &ImplItem,
        owner: &str,
        first_line: usize,
        last_line: usize,
    ) -> Option<ImplMember> {
        let (name, kind, parameters) = match item {
            ImplItem::Const(inner) => (inner.ident.unraw().to_string(), MemberKind::Head, 0),
            ImplItem::Type(inner) => (inner.ident.unraw().to_string(), MemberKind::Head, 0),
            ImplItem::Fn(method) => (
                method.sig.ident.unraw().to_string(),
                Self::kind(method, owner),
                method.sig.inputs.len(),
            ),
            _ => return None,
        };
        Some(ImplMember::new(
            name, kind, parameters, first_line, last_line,
        ))
    }

    fn names(ty: &Type, owner: &str) -> bool {
        match ty {
            Type::Path(path)
                if path
                    .path
                    .segments
                    .last()
                    .is_some_and(|segment| segment.ident == "Self" || segment.ident == owner) =>
            {
                true
            }
            _ => TypeWalk::children(ty)
                .into_iter()
                .any(|child| Self::names(child, owner)),
        }
    }

    fn refusal(
        file: &SourceFile,
        block: &ItemImpl,
        members: &[ImplMember],
        open_line: usize,
        close_line: usize,
    ) -> Option<FixRefusal> {
        if block
            .items
            .iter()
            .any(|item| matches!(item, ImplItem::Macro(_)))
        {
            return Some(FixRefusal::MacroInvocation);
        }
        if Self::crowded(members, open_line, close_line) {
            return Some(FixRefusal::SharedLine);
        }
        Self::free_comment(file, members).then_some(FixRefusal::FreeComment)
    }

    fn returns(output: &ReturnType, owner: &str) -> bool {
        match output {
            ReturnType::Type(_, ty) => Self::names(ty, owner),
            ReturnType::Default => false,
        }
    }
}
