#![deny(warnings)]
#![deny(rust_2018_idioms)]

#[macro_use]
extern crate log;

pub mod config;
pub mod error_page;
pub mod gzip;
pub mod helpers;
pub mod logger;
pub mod server;
pub mod signals;
pub mod staticfile_middleware;
pub mod staticfiles;

pub use config::Options;
