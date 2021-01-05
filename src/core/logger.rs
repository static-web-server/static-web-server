use env_logger::Builder;
use log::LevelFilter;
use std::io::Write;

/// Initialize logging builder with its levels
pub fn init(level: &str) {
    let log_level = match level {
        "off" => LevelFilter::Off,
        "error" => LevelFilter::Error,
        "warn" => LevelFilter::Warn,
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        _ => {
            println!("Log level \"{}\" is not supported", level);
            std::process::exit(1);
        }
    };

    Builder::new()
        .filter_level(log_level)
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] - {}",
                chrono::Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .init();
}

// `note` is a custom log level macro which is not affected by env log level.
// It takes precedence over other log levels
#[macro_export]
macro_rules! note {
    ($($arg:tt)*) => ({
        println!(
            "{} [NOTE] - {}",
            chrono::Local::now().format("%Y-%m-%dT%H:%M:%S"),
            format!($($arg)*)
        );
    })
}
