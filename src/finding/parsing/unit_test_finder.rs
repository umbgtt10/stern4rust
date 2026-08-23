// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use proc_macro2::TokenTree;
use quote::ToTokens;
use syn::Attribute;
use syn::Item;
use syn::parse_file;
use syn::spanned::Spanned;

use crate::finding::model::unit_test_site::UnitTestSite;
use crate::finding::parsing::item_naming::ItemNaming;
use crate::source_file::SourceFile;
use proc_macro2::TokenStream;

// Finds tests, and the machinery of tests, in the production source tree.
//
// Three shapes, and all three are about the same thing: code that exists only
// when the tests are being built. A function carrying a test attribute is the
// obvious one. `#[cfg(test)]` is the usual one. `#[cfg_attr(test, ...)]` is the
// third, and it is the one worth spelling out -- a type carrying a derive only
// under test is a type that means one thing to the tests and another to the
// shipped build.
//
// Only the test-gated spelling counts. `#[cfg_attr(feature = "serde", ...)]` is
// ordinary library work and is left alone, as is `#[cfg(feature = "...")]`:
// both gate on something the shipped build can also select.
pub struct UnitTestFinder;

impl UnitTestFinder {
    // None means the file does not parse. readable-source reports that.
    pub fn sites(file: &SourceFile) -> Option<Vec<UnitTestSite>> {
        let syntax = parse_file(&file.contents()).ok()?;
        Some(Self::in_items(file, &syntax.items))
    }

    // Descends into inline modules, because nesting is where a gate is easiest
    // to miss by eye. An item already reported is not descended into: the
    // module is the offence, and listing every test inside it would report the
    // same decision once per test.
    fn in_items(file: &SourceFile, items: &[Item]) -> Vec<UnitTestSite> {
        let mut sites = Vec::new();
        for item in items {
            if let Some(site) = Self::site_of(file, item) {
                sites.push(site);
                continue;
            }
            if let Item::Mod(module) = item {
                if let Some((_, inner)) = &module.content {
                    sites.extend(Self::in_items(file, inner));
                }
            }
        }
        sites
    }

    fn site_of(file: &SourceFile, item: &Item) -> Option<UnitTestSite> {
        let line = item.span().start().line.max(1);
        let attrs = Self::attributes(item);
        let subject = Self::describe(file, item, line);
        let mirror = Self::mirror_of(file.relative_path());

        if attrs.iter().any(Self::is_cfg_attr_test) {
            return Some(UnitTestSite::new(
                line,
                &format!("the `#[cfg_attr(test, ...)]` on the {subject}"),
                &format!(
                    "apply the attribute unconditionally, or move what it guards into {mirror}"
                ),
            ));
        }
        if attrs.iter().any(Self::is_cfg_test) {
            return Some(UnitTestSite::new(
                line,
                &format!("the `#[cfg(test)]` {subject}"),
                &format!("move the tests to {mirror} and delete this from the source tree"),
            ));
        }
        if attrs.iter().any(Self::is_test) {
            return Some(UnitTestSite::new(
                line,
                &format!("the test {subject}"),
                &format!("move the tests to {mirror} and delete this from the source tree"),
            ));
        }
        None
    }

    // Only when the predicate gates on test. `cfg_attr` is how a crate applies
    // a derive behind a feature, which is ordinary library work and none of
    // this rule's business -- what is forbidden is a type that means one thing
    // to the tests and another to the shipped build.
    fn is_cfg_attr_test(attr: &Attribute) -> bool {
        attr.path().is_ident("cfg_attr") && Self::mentions_test(attr)
    }

    // A predicate rather than the literal text, since `any(test, ...)` gates on
    // test just as effectively.
    fn is_cfg_test(attr: &Attribute) -> bool {
        attr.path().is_ident("cfg") && Self::mentions_test(attr)
    }

    // `#[test]`, `#[tokio::test]` and any other harness spelled the same way.
    fn is_test(attr: &Attribute) -> bool {
        attr.path()
            .segments
            .last()
            .is_some_and(|segment| segment.ident == "test")
    }

    // An identifier, not a substring, so `#[cfg(feature = "test")]` is a
    // feature named test rather than a test gate -- the string literal never
    // arrives as an Ident.
    fn mentions_test(attr: &Attribute) -> bool {
        Self::has_test_ident(attr.meta.to_token_stream(), false)
    }

    // Negation flips what a mention means, and reading the ident without the
    // polarity around it gets the answer exactly backwards. `not(test)` gates an
    // item OUT of the test build, which makes it production-only code -- the
    // opposite of what this rule looks for.
    //
    // This used to count a `test` ident anywhere in the predicate, and said in
    // as many words that `not(test)` gated on test "just as effectively".
    // `etheram-embassy` guards its arm-only allocator with
    // `#[cfg(all(not(test), target_arch = "arm"))]`, and every item behind one
    // was reported as test code living in the source tree: eleven offences,
    // none of them real, against code that cannot appear in a test build at all.
    //
    // A `not` applies to the group that follows it, so that group is walked with
    // the polarity flipped while everything beside it keeps the polarity it
    // inherited. Double negation therefore lands back on gating.
    fn has_test_ident(stream: TokenStream, negated: bool) -> bool {
        let mut trees = stream.into_iter().peekable();
        while let Some(tree) = trees.next() {
            match tree {
                TokenTree::Ident(ident) if ident == "not" => {
                    if let Some(TokenTree::Group(group)) =
                        trees.next_if(|next| matches!(next, TokenTree::Group(_)))
                        && Self::has_test_ident(group.stream(), !negated)
                    {
                        return true;
                    }
                }
                TokenTree::Ident(ident) => {
                    if ident == "test" && !negated {
                        return true;
                    }
                }
                TokenTree::Group(group) => {
                    if Self::has_test_ident(group.stream(), negated) {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }

    // src/<path>.rs mirrors onto tests/<path>_tests.rs, which is the same
    // pairing twin4rust enforces, so the correction names the file that should
    // already exist rather than describing a policy.
    fn mirror_of(relative_path: &str) -> String {
        let without_root = relative_path.strip_prefix("src/").unwrap_or(relative_path);
        let stem = without_root.strip_suffix(".rs").unwrap_or(without_root);
        format!("tests/{stem}_tests.rs")
    }

    // The kind words stay here for the same reason they stay in
    // `RegistryParser`: they sit inside offence descriptions, and this one says
    // "module" where that one says "inline module".
    fn describe(file: &SourceFile, item: &Item, line: usize) -> String {
        match (Self::kind(item), ItemNaming::identifier(item)) {
            (Some(kind), Some(subject)) => format!("{kind} `{subject}`"),
            _ => format!("`{}`", ItemNaming::source_line(file, line)),
        }
    }

    fn kind(item: &Item) -> Option<&'static str> {
        match item {
            Item::Const(_) => Some("constant"),
            Item::Enum(_) => Some("enum"),
            Item::Fn(_) => Some("function"),
            Item::Mod(_) => Some("module"),
            Item::Static(_) => Some("static"),
            Item::Struct(_) => Some("struct"),
            Item::Trait(_) => Some("trait"),
            Item::Type(_) => Some("type alias"),
            _ => None,
        }
    }

    fn attributes(item: &Item) -> &[Attribute] {
        match item {
            Item::Const(inner) => &inner.attrs,
            Item::Enum(inner) => &inner.attrs,
            Item::ExternCrate(inner) => &inner.attrs,
            Item::Fn(inner) => &inner.attrs,
            Item::ForeignMod(inner) => &inner.attrs,
            Item::Impl(inner) => &inner.attrs,
            Item::Macro(inner) => &inner.attrs,
            Item::Mod(inner) => &inner.attrs,
            Item::Static(inner) => &inner.attrs,
            Item::Struct(inner) => &inner.attrs,
            Item::Trait(inner) => &inner.attrs,
            Item::TraitAlias(inner) => &inner.attrs,
            Item::Type(inner) => &inner.attrs,
            Item::Union(inner) => &inner.attrs,
            Item::Use(inner) => &inner.attrs,
            _ => &[],
        }
    }
}
