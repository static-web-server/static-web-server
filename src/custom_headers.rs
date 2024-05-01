// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Module to append custom HTTP headers via TOML config file.
//!

use hyper::{Body, Request, Response};
use std::{ffi::OsStr, path::PathBuf};

use crate::{handler::RequestHandlerOpts, settings::Headers, Error};

/// Appends custom HTTP headers to a response if necessary
pub(crate) fn post_process<T>(
    opts: &RequestHandlerOpts,
    req: &Request<T>,
    mut resp: Response<Body>,
    file_path: Option<&PathBuf>,
) -> Result<Response<Body>, Error> {
    if let Some(advanced) = &opts.advanced_opts {
        append_headers(
            req.uri().path(),
            advanced.headers.as_deref(),
            &mut resp,
            file_path,
        )
    }
    Ok(resp)
}

/// Append custom HTTP headers to current response.
pub fn append_headers(
    uri_path: &str,
    headers_opts: Option<&[Headers]>,
    resp: &mut Response<Body>,
    file_path: Option<&PathBuf>,
) {
    if let Some(headers_vec) = headers_opts {
        let mut uri_path_auto_index = None;
        if file_path.is_some() && uri_path.ends_with('/') {
            if let Some(name) = file_path.unwrap().file_name().and_then(OsStr::to_str) {
                if uri_path == "/" {
                    uri_path_auto_index = Some([uri_path, name].concat())
                } else {
                    uri_path_auto_index = Some([uri_path, "/", name].concat())
                }
            }
        }

        let uri_path = match uri_path_auto_index {
            Some(ref s) => s.as_str(),
            _ => uri_path,
        };

        for headers_entry in headers_vec {
            // Match header glob pattern against request uri
            if headers_entry.source.is_match(uri_path) {
                // Add/update headers if uri matches
                for (name, value) in &headers_entry.headers {
                    resp.headers_mut().insert(name, value.to_owned());
                }
            }
        }
    }
}
