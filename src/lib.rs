#![deny(warnings)]

#[macro_use]
extern crate anyhow;

pub mod cache;
pub mod compression;
pub mod config;
pub mod cors;
pub mod filters;
pub mod helpers;
pub mod logger;
pub mod rejection;
pub mod server;
pub mod signals;

#[macro_use]
pub mod error;

pub use config::{Config, CONFIG};
pub use error::*;
pub use server::Server;
