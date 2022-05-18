#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

#[cfg(all(target_env = "musl", target_pointer_width = "64"))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

use static_web_server::Result;

#[cfg(unix)]
fn main() -> Result {
    use static_web_server::Server;

    Server::new(None)?.run()?;

    Ok(())
}

#[cfg(windows)]
fn main() -> Result {
    use static_web_server::settings::Commands;
    use static_web_server::winservice;
    use static_web_server::Settings;

    // Get server config
    let opts = Settings::get()?;

    if let Some(commands) = opts.general.commands {
        match commands {
            Commands::Install {} => winservice::install_service()?,
            Commands::Uninstall {} => winservice::uninstall_service()?,
        }
    } else if opts.general.as_windows_service {
        winservice::run_server_as_service()?
    } else {
        static_web_server::Server::new(None)?.run()?;
    }

    Ok(())
}
