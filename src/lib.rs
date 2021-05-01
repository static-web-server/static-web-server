#![deny(warnings)]

#[macro_use]
extern crate anyhow;

pub mod config;
pub mod error_page;
pub mod handler;
pub mod helpers;
pub mod logger;
pub mod server;
pub mod signals;
pub mod static_files;

#[macro_use]
pub mod error;

pub use config::{Config, CONFIG};
pub use error::*;
pub use server::Server;
