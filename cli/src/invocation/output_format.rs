// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use clap::ValueEnum;

// Which of the two reports a run emits.
//
// The table stays the default. A tool that changed its human output the moment
// it grew a machine one would break every terminal and every gate script
// already reading it, to serve a consumer that has to opt in anyway.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    #[default]
    Text,
    Json,
}
