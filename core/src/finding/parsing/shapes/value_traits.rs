// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

pub struct ValueTraits;

impl ValueTraits {
    const NAMES: [&'static str; 19] = [
        "AsRef",
        "Clone",
        "Copy",
        "Debug",
        "Default",
        "Deserialize",
        "Display",
        "Eq",
        "Error",
        "From",
        "FromStr",
        "Hash",
        "Ord",
        "PartialEq",
        "PartialOrd",
        "Send",
        "Serialize",
        "Sync",
        "TryFrom",
    ];

    pub fn contains(name: &str) -> bool {
        Self::NAMES.contains(&name)
    }
}
