#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

#[cfg(all(target_env = "musl", target_pointer_width = "64"))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

use static_web_server::Result;

fn main() -> Result {
    use static_web_server::settings::Commands;
    use static_web_server::Settings;

    // Get server config
    let opts = Settings::get()?;

    if let Some(commands) = opts.general.commands {
        match commands {
            Commands::Install {} => {
                #[cfg(windows)]
                return static_web_server::winservice::install_service(opts.general.config_file);

                #[cfg(unix)]
                println!("ignored: the `install` command is only available for Windows");
            }
            Commands::Uninstall {} => {
                #[cfg(windows)]
                return static_web_server::winservice::uninstall_service();

                #[cfg(unix)]
                println!("ignored: the `uninstall` command is only available for Windows");
            }
        }
    } else if opts.general.as_windows_service {
        #[cfg(windows)]
        return static_web_server::winservice::run_server_as_service();

        #[cfg(unix)]
        println!("ignored: the `--as-windows-service` option is only available for Windows");
    }

    // Run the server by default
    static_web_server::Server::new()?.run_standalone()?;

    Ok(())
}
