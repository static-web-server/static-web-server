// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Internal CLI functionality.

use shadow_rs::shadow;

use crate::Result;

shadow!(build);

/// Show details about the server.
/// It is intended to be used via the `-V` or `--version` flags.
pub fn display_version() -> Result {
    println!("Version:      {}", build::PKG_VERSION);
    println!("Built:        {}", build::COMMIT_DATE);
    println!("Git commit:   {}", build::COMMIT_HASH);
    println!("Build target: {}", build::BUILD_TARGET);
    println!("Rust version: {}", build::RUST_VERSION);
    println!("License:      {}", env!("CARGO_PKG_LICENSE"));
    println!("Homepage:     {}", env!("CARGO_PKG_HOMEPAGE"));
    println!("Author:       {}", env!("CARGO_PKG_AUTHORS"));

    Ok(())
}
