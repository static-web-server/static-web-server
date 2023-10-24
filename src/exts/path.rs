// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Path-related extension traits.

use std::path::{Component, Path};

/// SWS Path extensions trait.
pub trait PathExt {
    /// If file path is hidden.
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
