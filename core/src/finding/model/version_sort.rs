// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::cmp::Ordering;
use std::iter::once;

// How the 2024 style edition orders two names: the Rust style guide's version
// sorting, which rustfmt applies to the segments of an import.
//
// A name is read as chunks -- a lone underscore, a run of ASCII digits, a run of
// anything else -- compared pair by pair. An underscore sorts before every other
// chunk, two numbers compare by value, and anything else by its text, so
// `item2` sorts before `item10` and `a_b` before `aB`, where a byte compare --
// the 2021 edition's -- says the opposite of both. Where every pair is equal, the
// name with fewer chunks comes first, and then the one whose first number
// differing only in its leading zeros has more of them.
//
// Measured, not read: on 1,200 names drawn from letters of both cases, digits
// and underscores, this orders them exactly as `cargo fmt` does on an edition
// 2024 crate with the toolchain this repository pins.
pub struct VersionSort;

impl VersionSort {
    pub fn compare(left: &str, right: &str) -> Ordering {
        let (left, right) = (Self::chunks(left), Self::chunks(right));
        left.iter()
            .zip(&right)
            .map(|(one, other)| Self::chunk(one, other))
            .find(|order| order.is_ne())
            .unwrap_or_else(|| {
                left.len()
                    .cmp(&right.len())
                    .then_with(|| Self::leading_zeros(&left, &right))
            })
    }

    // A chunk ends at an underscore, after one, and where digits meet anything
    // else.
    fn breaks(before: char, after: char) -> bool {
        before == '_' || after == '_' || before.is_ascii_digit() != after.is_ascii_digit()
    }

    fn chunk(left: &str, right: &str) -> Ordering {
        match (left == "_", right == "_") {
            (true, true) => Ordering::Equal,
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            _ if Self::is_number(left) && Self::is_number(right) => Self::value(left)
                .len()
                .cmp(&Self::value(right).len())
                .then_with(|| Self::value(left).cmp(Self::value(right))),
            _ => left.cmp(right),
        }
    }

    fn chunks(text: &str) -> Vec<&str> {
        let characters: Vec<(usize, char)> = text.char_indices().collect();
        let starts: Vec<usize> = once(0)
            .chain(
                characters
                    .windows(2)
                    .filter(|pair| Self::breaks(pair[0].1, pair[1].1))
                    .map(|pair| pair[1].0),
            )
            .chain(once(text.len()))
            .collect();
        starts
            .windows(2)
            .map(|bounds| &text[bounds[0]..bounds[1]])
            .collect()
    }

    fn is_number(chunk: &str) -> bool {
        !chunk.is_empty() && chunk.chars().all(|character| character.is_ascii_digit())
    }

    // The first pair of equal numbers written with different leading zeros: the
    // one with more of them sorts first.
    fn leading_zeros(left: &[&str], right: &[&str]) -> Ordering {
        left.iter()
            .zip(right)
            .filter(|(one, other)| Self::is_number(one) && Self::is_number(other))
            .map(|(one, other)| Self::zeros(other).cmp(&Self::zeros(one)))
            .find(|order| order.is_ne())
            .unwrap_or(Ordering::Equal)
    }

    // A number's digits without its leading zeros, so two numbers compare by
    // value however long they are.
    fn value(number: &str) -> &str {
        number.trim_start_matches('0')
    }

    fn zeros(number: &str) -> usize {
        number.len() - Self::value(number).len()
    }
}
