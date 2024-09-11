// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Additional types for the headers module in order to handle Accept-Encoding
//! header.
//!

#![allow(unused)]

mod accept_encoding;
mod content_coding;
mod quality_value;

pub(crate) use accept_encoding::AcceptEncoding;
pub(crate) use content_coding::ContentCoding;
pub(crate) use quality_value::QualityValue;
