#![deny(warnings)]

#[macro_use]
extern crate anyhow;

pub mod config;
pub mod controller;
pub mod error_page;
pub mod fs;
pub mod helpers;
pub mod logger;
pub mod server;
pub mod signals;

#[macro_use]
pub mod error;

pub use config::{Config, CONFIG};
pub use error::*;
pub use server::Server;
