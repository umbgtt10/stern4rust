// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// A test reads no clock and waits on no timer. A function in tests/ or benches/, or in a `#[cfg(test)]` module or
// carrying a test attribute in src/, that reads the clock, sleeps, races a timer or draws unseeded randomness is
// reported once, at its name, naming its first such call; Tokio's paused clock may use `tokio::time`.

use stern4rust::reporting::offence::Offence;
use stern4rust::rule::Rule;
use stern4rust::rules::testing::timeless_tests_rule::TimelessTestsRule;
use stern4rust::source_file::SourceFile;

const RULE: &str = "timeless-tests";

fn check(path: &str, contents: &str) -> Vec<Offence> {
    TimelessTestsRule::new().check(&SourceFile::new(path, contents))
}

#[test]
fn check_of_a_bench_that_sleeps_reports_it() {
    // Arrange & Act
    let offences = check(
        "benches/pool.rs",
        "fn warm() {\n    std::thread::sleep(PAUSE);\n}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1, "{offences:?}");
}

#[test]
fn check_of_a_cfg_test_module_in_src_reports_its_test() {
    // Arrange & Act
    let offences = check(
        "src/pool.rs",
        "pub fn take() {}\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn waits() {\n        std::thread::sleep(PAUSE);\n    }\n}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1, "{offences:?}");
    assert_eq!(offences[0].line(), 6);
}

#[test]
fn check_of_a_helper_that_names_a_temp_directory_from_the_clock_reports_it() {
    // Arrange & Act
    let offences = check(
        "tests/scan_tests.rs",
        "use std::time::SystemTime;\n\nfn scratch() -> String {\n    format!(\"{:?}\", SystemTime::now())\n}\n",
    );

    // Assert
    assert_eq!(
        offences[0].description(),
        "`scratch` reads the clock with `SystemTime::now`; a test's verdict must not depend on how fast the machine is"
    );
}

#[test]
fn check_of_a_paused_clock_test_lets_tokio_time_through() {
    // Arrange & Act
    let offences = check(
        "tests/timer_tests.rs",
        "use tokio::time::{advance, sleep, timeout};\n\n#[tokio::test(start_paused = true)]\nasync fn fires() {\n    advance(LIMIT).await;\n    sleep(PAUSE).await;\n    let _ = timeout(LIMIT, work()).await;\n}\n",
    );

    // Assert
    assert!(offences.is_empty(), "{offences:?}");
}

#[test]
fn check_of_a_paused_clock_test_still_reports_the_threads_sleep() {
    // Arrange & Act
    let offences = check(
        "tests/timer_tests.rs",
        "#[tokio::test(start_paused = true)]\nasync fn fires() {\n    std::thread::sleep(PAUSE);\n}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1, "{offences:?}");
}

#[test]
fn check_of_a_stopwatch_test_reports_it_at_its_name() {
    // Arrange & Act
    let offences = check(
        "tests/executor_tests.rs",
        "use std::time::Instant;\n\n#[test]\nfn finishes_in_time() {\n    let start = Instant::now();\n    run();\n    assert!(start.elapsed() < LIMIT);\n}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1, "{offences:?}");
    assert_eq!(offences[0].rule(), RULE);
    assert_eq!(offences[0].line(), 4);
    assert_eq!(
        offences[0].description(),
        "`finishes_in_time` reads the clock with `Instant::now`; a test's verdict must not depend on how fast the machine is"
    );
}

#[test]
fn check_of_a_string_spelling_a_clock_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "tests/fixture_tests.rs",
        "#[test]\nfn finds_the_call() {\n    let source = \"let start = Instant::now();\";\n    assert!(source.contains(\"now\"));\n}\n",
    );

    // Assert
    assert!(offences.is_empty(), "{offences:?}");
}

#[test]
fn check_of_a_test_drawing_unseeded_randomness_names_the_seed() {
    // Arrange & Act
    let offences = check(
        "tests/shuffle_tests.rs",
        "#[test]\nfn shuffles() {\n    let n: u8 = rand::random();\n}\n",
    );

    // Assert
    assert_eq!(
        offences[0].description(),
        "`shuffles` draws unseeded randomness with `rand::random`; a test's verdict must not depend on a draw it cannot repeat"
    );
    assert_eq!(
        offences[0].correction(),
        "seed the generator, `StdRng::seed_from_u64`, so every run draws the same"
    );
}

#[test]
fn check_of_a_test_that_only_configures_a_duration_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "tests/runner_tests.rs",
        "use std::time::Duration;\n\n#[test]\nfn ends_a_hang() {\n    let runner = Runner::new(Duration::from_secs(120));\n    assert!(runner.run().is_ok());\n}\n",
    );

    // Assert
    assert!(offences.is_empty(), "{offences:?}");
}

#[test]
fn check_of_a_test_that_waits_through_an_import_names_the_correction() {
    // Arrange & Act
    let offences = check(
        "tests/pool_tests.rs",
        "use std::thread::sleep;\n\n#[test]\nfn acquire_blocks() {\n    sleep(PAUSE);\n}\n",
    );

    // Assert
    assert_eq!(
        offences[0].description(),
        "`acquire_blocks` waits on a timer with `sleep`; a test's verdict must not depend on how fast the machine is"
    );
    assert_eq!(
        offences[0].correction(),
        "wait for the event itself -- a channel, a barrier, a rendezvous of the parties --, run on Tokio's paused \
         clock, or inject a clock into the code under test and drive it from the test; name a temp directory after \
         the test, or let a temp-directory crate choose"
    );
}

#[test]
fn check_of_a_test_with_two_calls_reports_it_once_at_the_first() {
    // Arrange & Act
    let offences = check(
        "tests/pool_tests.rs",
        "#[test]\nfn acquire_blocks() {\n    std::thread::sleep(PAUSE);\n    let start = std::time::Instant::now();\n}\n",
    );

    // Assert
    assert_eq!(offences.len(), 1, "{offences:?}");
    assert!(
        offences[0].description().contains("`std::thread::sleep`"),
        "{}",
        offences[0].description()
    );
}

// readable-source reports a file that does not parse; this rule has nothing to read in it.
#[test]
fn check_of_an_unparseable_file_reports_nothing() {
    // Arrange & Act
    let offences = check("tests/broken_tests.rs", "fn broken( {\n");

    // Assert
    assert!(offences.is_empty(), "{offences:?}");
}

#[test]
fn check_of_production_code_in_src_reports_nothing() {
    // Arrange & Act
    let offences = check(
        "src/runner.rs",
        "pub fn run() {\n    let start = std::time::Instant::now();\n    std::thread::sleep(PAUSE);\n}\n",
    );

    // Assert
    assert!(offences.is_empty(), "{offences:?}");
}

// This rule judges one file at a time, and says so rather than inheriting a silence nobody chose.
#[test]
fn check_workspace_reports_nothing() {
    // Arrange
    let files = [SourceFile::new(
        "tests/pool_tests.rs",
        "#[test]\nfn waits() {\n    std::thread::sleep(PAUSE);\n}\n",
    )];

    // Act
    let offences = TimelessTestsRule::new().check_workspace(&files);

    // Assert
    assert!(offences.is_empty(), "{offences:?}");
}

// The example it teaches by must be one it reports.
#[test]
fn explanation_breaks_the_rule_it_explains() {
    // Arrange
    let explanation = TimelessTestsRule::new().explanation();

    // Act
    let broken = check("tests/example_tests.rs", explanation.breaks);
    let mended = check("tests/example_tests.rs", explanation.instead);

    // Assert
    assert_eq!(broken.len(), 1, "{broken:?}");
    assert!(mended.is_empty(), "{mended:?}");
}

#[test]
fn explanation_shows_the_stopwatch_and_the_channel() {
    // Arrange & Act
    let explanation = TimelessTestsRule::new().explanation();

    // Assert
    assert_eq!(
        explanation.breaks,
        "#[test]\nfn finishes_in_time() {\n    let start = std::time::Instant::now();\n    run();\n    \
         assert!(start.elapsed() < LIMIT);\n}\n"
    );
    assert_eq!(
        explanation.instead,
        "#[test]\nfn finishes() {\n    let (sender, receiver) = std::sync::mpsc::channel();\n    run(sender);\n    \
         assert!(receiver.recv().is_ok());\n}\n"
    );
}

#[test]
fn is_configured_needs_nothing() {
    // Arrange & Act & Assert
    assert!(TimelessTestsRule::new().is_configured());
    assert_eq!(TimelessTestsRule::new().requirement(), None);
}

#[test]
fn name_is_timeless_tests() {
    // Arrange & Act & Assert
    assert_eq!(TimelessTestsRule::new().name(), RULE);
}
