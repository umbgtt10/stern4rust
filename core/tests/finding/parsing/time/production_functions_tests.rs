// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The functions of a source file that are production code and not a port's
// adapter: free functions, inherent methods and the methods of a foreign
// trait's impl, through inline modules -- and not a test, a `#[cfg(test)]`
// module, an impl of a trait the package declares, or a trait's own default
// bodies, which a test's implementation overrides.

use std::collections::BTreeSet;
use stern4rust::finding::parsing::time::production_functions::ProductionFunctions;
use syn::parse_file;

fn names(source: &str, ports: &[&str]) -> Vec<String> {
    let syntax = parse_file(source).expect("parses");
    let ports: BTreeSet<String> = ports.iter().map(|port| (*port).to_string()).collect();
    ProductionFunctions::of(&syntax.items, &ports)
        .iter()
        .map(|(name, _)| name.to_string())
        .collect()
}

#[test]
fn of_a_free_function_and_an_inherent_method_lists_both() {
    // Arrange & Act
    let found = names(
        "pub fn expired() -> bool { true }\n\npub struct Retry;\n\nimpl Retry {\n    pub fn wait(&self) {}\n}\n",
        &[],
    );

    // Assert
    assert_eq!(found, vec!["expired", "wait"]);
}

#[test]
fn of_a_test_module_and_a_test_function_leaves_them_out() {
    // Arrange & Act
    let found = names(
        "pub fn run() {}\n\n#[test]\nfn runs() {}\n\n#[cfg(test)]\nmod tests {\n    fn helper() {}\n}\n",
        &[],
    );

    // Assert
    assert_eq!(found, vec!["run"]);
}

#[test]
fn of_a_trait_declaration_leaves_its_default_bodies_out() {
    // Arrange & Act
    let found = names(
        "pub trait Clock {\n    fn now(&self) -> u64 {\n        0\n    }\n}\n",
        &["Clock"],
    );

    // Assert
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn of_an_impl_of_a_foreign_trait_lists_its_methods() {
    // Arrange & Act
    let found = names(
        "impl Display for Elapsed {\n    fn fmt(&self, f: &mut Formatter) -> Result {\n        Ok(())\n    }\n}\n",
        &["Clock"],
    );

    // Assert
    assert_eq!(found, vec!["fmt"]);
}

#[test]
fn of_an_impl_of_a_port_leaves_its_methods_out() {
    // Arrange & Act
    let found = names(
        "impl Clock for SystemClock {\n    fn now(&self) -> u64 { 0 }\n}\n\nimpl crate::ports::Clock for Frozen {\n    \
         fn now(&self) -> u64 { 1 }\n}\n",
        &["Clock"],
    );

    // Assert
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn of_an_inline_module_lists_its_functions() {
    // Arrange & Act
    let found = names("mod detail {\n    pub fn deadline() {}\n}\n", &[]);

    // Assert
    assert_eq!(found, vec!["deadline"]);
}
