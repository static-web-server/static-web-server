use chrono::Local;
use env_logger::Builder;
use iron::prelude::*;
use iron::AfterMiddleware;
use log::LevelFilter;
use std::io::Write;

/// Initialize logging builder and format
pub fn init(log_level_str: &str) {
    let log_level = match log_level_str {
        "off" => LevelFilter::Off,
        "error" => LevelFilter::Error,
        "warn" => LevelFilter::Warn,
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        _ => {
            println!("Log level \"{}\" is not supported", log_level_str);
            std::process::exit(1);
        }
    };

    Builder::new()
        .filter_level(log_level)
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] - {}",
                Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .init();
}

/// Print specific log info for the server which doesn't depend on any level
pub fn log_server(msg: &str) {
    println!(
        "{} [SERVER] - {}",
        Local::now().format("%Y-%m-%dT%H:%M:%S"),
        &msg
    );
}

pub struct Logger;

impl AfterMiddleware for Logger {
    fn after(&self, req: &mut Request, res: Response) -> IronResult<Response> {
        info!(
            "Request [{}] {} - {}",
            req.method,
            res.status
                .into_iter()
                .map(|i| i.to_string())
                .collect::<String>(),
            req.url
                .path()
                .into_iter()
                .map(|i| format!("/{}", i))
                .collect::<String>()
        );

        Ok(res)
    }
}
