// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

pub mod call_site_finder;
pub mod constant_layout_parser;
pub mod declared_item_finder;
pub mod free_function_finder;
pub mod gated_modules;
pub mod identifiers;
pub mod impl_blocks;
pub mod implemented_type_finder;
pub mod inline_module;
pub mod item_naming;
pub mod module_declaration_finder;
pub mod public_entry_point_finder;
pub mod qualified_call_finder;
pub mod registry_parser;
pub mod restricted_method_finder;
pub mod signatures;
pub mod source_items;
pub mod test_file_parser;
pub mod type_homes;
pub mod unit_test_finder;
