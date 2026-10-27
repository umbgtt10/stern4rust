// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::settings::config_failure::ConfigFailure;
use crate::settings::config_file::ConfigFile;
use crate::source_path::SourcePath;

// `stern4rust.toml` in a directory: `None` where there is none, a failure where
// one is there and cannot be read or understood.
pub trait ConfigSource {
    fn load(&self, directory: &SourcePath) -> Result<Option<ConfigFile>, ConfigFailure>;
}
