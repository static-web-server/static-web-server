/// Convenient result return type alias of `anyhow::Result`
pub type Result<T = ()> = anyhow::Result<T, anyhow::Error>;

pub use anyhow::Context;
