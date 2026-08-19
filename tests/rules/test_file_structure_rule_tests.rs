// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A test file reads top to bottom in one order: header, imports, constants,
// helpers, tests. Each group is alphabetical, and the spacing between entries is
// part of the shape rather than a matter of taste -- imports run together, and
// everything else is separated by exactly one blank line.
//
// The order is what makes a test file skimmable without reading it. Once a
// constant appears below a helper, or a test lands between two others out of
// order, the file stops having a shape and every later addition is placed
// wherever the last one happened to end.
//
// Helpers are defined by exclusion: whatever is neither header, nor `use`, nor
// constant, nor test. A `struct` with an `impl` block -- a recording double, a
// fake -- is a helper and sorts among the functions.

use stern4rust::rule::Rule;
use stern4rust::rules::test_file_structure_rule::TestFileStructureRule;
use stern4rust::source_file::SourceFile;

const HEADER: &str = "// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>\n\
                      // Licensed under the MIT License\n\
                      // SPDX-License-Identifier: MIT\n";

const RULE: &str = "test-file-structure";

fn check(contents: &str) -> Vec<stern4rust::offence::Offence> {
    TestFileStructureRule::new().check(&test_file(contents))
}

fn descriptions(contents: &str) -> Vec<String> {
    check(contents)
        .into_iter()
        .map(|offence| offence.description)
        .collect()
}

fn registry_file(path: &str) -> SourceFile {
    SourceFile::new(
        path,
        &format!("{HEADER}\npub mod alpha_tests;\npub mod beta_tests;\n"),
    )
}

fn source_file(contents: &str) -> SourceFile {
    SourceFile::new("src/subject.rs", &format!("{HEADER}\n{contents}"))
}

fn test_file(contents: &str) -> SourceFile {
    SourceFile::new("tests/subject_tests.rs", &format!("{HEADER}\n{contents}"))
}

// Imports run together. A blank line inside them is how an import block turns
// into two blocks that each sort independently.
#[test]
fn check_a_blank_line_between_imports_reports_it() {
    // Arrange
    let contents = "use alpha::One;\n\
                    \n\
                    use beta::Two;\n";

    // Act
    let offences = check(contents);

    // Assert
    assert_eq!(offences.len(), 1);
    assert!(offences[0].description.contains("blank"));
}

// A comment introducing a test belongs to that test, so the blank line before
// the comment is the separator and the comment itself is not a gap.
#[test]
fn check_a_comment_introducing_a_test_is_part_of_that_test() {
    // Arrange
    let contents = "#[test]\n\
                    fn alpha_does_something() {}\n\
                    \n\
                    // Why this one matters.\n\
                    #[test]\n\
                    fn beta_does_something() {}\n";

    // Act
    let offences = check(contents);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// The sections have one order. A constant below a helper is the first step to a
// file with no shape at all.
#[test]
fn check_a_constant_after_a_helper_reports_the_section_order() {
    // Arrange
    let contents = "fn helper() {}\n\
                    \n\
                    const LATE: usize = 1;\n";

    // Act
    let offences = check(contents);

    // Assert
    assert_eq!(offences.len(), 1);
    assert!(offences[0].description.contains("constant"));
}

#[test]
fn check_a_file_holding_only_a_header_reports_nothing() {
    // Arrange & Act
    let offences = check("");

    // Assert
    assert!(offences.is_empty());
}

#[test]
fn check_a_file_in_the_expected_shape_reports_nothing() {
    // Arrange
    let contents = "use alpha::One;\n\
                    use beta::Two;\n\
                    \n\
                    const FIRST: usize = 1;\n\
                    \n\
                    const SECOND: usize = 2;\n\
                    \n\
                    fn helper_a() {}\n\
                    \n\
                    fn helper_b() {}\n\
                    \n\
                    #[test]\n\
                    fn alpha_does_something() {}\n\
                    \n\
                    #[test]\n\
                    fn beta_does_something() {}\n";

    // Act
    let offences = check(contents);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// Every section is optional but the order of those present still holds.
#[test]
fn check_a_file_of_tests_alone_reports_nothing() {
    // Arrange
    let contents = "#[test]\n\
                    fn alpha_does_something() {}\n\
                    \n\
                    #[test]\n\
                    fn beta_does_something() {}\n";

    // Act
    let offences = check(contents);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// The rule is about test files. A source file has a different shape and is not
// this rule's business.
#[test]
fn check_a_file_outside_the_tests_tree_reports_nothing() {
    // Arrange
    let contents = "use beta::Two;\n\
                    use alpha::One;\n";

    // Act
    let offences = TestFileStructureRule::new().check(&source_file(contents));

    // Assert
    assert!(offences.is_empty());
}

// A file that does not parse is not this rule's problem to report -- rustc will
// say so far more clearly, and guessing at a shape from broken source would
// produce noise on top of a compile error.
#[test]
fn check_a_file_that_does_not_parse_reports_nothing() {
    // Arrange
    let contents = "fn broken( {\n";

    // Act
    let offences = check(contents);

    // Assert
    assert!(offences.is_empty());
}

#[test]
fn check_a_helper_after_a_test_reports_the_section_order() {
    // Arrange
    let contents = "#[test]\n\
                    fn alpha_does_something() {}\n\
                    \n\
                    fn helper() {}\n";

    // Act
    let offences = check(contents);

    // Assert
    assert_eq!(offences.len(), 1);
    assert!(offences[0].description.contains("helper"));
}

#[test]
fn check_a_nested_registry_file_reports_nothing() {
    // Arrange
    let registry = registry_file("tests/rules/mod.rs");

    // Act
    let offences = TestFileStructureRule::new().check(&registry);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// A registry holds nothing but `pub mod` lines and reads as a list. Demanding a
// blank line between each entry would make the one file whose whole job is to be
// scannable the hardest one to scan.
#[test]
fn check_a_registry_file_reports_nothing() {
    // Arrange
    let registry = registry_file("tests/all_tests.rs");

    // Act
    let offences = TestFileStructureRule::new().check(&registry);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_a_section_with_a_single_entry_reports_nothing() {
    // Arrange
    let contents = "use only::One;\n\
                    \n\
                    const ONLY: usize = 1;\n\
                    \n\
                    fn only_helper() {}\n\
                    \n\
                    #[test]\n\
                    fn only_test() {}\n";

    // Act
    let offences = check(contents);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_a_struct_helper_out_of_order_reports_it() {
    // Arrange
    let contents = "struct Recorder;\n\
                    \n\
                    fn build() {}\n";

    // Act
    let offences = check(contents);

    // Assert
    assert_eq!(offences.len(), 1);
    assert!(offences[0].description.contains("build"));
}

// A recording double is a helper like any other, and sorts by its type name
// among the helper functions rather than sitting in a section of its own.
#[test]
fn check_a_struct_helper_sorts_among_the_helper_functions() {
    // Arrange
    let contents = "fn build() {}\n\
                    \n\
                    struct Recorder;\n\
                    \n\
                    fn verify() {}\n";

    // Act
    let offences = check(contents);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// An impl block belongs to the type it implements, so it sorts under that name
// and sits next to the struct rather than drifting to the end of the section.
#[test]
fn check_an_impl_block_sorts_under_the_type_it_implements() {
    // Arrange
    let contents = "fn build() {}\n\
                    \n\
                    struct Recorder;\n\
                    \n\
                    impl Recorder {\n\
                    fn record(&self) {}\n\
                    }\n";

    // Act
    let offences = check(contents);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_an_import_after_a_test_reports_the_section_order() {
    // Arrange
    let contents = "#[test]\n\
                    fn alpha_does_something() {}\n\
                    \n\
                    use late::Import;\n";

    // Act
    let offences = check(contents);

    // Assert
    assert_eq!(offences.len(), 1);
    assert!(offences[0].description.contains("import"));
}

#[test]
fn check_constants_out_of_alphabetic_order_reports_the_later_one() {
    // Arrange
    let contents = "const SECOND: usize = 2;\n\
                    \n\
                    const FIRST: usize = 1;\n";

    // Act
    let offences = check(contents);

    // Assert
    assert_eq!(offences.len(), 1);
    assert!(offences[0].description.contains("FIRST"));
}

#[test]
fn check_constants_run_together_without_a_blank_line_reports_it() {
    // Arrange
    let contents = "const FIRST: usize = 1;\n\
                    const SECOND: usize = 2;\n";

    // Act
    let offences = check(contents);

    // Assert
    assert_eq!(offences.len(), 1);
    assert!(offences[0].description.contains("blank"));
}

#[test]
fn check_helpers_out_of_alphabetic_order_reports_the_later_one() {
    // Arrange
    let contents = "fn zulu() {}\n\
                    \n\
                    fn alpha() {}\n";

    // Act
    let offences = check(contents);

    // Assert
    assert_eq!(offences.len(), 1);
    assert!(offences[0].description.contains("alpha"));
}

// rustfmt sorts crate/self/super ahead of every other path, so demanding the
// alphabet here would make the file unsatisfiable: cargo fmt writes one order,
// this rule demands the other, and stage 1 runs the formatter first. The rule
// stands down on that pair rather than start a fight it cannot win.
//
// The shape this exists for is a shared helper inside the tests tree, reached
// from a sibling as `use crate::support::...`.
#[test]
fn check_imports_out_of_alphabetic_order_around_a_crate_path_reports_nothing() {
    // Arrange
    let contents = "use crate::support::builders;\n\
                    use anyhow::Result;\n";

    // Act
    let offences = check(contents);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// Standing down on the keyword pair does not mean standing down on the file.
// Among ordinary paths rustfmt's comparator and the alphabet agree, so those
// are still ordered.
#[test]
fn check_imports_out_of_alphabetic_order_beside_a_crate_path_still_reports_them() {
    // Arrange
    let contents = "use crate::support::builders;\n\
                    use zebra::Z;\n\
                    use anyhow::Result;\n";

    // Act
    let offences = check(contents);

    // Assert
    assert_eq!(offences.len(), 1);
    assert!(offences[0].description.contains("anyhow::Result"));
}

#[test]
fn check_imports_out_of_alphabetic_order_reports_the_later_one() {
    // Arrange
    let contents = "use beta::Two;\n\
                    use alpha::One;\n";

    // Act
    let offences = check(contents);

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].rule, RULE);
    assert!(offences[0].description.contains("alpha::One"));
}

#[test]
fn check_names_the_file_it_judged() {
    // Arrange
    let contents = "use beta::Two;\n\
                    use alpha::One;\n";

    // Act
    let files = descriptions(contents);

    // Assert
    assert_eq!(files.len(), 1);
}

#[test]
fn check_reports_every_offence_in_a_file_rather_than_only_the_first() {
    // Arrange
    let contents = "use beta::Two;\n\
                    use alpha::One;\n\
                    \n\
                    #[test]\n\
                    fn zulu_does_something() {}\n\
                    \n\
                    #[test]\n\
                    fn alpha_does_something() {}\n";

    // Act
    let offences = check(contents);

    // Assert
    assert_eq!(offences.len(), 2);
}

#[test]
fn check_reports_the_line_the_offending_item_starts_on() {
    // Arrange
    let contents = "use beta::Two;\n\
                    use alpha::One;\n";

    // Act
    let offences = check(contents);

    // Assert
    assert_eq!(offences[0].line, 6);
}

#[test]
fn check_tests_out_of_alphabetic_order_reports_the_later_one() {
    // Arrange
    let contents = "#[test]\n\
                    fn zulu_does_something() {}\n\
                    \n\
                    #[test]\n\
                    fn alpha_does_something() {}\n";

    // Act
    let offences = check(contents);

    // Assert
    assert_eq!(offences.len(), 1);
    assert!(offences[0].description.contains("alpha_does_something"));
}

#[test]
fn check_two_blank_lines_between_tests_reports_it() {
    // Arrange
    let contents = "#[test]\n\
                    fn alpha_does_something() {}\n\
                    \n\
                    \n\
                    #[test]\n\
                    fn beta_does_something() {}\n";

    // Act
    let offences = check(contents);

    // Assert
    assert_eq!(offences.len(), 1);
    assert!(offences[0].description.contains("blank"));
}

#[test]
fn name_is_the_kebab_case_rule_name_used_in_the_report() {
    // Arrange & Act
    let name = TestFileStructureRule::new().name();

    // Assert
    assert_eq!(name, RULE);
}
