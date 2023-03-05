#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

#[macro_use]
extern crate anyhow;

#[macro_use]
extern crate serde;

pub mod basic_auth;
pub mod compression;
pub mod compression_static;
pub mod control_headers;
pub mod cors;
pub mod custom_headers;
pub mod directory_listing;
pub mod error_page;
pub mod exts;
pub mod fallback_page;
pub mod handler;
pub mod helpers;
pub mod logger;
pub mod redirects;
pub mod rewrites;
pub mod security_headers;
pub mod server;
pub mod service;
pub mod settings;
#[cfg(any(unix, windows))]
pub mod signals;
pub mod static_files;
#[cfg(feature = "tls")]
pub mod tls;
pub mod transport;

#[cfg(windows)]
pub mod winservice;

#[macro_use]
pub mod error;

pub use error::*;
pub use server::Server;
pub use settings::Settings;
