// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

#[cfg(all(target_env = "musl", target_pointer_width = "64"))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

use static_web_server::{Result, Settings};

fn main() -> Result {
    let opts = Settings::get(true)?;

    #[cfg(windows)]
    {
        use static_web_server::settings::Commands;
        use static_web_server::winservice;

        if let Some(commands) = opts.general.commands {
            match commands {
                Commands::Install {} => {
                    return winservice::install_service(&opts.general.config_file);
                }
                Commands::Uninstall {} => {
                    return winservice::uninstall_service();
                }
            }
        } else if opts.general.windows_service {
            return winservice::run_server_as_service();
        }
    }

    // Run the server by default
    static_web_server::Server::new(opts)?.run_standalone()?;

    Ok(())
}
