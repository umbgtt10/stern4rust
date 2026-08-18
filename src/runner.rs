// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use anyhow::Result;

use crate::args::Args;
use crate::config::Config;

pub struct Runner;

impl Runner {
    pub fn run(args: Args) -> Result<()> {
        let config = Config {
            manifest_path: args.manifest_path,
            packages: args.packages,
        };

        println!("stern4rust report");
        println!();
        println!("No rule is implemented yet, so nothing can fail.");
        println!(
            "summary: packages_requested={} manifest_path={}",
            config.packages.len(),
            config
                .manifest_path
                .as_deref()
                .map_or_else(|| "<default>".into(), |path| path.display().to_string())
        );

        Ok(())
    }
}
