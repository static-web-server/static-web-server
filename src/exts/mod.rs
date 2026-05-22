// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Extension traits and utilities for HTTP, MIME, and header types.
//!
//! This module groups all "extension" code that augments external types
//! (e.g., `hyper::Method`, `mime_guess::Mime`) with SWS-specific behaviour.

pub(crate) mod headers;
pub mod http;
#[allow(dead_code)]
pub(crate) mod mime;
