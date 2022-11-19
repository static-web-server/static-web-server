//! Path-related extension traits.

use std::path::{Component, Path};

/// SWS Path extensions trait.
pub trait PathExt {
    fn is_hidden(&self) -> bool;
}

impl PathExt for Path {
    /// Checks if the current path is hidden (dot file).
    fn is_hidden(&self) -> bool {
        self.components()
            .filter_map(|cmp| match cmp {
                Component::Normal(s) => s.to_str(),
                _ => None,
            })
            .any(|s| s.starts_with('.'))
    }
}
