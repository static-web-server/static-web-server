//! Useful error type re-exports based on [anyhow][mod@anyhow].
//!

/// Just a `anyhow::Result` type alias.
pub type Result<T = (), E = anyhow::Error> = anyhow::Result<T, E>;

/// Just an `anyhow::Error` type alias.
pub type Error = anyhow::Error;

/// Just re-export some `anyhow` stuff.
pub use anyhow::anyhow;
pub use anyhow::bail;
pub use anyhow::Context;
