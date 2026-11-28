// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Module to append custom HTTP headers via TOML config file.
//!
//! A `[[advanced.headers]]` source is matched against the path of the request and against
//! the path of the file the response serves, both in the spelling of
//! `fs::path::rule_path`. So the headers written for a file reach every request that is
//! answered with it: `/dir/` for `/dir/index.html`, `/page` for `/page.html`, or any
//! other spelling of its path.

use hyper::{Request, Response, StatusCode};
use percent_encoding::percent_encode;
use std::path::Path;

use crate::body::Body;
use crate::fs::path::{RULE_PATH_ENCODE, rule_path};
use crate::{
    Error,
    handler_opts::RequestHandlerOpts,
    settings::{Headers, HeadersScope},
};

/// Returns the custom headers entries, if any are configured.
fn entries(opts: &RequestHandlerOpts) -> Option<&[Headers]> {
    opts.advanced_opts.as_ref()?.headers.as_deref()
}

/// Returns the rule path of the file that `resp` serves from `file_path`, which lies
/// under the root `base_path`: `/dir/index.html` for a request for `/dir/`.
///
/// `None` when no entry is configured, when no file is served, and for the redirect that
/// adds a trailing slash to a directory, which serves none either.
pub(crate) fn served_path(
    opts: &RequestHandlerOpts,
    resp: &Response<Body>,
    file_path: Option<&Path>,
    base_path: &Path,
) -> Option<String> {
    entries(opts)?;
    if resp.status() == StatusCode::PERMANENT_REDIRECT {
        return None;
    }
    let tail = file_path?.strip_prefix(base_path).ok()?;
    let mut path = String::new();
    for segment in tail {
        path.push('/');
        path.extend(percent_encode(segment.as_encoded_bytes(), RULE_PATH_ENCODE));
    }
    (!path.is_empty()).then_some(path)
}

/// Appends the custom HTTP headers of every entry that applies to a response.
///
/// An entry applies when its source matches the [`rule_path`] of the request or
/// `served_path`, the one of the file served (see [`served_path`]).
pub(crate) fn post_process<T>(
    opts: &RequestHandlerOpts,
    req: &Request<T>,
    mut resp: Response<Body>,
    served_path: Option<&str>,
    early: bool,
) -> Result<Response<Body>, Error> {
    let Some(entries) = entries(opts) else {
        return Ok(resp);
    };
    let uri_path = rule_path(req.uri().path());
    let served_path = served_path.filter(|path| *path != uri_path);
    let status = resp.status();
    for entry in entries {
        // Skip the entry if it does not reach responses returned before the file handler
        if early && entry.scope == HeadersScope::Files {
            continue;
        }
        // Skip the entry if it is limited to other response status codes
        if entry
            .status
            .as_deref()
            .is_some_and(|codes| !codes.contains(&status))
        {
            continue;
        }
        if entry.source.is_match(&*uri_path)
            || served_path.is_some_and(|path| entry.source.is_match(path))
        {
            set_headers(&mut resp, entry);
        }
    }
    Ok(resp)
}

/// Sets the headers of `entry` on `resp`, replacing the values a header already has.
/// A header that `entry` gives several values keeps them all.
fn set_headers(resp: &mut Response<Body>, entry: &Headers) {
    for name in entry.headers.keys() {
        let mut values = entry.headers.get_all(name).iter().cloned();
        if let Some(value) = values.next() {
            resp.headers_mut().insert(name, value);
        }
        for value in values {
            resp.headers_mut().append(name, value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::served_path;
    use crate::handler_opts::RequestHandlerOpts;
    use crate::settings::{Advanced, Headers, HeadersScope};
    use crate::url_rules::to_matcher;
    use hyper::{Response, StatusCode};
    use std::path::Path;

    fn opts(with_entry: bool) -> RequestHandlerOpts {
        let entry = || Headers {
            source: to_matcher("/**").unwrap(),
            status: None,
            scope: HeadersScope::Files,
            headers: Default::default(),
        };
        RequestHandlerOpts {
            advanced_opts: Some(Advanced {
                headers: with_entry.then(|| vec![entry()]),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    fn served(status: StatusCode, file_path: &str) -> Option<String> {
        let mut resp = Response::new(crate::body::empty());
        *resp.status_mut() = status;
        served_path(
            &opts(true),
            &resp,
            Some(Path::new(file_path)),
            Path::new("/root"),
        )
    }

    #[test]
    fn served_path_is_the_file_path_under_the_root() {
        assert_eq!(
            served(StatusCode::OK, "/root/dir/index.html").as_deref(),
            Some("/dir/index.html")
        );
        assert_eq!(
            served(StatusCode::NOT_MODIFIED, "/root/page.html").as_deref(),
            Some("/page.html")
        );
    }

    #[test]
    fn served_path_takes_rule_path_spelling() {
        assert_eq!(
            served(StatusCode::OK, "/root/café/índex 1.html").as_deref(),
            Some("/caf%C3%A9/%C3%ADndex%201.html")
        );
    }

    #[test]
    fn served_path_is_none_without_a_file_under_the_root() {
        assert_eq!(served(StatusCode::OK, "/root"), None);
        assert_eq!(served(StatusCode::OK, "/other/file.html"), None);
        let resp = Response::new(crate::body::empty());
        assert_eq!(
            served_path(&opts(true), &resp, None, Path::new("/root")),
            None
        );
    }

    #[test]
    fn served_path_is_none_for_the_trailing_slash_redirect() {
        assert_eq!(
            served(StatusCode::PERMANENT_REDIRECT, "/root/dir/index.html"),
            None
        );
    }

    #[test]
    fn served_path_is_none_without_entries() {
        let resp = Response::new(crate::body::empty());
        let file_path = Some(Path::new("/root/file.html"));
        let root = Path::new("/root");
        assert_eq!(served_path(&opts(false), &resp, file_path, root), None);
        let no_advanced = RequestHandlerOpts::default();
        assert_eq!(served_path(&no_advanced, &resp, file_path, root), None);
    }
}
