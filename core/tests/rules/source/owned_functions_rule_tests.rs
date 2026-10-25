// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A file with a subject owns its functions.
//
// A file that declares a type or a trait, or holds an impl block, has a subject,
// and a free function beside it is a second, unnamed one: single-implemented-type
// and named-subject cannot see it. It belongs inside the impl, as an associated
// function, or in a file of its own. A file of functions alone has no subject to
// compete with and is left alone.
//
// The correction says which, from what the file shows: a function the subject's
// impl calls, or whose body names the subject, is offered the impl; one that
// touches neither is offered a file of its own first. A trait cannot take a
// function -- pure-traits forbids the default body it would need -- so beside a
// trait only a file of its own is offered.
//
// Left out: functions something other than the author requires to be free, and
// `main` in an entry file. tests/ is exempt, where helper functions are what
// test-file-structure asks for, and so are lib.rs and mod.rs, whose functions
// module-registry already reports, and build.rs, which is a Cargo script.

use stern4rust::reporting::offence::Offence;
use stern4rust::rule::Rule;
use stern4rust::rules::source::owned_functions_rule::OwnedFunctionsRule;
use stern4rust::source_file::SourceFile;

const HEADER: &str = "// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>\n\
                      // Licensed under the MIT License\n\
                      // SPDX-License-Identifier: MIT\n";
const IMPLEMENTED: &str = "pub struct Widget;\nimpl Widget { fn run(&self) {} }\n";
const RULE: &str = "owned-functions";

fn check(path: &str, body: &str) -> Vec<Offence> {
    OwnedFunctionsRule::new().check(&SourceFile::new(path, &format!("{HEADER}\n{body}")))
}

fn subjects(offences: &[Offence]) -> Vec<String> {
    offences
        .iter()
        .map(|offence| offence.subject().unwrap_or_default().to_string())
        .collect()
}

#[test]
fn check_a_file_of_functions_alone_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/replication.rs",
        "pub fn handle_heartbeat() {\n    helper();\n}\n\nfn helper() {}\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_a_file_that_does_not_parse_reports_nothing() {
    // Arrange & Act
    let offences = check("src/widget.rs", "pub struct Broken {\n");

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// Each is somebody else's: tests/ wants helper functions, lib.rs and mod.rs are
// module-registry's, and build.rs is a Cargo script rather than crate source.
#[test]
fn check_a_file_under_tests_a_registry_or_a_build_script_reports_nothing() {
    // Arrange
    let paths = [
        "tests/support.rs",
        "src/lib.rs",
        "src/transport/mod.rs",
        "build.rs",
    ];

    // Act
    let counts: Vec<usize> = paths
        .iter()
        .map(|path| check(path, &format!("{IMPLEMENTED}\nfn helper() {{}}\n")).len())
        .collect();

    // Assert
    assert_eq!(counts, [0, 0, 0, 0]);
}

// A trait would need a default body to hold the function, and pure-traits
// forbids one.
#[test]
fn check_a_function_beside_a_trait_offers_only_a_file_of_its_own() {
    // Arrange & Act
    let offences = check(
        "src/udp_stack_factory.rs",
        "pub trait UdpStackFactory {\n    fn make(&self);\n}\n\npub fn init_udp_stack_factory() {}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1, "got {offences:?}");
    assert_eq!(
        offences[0].description(),
        "`init_udp_stack_factory` is a free function in a file whose subject is `UdpStackFactory`"
    );
    assert_eq!(
        offences[0].correction(),
        "move `init_udp_stack_factory` into a file of its own"
    );
}

#[test]
fn check_a_function_beside_a_type_it_never_touches_offers_a_file_of_its_own_first() {
    // Arrange & Act
    let offences = check(
        "src/widget.rs",
        &format!(
            "{IMPLEMENTED}\npub fn checksum(bytes: &[u8]) -> u8 {{\n    bytes.len() as u8\n}}\n"
        ),
    );

    // Assert
    assert_eq!(offences.len(), 1, "got {offences:?}");
    assert_eq!(
        offences[0].correction(),
        "move `checksum` into a file of its own, or into `impl Widget` if it belongs there"
    );
}

// The subject is the type the impl is for, wherever it is declared.
#[test]
fn check_a_function_beside_an_impl_for_a_type_declared_elsewhere_reports_it() {
    // Arrange & Act
    let offences = check(
        "src/dispatch.rs",
        "impl IbftProtocol {\n    pub fn dispatch(&self) {\n        route();\n    }\n}\n\nfn route() {}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1, "got {offences:?}");
    assert_eq!(
        offences[0].description(),
        "`route` is a free function in a file whose subject is `IbftProtocol`"
    );
    assert_eq!(
        offences[0].correction(),
        "move `route` into `impl IbftProtocol` as an associated function"
    );
}

// Plain data is a subject too: `analyze` building an `Analysis` is `Analysis`'s.
#[test]
fn check_a_function_beside_plain_data_reports_it_with_the_type_to_move_into() {
    // Arrange & Act
    let offences = check(
        "src/analysis.rs",
        "pub struct Analysis {\n    pub groups: usize,\n}\n\npub fn analyze() -> Analysis {\n    Analysis { groups: 0 }\n}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1, "got {offences:?}");
    assert_eq!(
        offences[0].correction(),
        "move `analyze` into `impl Analysis` as an associated function"
    );
}

#[test]
fn check_a_function_in_an_inline_module_is_reported() {
    // Arrange & Act
    let offences = check(
        "src/widget.rs",
        &format!("{IMPLEMENTED}\nmod detail {{\n    pub fn helper() {{}}\n}}\n"),
    );

    // Assert
    assert_eq!(subjects(&offences), ["helper"]);
}

#[test]
fn check_a_function_naming_the_subject_is_offered_its_impl() {
    // Arrange & Act
    let offences = check(
        "src/widget.rs",
        &format!(
            "{IMPLEMENTED}\npub fn parse(line: &str) -> Option<Widget> {{\n    line.is_empty().then_some(Widget)\n}}\n"
        ),
    );

    // Assert
    assert_eq!(offences.len(), 1, "got {offences:?}");
    assert_eq!(
        offences[0].correction(),
        "move `parse` into `impl Widget` as an associated function"
    );
}

// Nesting a function inside a method keeps it out of sight, not out of the file.
#[test]
fn check_a_function_nested_in_a_method_is_reported() {
    // Arrange & Act
    let offences = check(
        "src/widget.rs",
        "pub struct Widget;\nimpl Widget {\n    pub fn draw(&self) -> u8 {\n        fn inner() -> u8 {\n            1\n        }\n        inner()\n    }\n}\n",
    );

    // Assert
    assert_eq!(subjects(&offences), ["inner"]);
    assert_eq!(
        offences[0].correction(),
        "move `inner` into `impl Widget` as an associated function"
    );
}

// Each of these is free because something other than the author says so.
#[test]
fn check_a_function_something_requires_free_reports_nothing() {
    // Arrange
    let functions = [
        "#[embassy_executor::task]\nasync fn run_task() {}\n",
        "#[panic_handler]\nfn panic(_info: &PanicInfo) -> ! {\n    loop {}\n}\n",
        "#[cortex_m_rt::exception]\nunsafe fn HardFault(_frame: &ExceptionFrame) -> ! {\n    loop {}\n}\n",
        "#[unsafe(no_mangle)]\npub fn exported() {}\n",
        "pub extern \"C\" fn callback() {}\n",
    ];

    // Act
    let counts: Vec<usize> = functions
        .iter()
        .map(|function| check("src/widget.rs", &format!("{IMPLEMENTED}\n{function}")).len())
        .collect();

    // Assert
    assert_eq!(counts, [0, 0, 0, 0, 0]);
}

#[test]
fn check_a_helper_beside_a_type_in_an_entry_file_reports_it() {
    // Arrange & Act
    let offences = check(
        "src/main.rs",
        &format!("{IMPLEMENTED}\nfn main() {{}}\n\nfn helper() {{}}\n"),
    );

    // Assert
    assert_eq!(subjects(&offences), ["helper"]);
}

// The case the rule exists for: a helper the subject's own impl calls.
#[test]
fn check_a_helper_the_impl_calls_reports_it_with_the_impl_to_move_into() {
    // Arrange & Act
    let offences = check(
        "src/widget.rs",
        "pub struct Widget;\nimpl Widget {\n    pub fn draw(&self) -> u8 {\n        border()\n    }\n}\n\nfn border() -> u8 {\n    1\n}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1, "got {offences:?}");
    assert_eq!(offences[0].rule(), RULE);
    assert_eq!(offences[0].file(), "src/widget.rs");
    assert_eq!(offences[0].line(), 12);
    assert_eq!(offences[0].subject(), Some("border"));
    assert_eq!(
        offences[0].description(),
        "`border` is a free function in a file whose subject is `Widget`"
    );
    assert_eq!(
        offences[0].correction(),
        "move `border` into `impl Widget` as an associated function"
    );
}

#[test]
fn check_a_test_only_function_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/widget.rs",
        &format!("{IMPLEMENTED}\n#[cfg(test)]\nfn only_in_tests() {{}}\n"),
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_associated_functions_methods_and_trait_items_report_nothing() {
    // Arrange & Act
    let offences = check(
        "src/widget.rs",
        "pub trait Draw {\n    fn draw(&self);\n}\n\npub struct Widget;\nimpl Widget {\n    pub fn new() -> Self {\n        Self\n    }\n\n    fn helper() {}\n}\n\nimpl Draw for Widget {\n    fn draw(&self) {}\n}\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_every_free_function_is_reported_once() {
    // Arrange & Act
    let offences = check(
        "src/widget.rs",
        &format!("{IMPLEMENTED}\nfn first() {{}}\n\nfn second() {{}}\n"),
    );

    // Assert
    assert_eq!(subjects(&offences), ["first", "second"]);
}

#[test]
fn check_fn_main_beside_a_type_in_an_entry_file_reports_nothing() {
    // Arrange
    let paths = ["src/main.rs", "src/bin/tool.rs"];

    // Act
    let counts: Vec<usize> = paths
        .iter()
        .map(|path| check(path, &format!("{IMPLEMENTED}\nfn main() {{}}\n")).len())
        .collect();

    // Assert
    assert_eq!(counts, [0, 0]);
}

// The functions are each file's own, so the tree as a whole has nothing to add.
#[test]
fn check_workspace_of_a_tree_with_a_free_function_reports_nothing() {
    // Arrange
    let file = SourceFile::new(
        "src/widget.rs",
        &format!("{IMPLEMENTED}\nfn helper() {{}}\n"),
    );

    // Act
    let offences = OwnedFunctionsRule::new().check_workspace(&[file]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn is_configured_for_a_rule_that_needs_nothing_returns_true() {
    // Arrange & Act
    let configured = OwnedFunctionsRule::new().is_configured();

    // Assert
    assert!(configured);
}

#[test]
fn name_is_the_kebab_case_rule_name_used_in_the_report() {
    // Arrange & Act
    let name = OwnedFunctionsRule::new().name();

    // Assert
    assert_eq!(name, RULE);
}

#[test]
fn requirement_of_a_rule_that_needs_nothing_is_none() {
    // Arrange & Act
    let requirement = OwnedFunctionsRule::new().requirement();

    // Assert
    assert!(requirement.is_none());
}
