// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::support::in_memory_source_tree::NotHeld;
use stern4rust::settings::config_failure::ConfigFailure;
use stern4rust::settings::config_file::ConfigFile;
use stern4rust::source_path::SourcePath;
use stern4rust::traits::config_source::ConfigSource;

// A `stern4rust.toml` already understood: none, one, or one that cannot be read.
pub struct InMemoryConfigSource {
    file: Option<ConfigFile>,
    unreadable: bool,
}

impl InMemoryConfigSource {
    pub fn absent() -> Self {
        Self {
            file: None,
            unreadable: false,
        }
    }

    pub fn holding(file: ConfigFile) -> Self {
        Self {
            file: Some(file),
            unreadable: false,
        }
    }

    pub fn unreadable() -> Self {
        Self {
            file: None,
            unreadable: true,
        }
    }
}

impl ConfigSource for InMemoryConfigSource {
    fn load(&self, directory: &SourcePath) -> Result<Option<ConfigFile>, ConfigFailure> {
        if self.unreadable {
            return Err(ConfigFailure::Unreadable {
                path: directory.join(ConfigFile::NAME),
                cause: Box::new(NotHeld),
            });
        }
        Ok(self.file.clone())
    }
}
