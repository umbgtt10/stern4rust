// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

pub mod args;

pub mod baseline;

pub mod baseline_outcome;

pub mod column_widths;

pub mod config;

pub mod config_file;

pub mod exclusion;

pub mod exclusion_outcome;

pub mod exclusion_set;

pub mod header_source;

pub mod implemented_type;

pub mod implemented_type_finder;

pub mod import_path;

pub mod json_printer;

pub mod manifest_resolver;

pub mod module_declaration_finder;

pub mod offence;

pub mod offence_fingerprint;

pub mod offence_threshold;

pub mod output_format;

pub mod package_tree;

pub mod qualified_call;

pub mod qualified_call_finder;

pub mod registry_item;

pub mod registry_parser;

pub mod registry_policy;

pub mod report_printer;

pub mod rule;

pub mod rule_registry;

pub mod rule_selection;

pub mod rules;

pub mod run_outcome;

pub mod runner;

pub mod section;

pub mod source_file;

pub mod source_reader;

pub mod source_walker;

pub mod test_file_item;

pub mod test_file_parser;

pub mod test_file_rewriter;

pub mod unit_test_finder;

pub mod unit_test_site;
