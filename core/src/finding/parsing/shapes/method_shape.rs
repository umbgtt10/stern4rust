// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MethodShape {
    Behaviour,
    Build,
    Getter,
    PlainConstructor,
    Step,
}
