// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Module to append custom HTTP headers via TOML config file.
//!

use hyper::{Request, Response};
use std::{ffi::OsStr, path::PathBuf};

use crate::body::Body;
use crate::{
    Error,
    handler_opts::RequestHandlerOpts,
    settings::{Headers, HeadersScope},
};

/// Appends custom HTTP headers to a response if necessary.
///
/// `fallback_path` is set when the response is the fallback page, which has no file
/// path of its own: rules whose `source` matches it apply too.
pub(crate) fn post_process<T>(
    opts: &RequestHandlerOpts,
    req: &Request<T>,
    mut resp: Response<Body>,
    file_path: Option<&PathBuf>,
    early: bool,
    fallback_path: Option<&str>,
) -> Result<Response<Body>, Error> {
    if let Some(advanced) = &opts.advanced_opts {
        append_headers(
            req.uri().path(),
            advanced.headers.as_deref(),
            &mut resp,
            file_path,
            opts.redirect_trailing_slash,
            early,
            fallback_path,
        )
    }
    Ok(resp)
}

/// Append custom HTTP headers to current response.
fn append_headers(
    uri_path: &str,
    headers_opts: Option<&[Headers]>,
    resp: &mut Response<Body>,
    file_path: Option<&PathBuf>,
    redirect_trailing_slash: bool,
    early: bool,
    fallback_path: Option<&str>,
) {
    if let Some(headers_vec) = headers_opts {
        let uri_path_auto_index = file_path
            .filter(|_| uri_path.ends_with('/') || !redirect_trailing_slash)
            .and_then(|p| p.file_name())
            .and_then(OsStr::to_str)
            .map(|name| match uri_path {
                "/" => ["/", name].concat(),
                _ => [uri_path, "/", name].concat(),
            });

        let uri_path = match uri_path_auto_index {
            Some(ref s) => s.as_str(),
            _ => uri_path,
        };

        let status = resp.status();
        for headers_entry in headers_vec {
            // Skip the entry if it does not reach responses returned before the file handler
            if early && headers_entry.scope == HeadersScope::Files {
                continue;
            }
            // Skip the entry if it is limited to other response status codes
            if headers_entry
                .status
                .as_deref()
                .is_some_and(|codes| !codes.contains(&status))
            {
                continue;
            }
            // Match header glob pattern against request uri, or the fallback page's name
            if headers_entry.source.is_match(uri_path)
                || fallback_path.is_some_and(|path| headers_entry.source.is_match(path))
            {
                // Add/update headers if uri matches
                for (name, value) in &headers_entry.headers {
                    resp.headers_mut().insert(name, value.to_owned());
                }
            }
        }
    }
}
