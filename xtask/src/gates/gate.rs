// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

pub trait Gate {
    fn label(&self) -> String;

    fn run(&self) -> Result<(), String>;
}
