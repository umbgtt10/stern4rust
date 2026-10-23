// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A source file is named after its subject.
//
// The subject is a type the file implements -- declared here, with an impl block
// here, as single-implemented-type counts them. A file with none is named after
// one of the traits and types it declares instead. A trait beside an implemented
// type is that type's companion and is not a candidate.
//
// Named after means the file's stem is the type's name, or its last words: a
// type's qualifier is usually supplied by the crate it lives in, and
// `loopback/src/driver.rs` holding `LoopbackDriver` says what it holds.
//
// The offence is one per file, against the first candidate, with the file it
// would be named for. Entry points and module roots are named for their role,
// and tests/, benches/, examples/ and src/bin/ for what they exercise or run.

use stern4rust::reporting::offence::Offence;
use stern4rust::rule::Rule;
use stern4rust::rules::source::named_subject_rule::NamedSubjectRule;
use stern4rust::source_file::SourceFile;

const HEADER: &str = "// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>\n\
                      // Licensed under the MIT License\n\
                      // SPDX-License-Identifier: MIT\n";
const IMPLEMENTED: &str = "pub struct Widget;\nimpl Widget {}\n";
const RULE: &str = "named-subject";

fn check(path: &str, body: &str) -> Vec<Offence> {
    NamedSubjectRule::new().check(&SourceFile::new(path, &format!("{HEADER}\n{body}")))
}

#[test]
fn check_a_file_named_after_its_implemented_type_reports_nothing() {
    // Arrange & Act
    let offences = check("src/widget.rs", IMPLEMENTED);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// single-implemented-type reports the companion; this rule has nothing to add.
#[test]
fn check_a_file_named_after_its_second_implemented_type_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/wire_client_request.rs",
        "pub struct WireTransaction;\nimpl WireTransaction {}\n\npub enum WireClientRequest {\n    One,\n}\nimpl WireClientRequest {}\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn check_a_file_named_after_its_trait_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/partitioner.rs",
        "pub type Partitioned = Vec<u8>;\n\npub trait Partitioner {\n    fn partition(&self) -> Partitioned;\n}\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// The case the rule exists for. The line is the declaration's, as
// single-implemented-type's is: the type is what a reader goes looking for.
#[test]
fn check_a_file_named_after_no_implemented_type_reports_the_first() {
    // Arrange & Act
    let offences = check(
        "src/listener.rs",
        "const BUFFER: usize = 256;\n\npub struct UdpListenerTask;\nimpl UdpListenerTask {}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1, "got {offences:?}");
    assert_eq!(offences[0].rule(), RULE);
    assert_eq!(offences[0].file(), "src/listener.rs");
    assert_eq!(offences[0].line(), 7);
    assert_eq!(offences[0].subject(), Some("UdpListenerTask"));
    assert_eq!(
        offences[0].description(),
        "`listener.rs` is named after none of the types it implements: `UdpListenerTask`"
    );
    assert_eq!(
        offences[0].correction(),
        "rename the file to udp_listener_task.rs, or rename `UdpListenerTask` after the file"
    );
}

#[test]
fn check_a_file_named_after_no_trait_or_type_reports_the_first_trait() {
    // Arrange & Act
    let offences = check(
        "src/inbox.rs",
        "pub struct Envelope;\n\npub trait InboxSend {\n    fn send(&self, envelope: Envelope);\n}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1, "got {offences:?}");
    assert_eq!(offences[0].subject(), Some("InboxSend"));
    assert_eq!(
        offences[0].description(),
        "`inbox.rs` is named after none of the traits and types it declares: `InboxSend`, `Envelope`"
    );
    assert_eq!(
        offences[0].correction(),
        "rename the file to inbox_send.rs, or rename `InboxSend` after the file"
    );
}

#[test]
fn check_a_file_named_after_one_of_its_data_types_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/action.rs",
        "pub enum TimerKey {\n    Round,\n}\n\npub enum Action {\n    Start(TimerKey),\n}\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// The data type the file is named for has no behaviour; the type that has some
// is the subject, and it is somewhere it was not looked for.
#[test]
fn check_a_file_named_after_plain_data_beside_an_implemented_type_reports_the_implemented_type() {
    // Arrange & Act
    let offences = check(
        "src/log_record.rs",
        "pub struct LogRecord {\n    pub severity: LogSeverity,\n}\n\npub enum LogSeverity {\n    Info,\n}\nimpl LogSeverity {}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1, "got {offences:?}");
    assert_eq!(offences[0].subject(), Some("LogSeverity"));
    assert_eq!(
        offences[0].correction(),
        "rename the file to log_severity.rs, or rename `LogSeverity` after the file"
    );
}

#[test]
fn check_a_file_named_after_the_last_words_of_its_implemented_type_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/listener.rs",
        "pub struct UdpEiListener;\nimpl UdpEiListener {}\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// Each is named for its role in the package, not for a type.
#[test]
fn check_a_file_of_a_reserved_name_reports_nothing() {
    // Arrange
    let paths = [
        "src/lib.rs",
        "src/main.rs",
        "src/transport/mod.rs",
        "build.rs",
    ];

    // Act
    let counts: Vec<usize> = paths
        .iter()
        .map(|path| check(path, IMPLEMENTED).len())
        .collect();

    // Assert
    assert_eq!(counts, [0, 0, 0, 0]);
}

#[test]
fn check_a_file_of_data_types_named_after_none_reports_the_first() {
    // Arrange & Act
    let offences = check(
        "src/cluster_query_types.rs",
        "pub struct QueryRequest;\n\npub struct QueryResponse;\n",
    );

    // Assert
    assert_eq!(offences.len(), 1, "got {offences:?}");
    assert_eq!(offences[0].line(), 5);
    assert_eq!(
        offences[0].description(),
        "`cluster_query_types.rs` is named after none of the traits and types it declares: `QueryRequest`, `QueryResponse`"
    );
    assert_eq!(
        offences[0].correction(),
        "rename the file to query_request.rs, or rename `QueryRequest` after the file"
    );
}

#[test]
fn check_a_file_that_does_not_parse_reports_nothing() {
    // Arrange & Act
    let offences = check("src/listener.rs", "pub struct Broken {\n");

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// Named for what they exercise or run, and a test file holds as many fakes as
// it needs.
#[test]
fn check_a_file_under_tests_benches_examples_or_src_bin_reports_nothing() {
    // Arrange
    let paths = [
        "tests/support.rs",
        "benches/throughput.rs",
        "examples/demo.rs",
        "src/bin/tool.rs",
    ];

    // Act
    let counts: Vec<usize> = paths
        .iter()
        .map(|path| check(path, IMPLEMENTED).len())
        .collect();

    // Assert
    assert_eq!(counts, [0, 0, 0, 0]);
}

// Functions and constants have no type to be named after, so there is nothing
// to hold the name to.
#[test]
fn check_a_file_with_neither_types_nor_traits_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/block_on.rs",
        "pub const LIMIT: u32 = 3;\n\npub fn block_on() {}\n",
    );

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

// One file, one wrong name: the second subject is single-implemented-type's.
#[test]
fn check_a_file_with_several_implemented_types_named_after_none_reports_once() {
    // Arrange & Act
    let offences = check(
        "src/pair.rs",
        "pub struct Left;\nimpl Left {}\n\npub struct Right;\nimpl Right {}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1, "got {offences:?}");
    assert_eq!(offences[0].subject(), Some("Left"));
}

// The trait is the implemented type's companion, so the file answers for the
// type alone.
#[test]
fn check_a_trait_beside_an_implemented_type_reports_the_type() {
    // Arrange & Act
    let offences = check(
        "src/backoff_policy.rs",
        "pub trait BackoffPolicy {}\n\npub struct Fixed;\nimpl BackoffPolicy for Fixed {}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1, "got {offences:?}");
    assert_eq!(
        offences[0].description(),
        "`backoff_policy.rs` is named after none of the types it implements: `Fixed`"
    );
}

// The subject is each file's own, so the tree as a whole has nothing to add.
#[test]
fn check_workspace_of_a_tree_with_a_misnamed_file_reports_nothing() {
    // Arrange
    let file = SourceFile::new("src/listener.rs", "pub struct Backoff;\nimpl Backoff {}\n");

    // Act
    let offences = NamedSubjectRule::new().check_workspace(&[file]);

    // Assert
    assert!(offences.is_empty(), "expected none, got {offences:?}");
}

#[test]
fn is_configured_for_a_rule_that_needs_nothing_returns_true() {
    // Arrange & Act
    let configured = NamedSubjectRule::new().is_configured();

    // Assert
    assert!(configured);
}

#[test]
fn name_is_the_kebab_case_rule_name_used_in_the_report() {
    // Arrange & Act
    let name = NamedSubjectRule::new().name();

    // Assert
    assert_eq!(name, RULE);
}

#[test]
fn requirement_of_a_rule_that_needs_nothing_is_none() {
    // Arrange & Act
    let requirement = NamedSubjectRule::new().requirement();

    // Assert
    assert!(requirement.is_none());
}
