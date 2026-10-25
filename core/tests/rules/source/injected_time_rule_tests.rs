// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// Time reaches production code through a port. In the library target, a
// function reading a clock, waiting on a timer or racing one is reported once,
// at its name, naming the first call -- unless it implements a trait the
// package declares, the adapter a test replaces. A binary's files, test code,
// randomness and string literals are left alone.

use stern4rust::reporting::offence::Offence;
use stern4rust::rule::Rule;
use stern4rust::rules::source::injected_time_rule::InjectedTimeRule;
use stern4rust::source_file::SourceFile;

const CORRECTION: &str = "declare a trait for the clock or timer it needs, implement it with the real \
                          one in an adapter and inject it -- or, for a timer that only marks a \
                          deadline, take the deadline in as an event the caller feeds";
const LIBRARY: &str = "pub mod retry;\n";
const RULE: &str = "injected-time";

fn check(files: &[(&str, &str)]) -> Vec<Offence> {
    let files: Vec<SourceFile> = files
        .iter()
        .map(|(path, contents)| SourceFile::new(path, contents))
        .collect();
    InjectedTimeRule::new().check_workspace(&files)
}

#[test]
fn check_of_a_file_reports_nothing_on_its_own() {
    // Arrange
    let file = SourceFile::new(
        "src/retry.rs",
        "pub fn expired() -> bool {\n    std::time::Instant::now().elapsed().is_zero()\n}\n",
    );

    // Act
    let offences = InjectedTimeRule::new().check(&file);

    // Assert
    assert!(offences.is_empty(), "{offences:?}");
}

#[test]
fn check_workspace_of_a_binarys_files_reports_nothing() {
    // Arrange & Act
    let offences = check(&[
        ("src/lib.rs", LIBRARY),
        ("src/retry.rs", "pub fn attempts() -> u8 {\n    3\n}\n"),
        (
            "src/main.rs",
            "mod cli;\n\nfn main() {\n    let started = std::time::Instant::now();\n}\n",
        ),
        (
            "src/cli.rs",
            "pub fn pause() {\n    std::thread::sleep(PAUSE);\n}\n",
        ),
        (
            "src/bin/probe.rs",
            "fn main() {\n    std::thread::sleep(PAUSE);\n}\n",
        ),
    ]);

    // Assert
    assert!(offences.is_empty(), "{offences:?}");
}

#[test]
fn check_workspace_of_a_foreign_traits_impl_reports_it() {
    // Arrange & Act
    let offences = check(&[
        ("src/lib.rs", LIBRARY),
        (
            "src/retry.rs",
            "use std::fmt::Display;\n\npub struct Elapsed(std::time::Instant);\n\nimpl Display for Elapsed {\n    \
             fn fmt(&self, f: &mut Formatter) -> Result {\n        write!(f, \"{:?}\", self.0.elapsed())\n    \
             }\n}\n",
        ),
    ]);

    // Assert
    assert_eq!(offences.len(), 1, "{offences:?}");
    assert_eq!(offences[0].subject(), Some("fmt"));
}

#[test]
fn check_workspace_of_a_free_function_reading_the_clock_reports_it_at_its_name() {
    // Arrange & Act
    let offences = check(&[
        ("src/lib.rs", LIBRARY),
        (
            "src/retry.rs",
            "use std::time::Instant;\n\npub fn expired(deadline: Instant) -> bool {\n    Instant::now() > deadline\n}\n",
        ),
    ]);

    // Assert
    assert_eq!(offences.len(), 1, "{offences:?}");
    assert_eq!(offences[0].rule(), RULE);
    assert_eq!(offences[0].file(), "src/retry.rs");
    assert_eq!(offences[0].line(), 3);
    assert_eq!(offences[0].subject(), Some("expired"));
    assert_eq!(
        offences[0].description(),
        "`expired` reads the clock with `Instant::now`; take time through a port the package declares, so a \
         test can drive it"
    );
    assert_eq!(offences[0].correction(), CORRECTION);
}

#[test]
fn check_workspace_of_a_function_with_randomness_then_a_clock_names_the_clock() {
    // Arrange & Act
    let offences = check(&[
        ("src/lib.rs", LIBRARY),
        (
            "src/retry.rs",
            "pub fn jitter() -> u64 {\n    let spread: u64 = rand::random();\n    \
             spread + std::time::Instant::now().elapsed().as_secs()\n}\n",
        ),
    ]);

    // Assert
    assert_eq!(offences.len(), 1, "{offences:?}");
    assert_eq!(
        offences[0].description(),
        "`jitter` reads the clock with `std::time::Instant::now`; take time through a port the package \
         declares, so a test can drive it"
    );
}

#[test]
fn check_workspace_of_a_function_with_two_calls_reports_it_once_at_the_first() {
    // Arrange & Act
    let offences = check(&[
        ("src/lib.rs", LIBRARY),
        (
            "src/retry.rs",
            "use std::thread::sleep;\n\npub fn backoff() {\n    sleep(PAUSE);\n    \
             let started = std::time::Instant::now();\n}\n",
        ),
    ]);

    // Assert
    assert_eq!(offences.len(), 1, "{offences:?}");
    assert_eq!(
        offences[0].description(),
        "`backoff` waits on a timer with `sleep`; take time through a port the package declares, so a test \
         can drive it"
    );
}

#[test]
fn check_workspace_of_a_package_without_a_library_reports_nothing() {
    // Arrange & Act
    let offences = check(&[(
        "src/main.rs",
        "fn main() {\n    let started = std::time::Instant::now();\n}\n",
    )]);

    // Assert
    assert!(offences.is_empty(), "{offences:?}");
}

#[test]
fn check_workspace_of_a_port_the_package_declares_reports_nothing_in_its_adapter() {
    // Arrange & Act
    let offences = check(&[
        ("src/lib.rs", "pub mod clock;\npub mod system_clock;\n"),
        (
            "src/clock.rs",
            "use std::time::Instant;\n\npub trait Clock {\n    fn now(&self) -> Instant;\n}\n",
        ),
        (
            "src/system_clock.rs",
            "use crate::clock::Clock;\nuse std::time::Instant;\n\npub struct SystemClock;\n\n\
             impl Clock for SystemClock {\n    fn now(&self) -> Instant {\n        Instant::now()\n    }\n}\n",
        ),
    ]);

    // Assert
    assert!(offences.is_empty(), "{offences:?}");
}

#[test]
fn check_workspace_of_a_race_in_an_inherent_method_reports_it() {
    // Arrange & Act
    let offences = check(&[
        ("src/lib.rs", LIBRARY),
        (
            "src/retry.rs",
            "pub struct Retry;\n\nimpl Retry {\n    pub async fn attempt(&self) {\n        \
             let _ = tokio::time::timeout(LIMIT, send()).await;\n    }\n}\n",
        ),
    ]);

    // Assert
    assert_eq!(offences.len(), 1, "{offences:?}");
    assert_eq!(offences[0].subject(), Some("attempt"));
    assert_eq!(
        offences[0].description(),
        "`attempt` races a timer with `tokio::time::timeout`; take time through a port the package declares, \
         so a test can drive it"
    );
}

#[test]
fn check_workspace_of_a_string_spelling_a_clock_reports_nothing() {
    // Arrange & Act
    let offences = check(&[
        ("src/lib.rs", LIBRARY),
        (
            "src/retry.rs",
            "pub fn pattern() -> &'static str {\n    \"Instant::now()\"\n}\n",
        ),
    ]);

    // Assert
    assert!(offences.is_empty(), "{offences:?}");
}

// A family's core crate declares the ports; a runtime package implements them,
// and its adapter is where the real clock is read.
#[test]
fn check_workspace_of_an_adapter_of_a_dependencys_port_reports_nothing() {
    // Arrange & Act
    let offences = check(&[
        ("src/lib.rs", "pub mod desktop_timer;\n"),
        (
            "src/desktop_timer.rs",
            "use etheram_core::timer_output::TimerOutput;\nuse std::time::Instant;\n\npub struct DesktopTimer;\n\n\
             impl TimerOutput for DesktopTimer {\n    fn schedule(&self) {\n        let deadline = Instant::now();\n    \
             }\n}\n",
        ),
    ]);

    // Assert
    assert!(offences.is_empty(), "{offences:?}");
}

#[test]
fn check_workspace_of_an_unparseable_file_reports_nothing() {
    // Arrange & Act
    let offences = check(&[
        ("src/lib.rs", LIBRARY),
        ("src/retry.rs", "pub fn expired( {\n    Instant::now()\n"),
    ]);

    // Assert
    assert!(offences.is_empty(), "{offences:?}");
}

#[test]
fn check_workspace_of_test_code_reports_nothing() {
    // Arrange & Act
    let offences = check(&[
        ("src/lib.rs", LIBRARY),
        (
            "src/retry.rs",
            "pub fn attempts() -> u8 {\n    3\n}\n\n#[cfg(test)]\nmod tests {\n    fn stamp() {\n        \
             let now = std::time::Instant::now();\n    }\n}\n",
        ),
        (
            "tests/retry_tests.rs",
            "#[test]\nfn attempts_are_three() {\n    std::thread::sleep(PAUSE);\n}\n",
        ),
    ]);

    // Assert
    assert!(offences.is_empty(), "{offences:?}");
}

#[test]
fn check_workspace_of_unseeded_randomness_reports_nothing() {
    // Arrange & Act
    let offences = check(&[
        ("src/lib.rs", LIBRARY),
        (
            "src/retry.rs",
            "pub fn jitter() -> u64 {\n    rand::random()\n}\n",
        ),
    ]);

    // Assert
    assert!(offences.is_empty(), "{offences:?}");
}

#[test]
fn explanation_breaks_the_rule_it_explains() {
    // Arrange
    let explanation = InjectedTimeRule::new().explanation();

    // Act
    let broken = check(&[("src/lib.rs", explanation.breaks)]);
    let mended = check(&[("src/lib.rs", explanation.instead)]);

    // Assert
    assert_eq!(broken.len(), 1, "{broken:?}");
    assert!(mended.is_empty(), "{mended:?}");
}

#[test]
fn is_configured_needs_nothing() {
    // Arrange & Act & Assert
    assert!(InjectedTimeRule::new().is_configured());
    assert_eq!(InjectedTimeRule::new().requirement(), None);
}

#[test]
fn name_is_injected_time() {
    // Arrange & Act & Assert
    assert_eq!(InjectedTimeRule::new().name(), RULE);
}
