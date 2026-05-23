// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! In-memory file cache with LFU admission and LRU eviction.
//!
//! Provides file-level caching with configurable capacity, TTL, and TTI.
//! Enabled via the `mem-cache` Cargo feature and configured through the
//! `[advanced.memory-cache]` TOML section.

pub mod cache;
pub(crate) mod stream;
