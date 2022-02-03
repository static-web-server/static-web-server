#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

#[macro_use]
extern crate anyhow;

pub mod basic_auth;
pub mod compression;
pub mod config;
pub mod control_headers;
pub mod cors;
pub mod error_page;
pub mod handler;
pub mod helpers;
pub mod logger;
pub mod security_headers;
pub mod server;
pub mod service;
pub mod signals;
pub mod static_files;
pub mod tls;
pub mod transport;

#[macro_use]
pub mod error;

pub use config::Config;
pub use error::*;
pub use server::Server;
