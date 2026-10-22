// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The first call in a function's body through which it depends on time: a clock read, a wait, a race or an
// unseeded draw, resolved through the file's imports. On Tokio's paused clock only `tokio::time` is let through.

use stern4rust::finding::model::time_call_kind::TimeCallKind;
use stern4rust::finding::parsing::time::imported_names::ImportedNames;
use stern4rust::finding::parsing::time::time_call_finder::TimeCallFinder;
use syn::Item;
use syn::parse_file;

fn first(source: &str, paused: bool) -> Option<(TimeCallKind, String, usize)> {
    let syntax = parse_file(source).expect("parses");
    let names = ImportedNames::of(&syntax.items);
    let body = syntax
        .items
        .iter()
        .find_map(|item| match item {
            Item::Fn(function) => Some(function.block.as_ref().clone()),
            _ => None,
        })
        .expect("a function");
    TimeCallFinder::new(&names, paused)
        .first_in(&body)
        .map(|call| (call.kind, call.written, call.line))
}

#[test]
fn calls_in_lists_every_call_in_the_order_written() {
    // Arrange
    let syntax = parse_file(
        "use std::time::Instant;\nfn t() {\n    let seed = rand::random::<u8>();\n    let start = Instant::now();\n}\n",
    )
    .expect("parses");
    let names = ImportedNames::of(&syntax.items);
    let Item::Fn(function) = &syntax.items[1] else {
        panic!("a function");
    };

    // Act
    let calls = TimeCallFinder::new(&names, false).calls_in(&function.block);

    // Assert
    let found: Vec<(TimeCallKind, &str)> = calls
        .iter()
        .map(|call| (call.kind, call.written.as_str()))
        .collect();
    assert_eq!(
        found,
        vec![
            (TimeCallKind::Random, "rand::random"),
            (TimeCallKind::Clock, "Instant::now"),
        ]
    );
}

#[test]
fn first_in_a_body_without_time_is_none() {
    // Arrange & Act
    let found = first(
        "use std::time::Duration;\nfn t() {\n    let limit = Duration::from_secs(5);\n    let _ = \"Instant::now()\";\n}\n",
        false,
    );

    // Assert
    assert_eq!(found, None);
}

#[test]
fn first_in_finds_a_clock_handed_over_as_a_function() {
    // Arrange & Act
    let found = first(
        "use std::time::Instant;\nfn t(cached: Option<Instant>) -> Instant {\n    cached.unwrap_or_else(Instant::now)\n}\n",
        false,
    );

    // Assert
    assert_eq!(
        found,
        Some((TimeCallKind::Clock, String::from("Instant::now"), 3))
    );
}

#[test]
fn first_in_finds_a_clock_read_through_an_import() {
    // Arrange & Act
    let found = first(
        "use std::time::Instant;\nfn t() {\n    let start = Instant::now();\n}\n",
        false,
    );

    // Assert
    assert_eq!(
        found,
        Some((TimeCallKind::Clock, String::from("Instant::now"), 3))
    );
}

#[test]
fn first_in_finds_a_wait_through_a_full_path() {
    // Arrange & Act
    let found = first(
        "fn t() {\n    std::thread::sleep(std::time::Duration::from_millis(5));\n}\n",
        false,
    );

    // Assert
    assert_eq!(
        found,
        Some((TimeCallKind::Wait, String::from("std::thread::sleep"), 2))
    );
}

#[test]
fn first_in_finds_an_elapsed_reading() {
    // Arrange & Act
    let found = first(
        "fn t(start: u8) {\n    let taken = start.elapsed();\n}\n",
        false,
    );

    // Assert
    assert_eq!(
        found,
        Some((TimeCallKind::Clock, String::from("elapsed"), 2))
    );
}

#[test]
fn first_in_finds_an_embassy_clock_read() {
    // Arrange & Act
    let found = first(
        "use embassy_time::Instant;\nfn t() {\n    let deadline = Instant::now();\n}\n",
        false,
    );

    // Assert
    assert_eq!(
        found,
        Some((TimeCallKind::Clock, String::from("Instant::now"), 3))
    );
}

#[test]
fn first_in_finds_an_embassy_timeout_as_a_race() {
    // Arrange & Act
    let found = first(
        "use embassy_time::with_timeout;\nasync fn t() {\n    let _ = with_timeout(LIMIT, receive()).await;\n}\n",
        false,
    );

    // Assert
    assert_eq!(
        found,
        Some((TimeCallKind::Race, String::from("with_timeout"), 3))
    );
}

#[test]
fn first_in_finds_an_unseeded_generator_as_a_value() {
    // Arrange & Act
    let found = first(
        "use rand::rngs::OsRng;\nfn t() {\n    let n = OsRng.next_u32();\n}\n",
        false,
    );

    // Assert
    assert_eq!(
        found,
        Some((TimeCallKind::Random, String::from("OsRng"), 3))
    );
}

#[test]
fn first_in_leaves_a_binding_named_after_an_imported_timer_alone() {
    // Arrange & Act
    let found = first(
        "use tokio::time::sleep;\nfn t(sleep: Duration) {\n    pause(sleep);\n}\n",
        false,
    );

    // Assert
    assert_eq!(found, None);
}

#[test]
fn first_in_leaves_a_duration_between_two_instants_alone() {
    // Arrange & Act
    let found = first(
        "fn t(started: Instant, finished: Instant) {\n    let taken = finished.duration_since(started);\n}\n",
        false,
    );

    // Assert
    assert_eq!(found, None);
}

#[test]
fn first_in_leaves_a_field_named_after_an_imported_timer_alone() {
    // Arrange & Act
    let found = first(
        "use tokio::time::timeout;\nfn t(timeout: Duration) -> Runner {\n    Runner { timeout }\n}\n",
        false,
    );

    // Assert
    assert_eq!(found, None);
}

#[test]
fn first_in_leaves_a_function_of_the_same_name_alone() {
    // Arrange & Act
    let found = first("fn t() {\n    sleep(5);\n}\nfn sleep(_: u8) {}\n", false);

    // Assert
    assert_eq!(found, None);
}

#[test]
fn first_in_leaves_a_seeded_generator_alone() {
    // Arrange & Act
    let found = first(
        "use rand::SeedableRng;\nuse rand::rngs::StdRng;\nfn t() {\n    let rng = StdRng::seed_from_u64(7);\n}\n",
        false,
    );

    // Assert
    assert_eq!(found, None);
}

#[test]
fn first_in_names_the_earliest_call_of_several() {
    // Arrange & Act
    let found = first(
        "use tokio::time::timeout;\nfn t() {\n    let _ = timeout(LIMIT, work());\n    std::thread::sleep(PAUSE);\n}\n",
        false,
    );

    // Assert
    assert_eq!(
        found,
        Some((TimeCallKind::Race, String::from("timeout"), 3))
    );
}

#[test]
fn first_in_on_the_paused_clock_lets_tokio_time_through() {
    // Arrange & Act
    let found = first(
        "use tokio::time::{sleep, Instant};\nfn t() {\n    let start = Instant::now();\n    sleep(PAUSE);\n    let taken = start.elapsed();\n}\n",
        true,
    );

    // Assert
    assert_eq!(found, None);
}

#[test]
fn first_in_on_the_paused_clock_still_finds_the_systems_clock() {
    // Arrange & Act
    let found = first(
        "use std::time::Instant;\nfn t() {\n    let start = Instant::now();\n}\n",
        true,
    );

    // Assert
    assert_eq!(
        found,
        Some((TimeCallKind::Clock, String::from("Instant::now"), 3))
    );
}

#[test]
fn first_in_reads_a_call_inside_a_macros_arguments() {
    // Arrange & Act
    let found = first(
        "fn t(start: u8) {\n    assert!(start.elapsed() < LIMIT, \"too slow\");\n}\n",
        false,
    );

    // Assert
    assert_eq!(
        found,
        Some((TimeCallKind::Clock, String::from("elapsed"), 2))
    );
}
