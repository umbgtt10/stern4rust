// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A record or a data enum lives where its consumers put it. Named in the
// crate's public API, used nowhere, used by two or more files, or by any file
// outside the crate's src/ -- a test, another crate -- it is pub, in a file
// named after it. Used by exactly one file, it is private, inside that file.
// The declaring file's own code counts as a user; the declaration and the
// type's impls do not. Objects are R007's, not this rule's.

use stern4rust::finding::model::name_readers::NameReaders;
use stern4rust::finding::parsing::read_names::ReadNames;
use stern4rust::reporting::offence::Offence;
use stern4rust::rule::Rule;
use stern4rust::rules::types::record_home_rule::RecordHomeRule;
use stern4rust::source_file::SourceFile;

const RULE: &str = "record-home";
const SCHEDULER: &str = "use crate::window::Window;\n\npub struct Scheduler {\n    windows: Vec<Window>,\n}\n\nimpl Scheduler {\n    pub fn run(&self) {}\n}\n";
const WINDOW: &str = "pub struct Window {\n    pub size: u32,\n}\n";

fn check(files: &[(&str, &str)]) -> Vec<Offence> {
    let sources: Vec<SourceFile> = files
        .iter()
        .map(|(path, contents)| SourceFile::new(path, contents))
        .collect();
    RecordHomeRule::new(NameReaders::new(ReadNames::index(&sources))).check_workspace(&sources)
}

#[test]
fn check_of_a_single_file_reports_nothing() {
    // Arrange
    let file = SourceFile::new("src/parser.rs", WINDOW);

    // Act
    let offences = RecordHomeRule::new(NameReaders::default()).check(&file);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_of_a_data_enum_named_in_the_public_api_but_declared_in_another_file_reports_it()
{
    // Arrange & Act
    let offences = check(&[(
        "src/report.rs",
        "pub enum Verdict {\n    Passed,\n    Failed,\n}\n\npub fn verdict() -> Verdict {\n    Verdict::Passed\n}\n",
    )]);

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(
        offences[0].description(),
        "`Verdict` is named in the crate's public API but declared in `src/report.rs`"
    );
}

#[test]
fn check_workspace_of_a_data_enum_used_by_two_files_but_declared_in_a_third_reports_it() {
    // Arrange & Act
    let offences = check(&[
        (
            "src/parser.rs",
            "pub enum Mode {\n    Fast,\n    Slow,\n}\n",
        ),
        ("src/lexer.rs", "use crate::parser::Mode;\n"),
        ("src/reader.rs", "use crate::parser::Mode;\n"),
    ]);

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(
        offences[0].description(),
        "`Mode` is used by 2 files but declared in `src/parser.rs`"
    );
}

#[test]
fn check_workspace_of_a_data_enum_used_nowhere_but_declared_in_another_file_reports_it() {
    // Arrange & Act
    let offences = check(&[("src/parser.rs", "pub enum Mode {\n    Fast,\n}\n")]);

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(
        offences[0].description(),
        "`Mode` is used nowhere but declared in `src/parser.rs`"
    );
}

#[test]
fn check_workspace_of_a_data_enum_used_only_by_one_file_but_in_a_file_of_its_own_reports_it() {
    // Arrange & Act
    let offences = check(&[
        ("src/mode.rs", "pub enum Mode {\n    Fast,\n}\n"),
        ("src/scheduler.rs", "use crate::mode::Mode;\n"),
    ]);

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(
        offences[0].description(),
        "`Mode` is used only by `src/scheduler.rs` but has a file of its own"
    );
}

#[test]
fn check_workspace_of_a_data_enum_with_a_value_trait_living_private_in_its_only_consumer_reports_nothing()
 {
    // Arrange & Act
    let offences = check(&[(
        "src/scheduler.rs",
        "enum Mode {\n    Fast,\n}\n\nimpl Default for Mode {\n    fn default() -> Self {\n        Self::Fast\n    }\n}\n\npub struct Scheduler {\n    mode: Mode,\n}\n\nimpl Scheduler {\n    pub fn run(&self) {}\n}\n",
    )]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_of_a_file_that_does_not_parse_reports_nothing() {
    // Arrange & Act
    let offences = check(&[(
        "src/parser.rs",
        "pub struct LogLine {\n    pub text: String,\n",
    )]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_of_a_record_a_test_also_names_keeps_it_pub_in_its_file() {
    // Arrange & Act
    let offences = check(&[
        ("src/window.rs", WINDOW),
        ("src/scheduler.rs", SCHEDULER),
        ("tests/window_tests.rs", "use node::window::Window;\n"),
    ]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_of_a_record_another_crate_also_uses_keeps_it_pub_in_its_file() {
    // Arrange
    let workspace = [
        SourceFile::new("node/src/window.rs", WINDOW),
        SourceFile::new("node/src/scheduler.rs", SCHEDULER),
        SourceFile::new("app/src/main.rs", "use node::window::Window;\n"),
    ];
    let package = [
        SourceFile::new("src/window.rs", WINDOW),
        SourceFile::new("src/scheduler.rs", SCHEDULER),
    ];
    let rule =
        RecordHomeRule::new(NameReaders::new(ReadNames::index(&workspace)).in_package("node"));

    // Act
    let offences = rule.check_workspace(&package);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_of_a_record_in_a_binary_under_src_bin_leaves_other_crates_out() {
    // Arrange
    let tool = "struct Options {\n    pub fast: bool,\n}\n\nfn main() {\n    let _ = Options { fast: true };\n}\n";
    let workspace = [
        SourceFile::new("node/src/bin/tool.rs", tool),
        SourceFile::new("app/src/lib.rs", "pub fn configure(options: Options) {}\n"),
    ];
    let rule =
        RecordHomeRule::new(NameReaders::new(ReadNames::index(&workspace)).in_package("node"));

    // Act
    let offences = rule.check_workspace(&[SourceFile::new("src/bin/tool.rs", tool)]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_of_a_record_in_a_binarys_main_leaves_other_crates_same_named_types_out() {
    // Arrange
    let main = "struct Cli {\n    pub verbose: bool,\n}\n\nfn main() {\n    let _ = Cli { verbose: true };\n}\n";
    let workspace = [
        SourceFile::new("node/src/main.rs", main),
        SourceFile::new("app/src/main.rs", main),
    ];
    let rule =
        RecordHomeRule::new(NameReaders::new(ReadNames::index(&workspace)).in_package("node"));

    // Act
    let offences = rule.check_workspace(&[SourceFile::new("src/main.rs", main)]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_of_a_record_in_a_cfg_test_item_reports_nothing() {
    // Arrange & Act
    let offences = check(&[(
        "src/parser.rs",
        "#[cfg(test)]\npub struct Fake {\n    pub size: u32,\n}\n",
    )]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_of_a_record_living_private_in_its_only_consumer_reports_nothing() {
    // Arrange & Act
    let offences = check(&[(
        "src/scheduler.rs",
        "struct Window {\n    pub size: u32,\n}\n\npub struct Scheduler {\n    windows: Vec<Window>,\n}\n\nimpl Scheduler {\n    pub fn run(&self) {}\n}\n",
    )]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_of_a_record_living_pub_in_its_only_consumer_says_to_drop_its_pub() {
    // Arrange & Act
    let offences = check(&[(
        "src/scheduler.rs",
        "pub struct Window {\n    pub size: u32,\n}\n\npub struct Scheduler {\n    windows: Vec<Window>,\n}\n\nimpl Scheduler {\n    pub fn run(&self) {}\n}\n",
    )]);

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].rule(), RULE);
    assert_eq!(offences[0].line(), 1);
    assert_eq!(offences[0].subject(), Some("Window"));
    assert_eq!(
        offences[0].description(),
        "`Window` lives in its only consumer, `src/scheduler.rs`, but is `pub`"
    );
    assert_eq!(
        offences[0].correction(),
        "drop its pub: no other file uses it"
    );
}

#[test]
fn check_workspace_of_a_record_named_by_a_pub_alias_counts_it_as_api() {
    // Arrange & Act
    let offences = check(&[(
        "src/votes.rs",
        "pub type Votes = Vec<Ballot>;\n\nstruct Ballot {\n    pub term: u64,\n}\n",
    )]);

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].subject(), Some("Ballot"));
}

#[test]
fn check_workspace_of_a_record_named_by_a_pub_enums_variant_counts_it_as_api() {
    // Arrange & Act
    let offences = check(&[(
        "src/event.rs",
        "pub enum Event {\n    Vote(Ballot),\n}\n\nstruct Ballot {\n    pub term: u64,\n}\n",
    )]);

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].subject(), Some("Ballot"));
}

#[test]
fn check_workspace_of_a_record_named_by_a_pub_field_counts_it_as_api() {
    // Arrange & Act
    let offences = check(&[(
        "src/outer.rs",
        "pub struct Outer {\n    pub inner: Inner,\n}\n\nstruct Inner {\n    pub size: u32,\n}\n",
    )]);

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].subject(), Some("Inner"));
    assert_eq!(offences[0].line(), 5);
}

#[test]
fn check_workspace_of_a_record_named_by_a_pub_fn_of_a_binary_is_not_api() {
    // Arrange & Act
    let offences = check(&[(
        "src/main.rs",
        "pub fn run(options: Options) {}\n\nstruct Options {\n    pub fast: bool,\n}\n\nfn main() {}\n",
    )]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_of_a_record_named_by_a_trait_method_counts_it_as_api() {
    // Arrange & Act
    let offences = check(&[(
        "src/port.rs",
        "pub trait Port {\n    fn send(&self, frame: Frame);\n}\n\nstruct Frame {\n    pub bytes: u32,\n}\n",
    )]);

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].subject(), Some("Frame"));
}

#[test]
fn check_workspace_of_a_record_named_in_the_public_api_but_declared_in_another_file_says_where_it_goes()
 {
    // Arrange & Act
    let offences = check(&[(
        "src/reporting/report.rs",
        "pub struct Verdict {\n    pub passed: bool,\n}\n\npub fn verdict() -> Verdict {\n    Verdict { passed: true }\n}\n",
    )]);

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].file(), "src/reporting/report.rs");
    assert_eq!(
        offences[0].description(),
        "`Verdict` is named in the crate's public API but declared in `src/reporting/report.rs`"
    );
    assert_eq!(
        offences[0].correction(),
        "move Verdict into src/reporting/verdict.rs and make it pub"
    );
}

#[test]
fn check_workspace_of_a_record_named_in_the_public_api_in_a_file_of_its_own_reports_nothing() {
    // Arrange & Act
    let offences = check(&[
        (
            "src/verdict.rs",
            "pub struct Verdict {\n    pub passed: bool,\n}\n",
        ),
        (
            "src/report.rs",
            "use crate::verdict::Verdict;\n\npub fn verdict() -> Verdict {\n    Verdict { passed: true }\n}\n",
        ),
    ]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_of_a_record_used_by_its_declaring_file_and_one_other_counts_both() {
    // Arrange & Act
    let offences = check(&[
        (
            "src/parser.rs",
            "pub struct LogLine {\n    pub text: String,\n}\n\nfn first(lines: &[LogLine]) -> usize {\n    lines.len()\n}\n",
        ),
        ("src/lexer.rs", "use crate::parser::LogLine;\n"),
    ]);

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(
        offences[0].description(),
        "`LogLine` is used by 2 files but declared in `src/parser.rs`"
    );
}

#[test]
fn check_workspace_of_a_record_used_by_two_files_but_declared_in_a_third_says_where_it_goes() {
    // Arrange & Act
    let offences = check(&[
        (
            "src/parser.rs",
            "pub struct LogLine {\n    pub text: String,\n}\n",
        ),
        ("src/lexer.rs", "use crate::parser::LogLine;\n"),
        ("src/reader.rs", "use crate::parser::LogLine;\n"),
    ]);

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(
        offences[0].description(),
        "`LogLine` is used by 2 files but declared in `src/parser.rs`"
    );
    assert_eq!(
        offences[0].correction(),
        "move LogLine into src/log_line.rs and make it pub"
    );
}

#[test]
fn check_workspace_of_a_record_used_by_two_files_in_a_file_of_its_own_but_not_pub_says_to_make_it_pub()
 {
    // Arrange & Act
    let offences = check(&[
        (
            "src/log_line.rs",
            "pub(crate) struct LogLine {\n    pub text: String,\n}\n",
        ),
        ("src/lexer.rs", "use crate::log_line::LogLine;\n"),
        ("src/reader.rs", "use crate::log_line::LogLine;\n"),
    ]);

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(
        offences[0].description(),
        "`LogLine` is used by 2 files but is not `pub`"
    );
    assert_eq!(offences[0].correction(), "make it pub");
}

#[test]
fn check_workspace_of_a_record_used_by_two_files_in_a_file_of_its_own_reports_nothing() {
    // Arrange & Act
    let offences = check(&[
        (
            "src/log_line.rs",
            "pub struct LogLine {\n    pub text: String,\n}\n",
        ),
        ("src/lexer.rs", "use crate::log_line::LogLine;\n"),
        ("src/reader.rs", "use crate::log_line::LogLine;\n"),
    ]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_of_a_record_used_nowhere_in_a_file_of_its_own_but_not_pub_reports_it() {
    // Arrange & Act
    let offences = check(&[(
        "src/log_line.rs",
        "struct LogLine {\n    pub text: String,\n}\n",
    )]);

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(
        offences[0].description(),
        "`LogLine` is used nowhere but is not `pub`"
    );
}

#[test]
fn check_workspace_of_a_record_used_nowhere_in_a_file_of_its_own_reports_nothing() {
    // Arrange & Act
    let offences = check(&[(
        "src/log_line.rs",
        "pub struct LogLine {\n    pub text: String,\n}\n",
    )]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_of_a_record_used_only_by_one_file_but_declared_in_another_says_where_it_goes() {
    // Arrange & Act
    let offences = check(&[("src/shapes.rs", WINDOW), ("src/scheduler.rs", SCHEDULER)]);

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(
        offences[0].description(),
        "`Window` is used only by `src/scheduler.rs` but declared in `src/shapes.rs`"
    );
    assert_eq!(
        offences[0].correction(),
        "move Window into src/scheduler.rs and drop its pub"
    );
}

#[test]
fn check_workspace_of_a_record_used_only_by_one_file_but_in_a_file_of_its_own_reports_it() {
    // Arrange & Act
    let offences = check(&[("src/window.rs", WINDOW), ("src/scheduler.rs", SCHEDULER)]);

    // Assert
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].file(), "src/window.rs");
    assert_eq!(
        offences[0].description(),
        "`Window` is used only by `src/scheduler.rs` but has a file of its own"
    );
    assert_eq!(
        offences[0].correction(),
        "move Window into src/scheduler.rs and drop its pub"
    );
}

#[test]
fn check_workspace_of_a_struct_under_tests_reports_nothing() {
    // Arrange & Act
    let offences = check(&[(
        "tests/support.rs",
        "pub struct Fake {\n    pub size: u32,\n}\n",
    )]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_of_an_enum_with_behaviour_reports_nothing() {
    // Arrange & Act
    let offences = check(&[(
        "src/parser.rs",
        "pub enum Mode {\n    Fast,\n}\n\nimpl Mode {\n    pub fn speed(&self) -> u32 {\n        1\n    }\n}\n",
    )]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_workspace_of_an_object_reports_nothing() {
    // Arrange & Act
    let offences = check(&[(
        "src/parser.rs",
        "pub struct Lexer {\n    position: usize,\n}\n\nimpl Lexer {\n    pub fn advance(&mut self) {\n        self.position += 1;\n    }\n}\n",
    )]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn explanation_shows_a_record_with_one_consumer_moved_into_it() {
    // Arrange & Act
    let explanation = RecordHomeRule::new(NameReaders::default()).explanation();

    // Assert
    assert_eq!(explanation.name, RULE);
    assert!(explanation.breaks.contains("pub struct Window"));
    assert!(explanation.instead.contains("struct Window"));
    assert!(!explanation.instead.contains("pub struct Window"));
}

#[test]
fn is_configured_for_a_rule_that_needs_nothing_returns_true() {
    // Arrange & Act
    let configured = RecordHomeRule::new(NameReaders::default()).is_configured();

    // Assert
    assert!(configured);
}

#[test]
fn name_is_the_kebab_case_rule_name_used_in_the_report() {
    // Arrange & Act
    let name = RecordHomeRule::new(NameReaders::default()).name();

    // Assert
    assert_eq!(name, RULE);
}

#[test]
fn requirement_of_a_rule_that_needs_nothing_is_none() {
    // Arrange & Act
    let requirement = RecordHomeRule::new(NameReaders::default()).requirement();

    // Assert
    assert!(requirement.is_none());
}
