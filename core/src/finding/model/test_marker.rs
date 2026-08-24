// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The phase or phases one marker comment names. Plain data: a marker is read,
// never asked anything.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkerPhase {
    Arrange,
    Act,
    Assert,
}

// One AAA marker comment, read as the phases it names.
//
// Two properties do the work, and both were taken from what the family already
// writes rather than from the standard as stated.
//
// A marker may carry **trailing prose** -- `// Arrange -- four nodes`,
// `// Act: heal the partition`, `// Assert. every node commits`. All three
// punctuations appear across the repositories, and demanding a bare marker would
// report the tests that took the trouble to explain themselves.
//
// A marker ends on a **word boundary**. Without it `// Actually this needs
// explaining` is an Act, and the sequence a test reads as becomes nonsense.
pub struct TestMarker {
    pub line: usize,
    pub label: String,
    pub phases: Vec<MarkerPhase>,
}

impl TestMarker {
    // Longest first, so `Arrange & Act` is never read as a bare `Arrange`.
    pub const FORMS: [&'static str; 6] = [
        "Arrange & Act & Assert",
        "Arrange & Act",
        "Act & Assert",
        "Arrange",
        "Assert",
        "Act",
    ];

    pub fn parse(text: &str, line: usize) -> Option<Self> {
        let comment = text.trim_start().strip_prefix("//")?.trim_start();
        let form = Self::form_of(comment)?;
        Some(Self {
            line,
            label: format!("// {form}"),
            phases: Self::phases_of(form),
        })
    }

    fn form_of(comment: &str) -> Option<&'static str> {
        Self::FORMS
            .into_iter()
            .find(|form| Self::is_form(comment, form))
    }

    // The rest may be nothing, or anything that does not continue the word.
    fn is_form(comment: &str, form: &str) -> bool {
        comment.strip_prefix(form).is_some_and(|rest| {
            rest.chars()
                .next()
                .is_none_or(|next| !next.is_alphanumeric() && next != '_')
        })
    }

    fn phases_of(form: &str) -> Vec<MarkerPhase> {
        match form {
            "Arrange & Act & Assert" => {
                vec![MarkerPhase::Arrange, MarkerPhase::Act, MarkerPhase::Assert]
            }
            "Arrange & Act" => vec![MarkerPhase::Arrange, MarkerPhase::Act],
            "Act & Assert" => vec![MarkerPhase::Act, MarkerPhase::Assert],
            "Arrange" => vec![MarkerPhase::Arrange],
            "Assert" => vec![MarkerPhase::Assert],
            _ => vec![MarkerPhase::Act],
        }
    }
}
