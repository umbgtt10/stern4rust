// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A lib.rs or mod.rs under src/ is a list of the modules beneath it and nothing
// else: the header, the crate's inner attributes, `extern crate alloc;`, and
// `pub mod` declarations.
//
// The file that names a crate's shape should be readable in one glance. A `use`
// here is a re-export shim wearing a registry's clothes; a `fn` here is code in
// the one file nobody opens expecting code; an inline `mod name { ... }` is a
// module that no longer has a file to be found in.
//
// Inner attributes never reach the item list -- syn keeps `#![no_std]` on the
// file rather than among its items -- so they are allowed without the rule
// having to know they exist.

use stern4rust::reporting::offence::Offence;
use stern4rust::rule::Rule;
use stern4rust::rules::layout::module_registry_rule::ModuleRegistryRule;
use stern4rust::source_file::SourceFile;

const HEADER: &str = "// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>\n\
                      // Licensed under the MIT License\n\
                      // SPDX-License-Identifier: MIT\n";

const RULE: &str = "module-registry";

fn check(path: &str, body: &str) -> Vec<Offence> {
    ModuleRegistryRule::new().check(&SourceFile::new(path, &format!("{HEADER}\n{body}")))
}

#[test]
fn check_a_lib_file_of_only_declarations_reports_nothing() {
    // Arrange & Act
    let offences = check("src/lib.rs", "pub mod alpha;\npub mod beta;\n");

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// syn keeps inner attributes on the file, not among its items, so a no_std
// crate root passes without the rule enumerating attribute names.
#[test]
fn check_a_lib_file_with_inner_attributes_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/lib.rs",
        "#![no_std]\n#![forbid(unsafe_code)]\n\npub mod alpha;\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_a_mod_file_of_only_declarations_reports_nothing() {
    // Arrange & Act
    let offences = check("src/node_common/mod.rs", "pub mod alpha;\npub mod beta;\n");

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// A no_std crate has to say this somewhere and the crate root is where it
// belongs.
#[test]
fn check_a_registry_declaring_extern_crate_alloc_reports_nothing() {
    // Arrange & Act
    let offences = check("src/lib.rs", "extern crate alloc;\n\npub mod alpha;\n");

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_a_registry_holding_a_function_reports_it() {
    // Arrange & Act
    let offences = check("src/lib.rs", "pub mod alpha;\n\npub fn helper() {}\n");

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].rule, RULE);
    assert!(
        offences[0].description.contains("the function `helper`"),
        "got {}",
        offences[0].description
    );
}

// A private mod hides part of the crate's shape from the file whose job is to
// state it. Under tests/ the same spelling is fine, because there being
// compiled is the only concern.
#[test]
fn check_a_registry_holding_a_private_mod_reports_it() {
    // Arrange & Act
    let offences = check("src/lib.rs", "pub mod alpha;\nmod hidden;\n");

    // Assert
    assert_eq!(offences.len(), 1);
    assert!(
        offences[0].description.contains("hidden"),
        "got {}",
        offences[0].description
    );
}

// The re-export shim this repository's own standards forbid, caught where it
// most often appears.
#[test]
fn check_a_registry_holding_a_pub_use_reports_it() {
    // Arrange & Act
    let offences = check("src/lib.rs", "pub mod alpha;\n\npub use alpha::Thing;\n");

    // Assert
    assert_eq!(offences.len(), 1);
}

#[test]
fn check_a_registry_holding_an_extern_crate_other_than_alloc_reports_it() {
    // Arrange & Act
    let offences = check("src/lib.rs", "extern crate serde;\n\npub mod alpha;\n");

    // Assert
    assert_eq!(offences.len(), 1);
}

#[test]
fn check_a_registry_holding_an_inline_module_reports_it() {
    // Arrange & Act
    let offences = check("src/lib.rs", "pub mod alpha { }\n");

    // Assert
    assert_eq!(offences.len(), 1);
}

// tests-layout owns the registries under tests/, where a private mod is fine
// and the answers differ. Two rules reporting the same file would say it twice.
#[test]
fn check_a_registry_under_tests_reports_nothing() {
    // Arrange & Act
    let offences = check("tests/rules/mod.rs", "mod alpha_tests;\n");

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// An ordinary source file is where code belongs; this rule has nothing to say
// about it.
#[test]
fn check_an_ordinary_source_file_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/collector.rs",
        "pub struct Collector;\n\nuse alpha::One;\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_reports_every_stray_at_its_own_line() {
    // Arrange & Act
    let offences = check(
        "src/lib.rs",
        "use alpha::One;\n\npub mod alpha;\n\npub fn helper() {}\n",
    );

    // Assert
    assert_eq!(offences.len(), 2);
    assert_eq!(
        offences.iter().map(|o| o.line).collect::<Vec<usize>>(),
        [5, 9]
    );
}

#[test]
fn name_is_the_kebab_case_rule_name_used_in_the_report() {
    // Arrange & Act
    let name = ModuleRegistryRule::new().name();

    // Assert
    assert_eq!(name, RULE);
}
