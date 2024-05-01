// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Basic HTTP Authorization Schema module.
//!

use bcrypt::verify as bcrypt_verify;
use headers::{authorization::Basic, Authorization, HeaderMap, HeaderMapExt};
use hyper::{header::WWW_AUTHENTICATE, Body, Request, Response, StatusCode};

use crate::{error_page, handler::RequestHandlerOpts, http_ext::MethodExt, Error};

/// Initializes `Basic` HTTP Authorization handling
pub(crate) fn init(credentials: &str, handler_opts: &mut RequestHandlerOpts) {
    credentials.trim().clone_into(&mut handler_opts.basic_auth);
    server_info!(
        "basic authentication: enabled={}",
        !handler_opts.basic_auth.is_empty()
    );
}

/// Handles `Basic` HTTP Authorization Schema
pub(crate) fn pre_process<T>(
    opts: &RequestHandlerOpts,
    req: &Request<T>,
) -> Option<Result<Response<Body>, Error>> {
    if opts.basic_auth.is_empty() {
        return None;
    }

    let method = req.method();
    if method.is_options() {
        return None;
    }

    let uri = req.uri();
    if let Some((user_id, password)) = opts.basic_auth.split_once(':') {
        let err = check_request(req.headers(), user_id, password).err()?;
        tracing::warn!("basic authentication failed {:?}", err);
        let mut result = error_page::error_response(
            uri,
            method,
            &StatusCode::UNAUTHORIZED,
            &opts.page404,
            &opts.page50x,
        );
        if let Ok(ref mut resp) = result {
            resp.headers_mut().insert(
                WWW_AUTHENTICATE,
                "Basic realm=\"Static Web Server\", charset=\"UTF-8\""
                    .parse()
                    .unwrap(),
            );
        }
        Some(result)
    } else {
        tracing::error!("invalid basic authentication `user_id:password` pairs");
        Some(error_page::error_response(
            uri,
            method,
            &StatusCode::INTERNAL_SERVER_ERROR,
            &opts.page404,
            &opts.page50x,
        ))
    }
}

/// Check for a `Basic` HTTP Authorization Schema of an incoming request
/// and uses `bcrypt` for password hashing verification.
pub fn check_request(headers: &HeaderMap, userid: &str, password: &str) -> Result<(), StatusCode> {
    let credentials = headers
        .typed_get::<Authorization<Basic>>()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if credentials.0.username() != userid {
        return Err(StatusCode::UNAUTHORIZED);
    }

    match bcrypt_verify(credentials.0.password(), password) {
        Ok(valid) if valid => Ok(()),
        Ok(_) => Err(StatusCode::UNAUTHORIZED),
        Err(err) => {
            tracing::error!("bcrypt password verification error: {:?}", err);
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{check_request, pre_process};
    use crate::{handler::RequestHandlerOpts, Error};
    use headers::HeaderMap;
    use hyper::{header::WWW_AUTHENTICATE, Body, Request, Response, StatusCode};

    fn make_request(method: &str, auth_header: &str) -> Request<Body> {
        let mut builder = Request::builder();
        if !auth_header.is_empty() {
            builder = builder.header("Authorization", auth_header);
        }
        builder.method(method).uri("/").body(Body::empty()).unwrap()
    }

    fn is_401(result: Option<Result<Response<Body>, Error>>) -> bool {
        if let Some(Ok(response)) = result {
            response.status() == StatusCode::UNAUTHORIZED
                && response.headers().get(WWW_AUTHENTICATE).is_some()
        } else {
            false
        }
    }

    fn is_500(result: Option<Result<Response<Body>, Error>>) -> bool {
        if let Some(Ok(response)) = result {
            response.status() == StatusCode::INTERNAL_SERVER_ERROR
        } else {
            false
        }
    }

    #[test]
    fn test_auth_disabled() {
        assert!(pre_process(
            &RequestHandlerOpts {
                basic_auth: "".into(),
                ..Default::default()
            },
            &make_request("GET", "Basic anE6anE=")
        )
        .is_none());
    }

    #[test]
    fn test_invalid_auth_configuration() {
        assert!(is_500(pre_process(
            &RequestHandlerOpts {
                basic_auth: "xyz".into(),
                ..Default::default()
            },
            &make_request("GET", "Basic anE6anE=")
        )));
    }

    #[test]
    fn test_options_request() {
        assert!(pre_process(
            &RequestHandlerOpts {
                basic_auth: "jq:$2y$05$32zazJ1yzhlDHnt26L3MFOgY0HVqPmDUvG0KUx6cjf9RDiUGp/M9q"
                    .into(),
                ..Default::default()
            },
            &make_request("OPTIONS", "")
        )
        .is_none());
    }

    #[test]
    fn test_valid_auth() {
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", "Basic anE6anE=".parse().unwrap());
        assert!(check_request(
            &headers,
            "jq",
            "$2y$05$32zazJ1yzhlDHnt26L3MFOgY0HVqPmDUvG0KUx6cjf9RDiUGp/M9q"
        )
        .is_ok());

        assert!(pre_process(
            &RequestHandlerOpts {
                basic_auth: "jq:$2y$05$32zazJ1yzhlDHnt26L3MFOgY0HVqPmDUvG0KUx6cjf9RDiUGp/M9q"
                    .into(),
                ..Default::default()
            },
            &make_request("GET", "Basic anE6anE=")
        )
        .is_none());
    }

    #[test]
    fn test_invalid_auth_header() {
        let headers = HeaderMap::new();
        assert!(check_request(&headers, "jq", "").is_err());

        assert!(is_401(pre_process(
            &RequestHandlerOpts {
                basic_auth: "jq:".into(),
                ..Default::default()
            },
            &make_request("GET", "")
        )));
    }

    #[test]
    fn test_invalid_auth_pairs() {
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", "Basic anE6anE=".parse().unwrap());
        assert!(check_request(&headers, "xyz", "").is_err());

        assert!(is_401(pre_process(
            &RequestHandlerOpts {
                basic_auth: "xyz:".into(),
                ..Default::default()
            },
            &make_request("GET", "Basic anE6anE=")
        )));
    }

    #[test]
    fn test_invalid_auth() {
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", "Basic anE6anE=".parse().unwrap());
        assert!(check_request(
            &headers,
            "abc",
            "$2y$05$32zazJ1yzhlDHnt26L3MFOgY0HVqPmDUvG0KUx6cjf9RDiUGp/M9q"
        )
        .is_err());
        assert!(check_request(&headers, "jq", "password").is_err());
        assert!(check_request(&headers, "", "password").is_err());
        assert!(check_request(&headers, "jq", "").is_err());

        assert!(is_401(pre_process(
            &RequestHandlerOpts {
                basic_auth: "abc:$2y$05$32zazJ1yzhlDHnt26L3MFOgY0HVqPmDUvG0KUx6cjf9RDiUGp/M9q"
                    .into(),
                ..Default::default()
            },
            &make_request("GET", "Basic anE6anE=")
        )));
        assert!(is_401(pre_process(
            &RequestHandlerOpts {
                basic_auth: "jq:password".into(),
                ..Default::default()
            },
            &make_request("GET", "Basic anE6anE=")
        )));
        assert!(is_401(pre_process(
            &RequestHandlerOpts {
                basic_auth: ":password".into(),
                ..Default::default()
            },
            &make_request("GET", "Basic anE6anE=")
        )));
        assert!(is_401(pre_process(
            &RequestHandlerOpts {
                basic_auth: "jq:".into(),
                ..Default::default()
            },
            &make_request("GET", "Basic anE6anE=")
        )));
    }

    #[test]
    fn test_invalid_auth_encoding() {
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", "Basic xyz".parse().unwrap());
        assert!(check_request(
            &headers,
            "jq",
            "$2y$05$32zazJ1yzhlDHnt26L3MFOgY0HVqPmDUvG0KUx6cjf9RDiUGp/M9q"
        )
        .is_err());

        assert!(is_401(pre_process(
            &RequestHandlerOpts {
                basic_auth: "jq:$2y$05$32zazJ1yzhlDHnt26L3MFOgY0HVqPmDUvG0KUx6cjf9RDiUGp/M9q"
                    .into(),
                ..Default::default()
            },
            &make_request("GET", "Basic xyz")
        )));
    }

    #[test]
    fn test_invalid_auth_encoding2() {
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", "abcd".parse().unwrap());
        assert!(check_request(
            &headers,
            "jq",
            "$2y$05$32zazJ1yzhlDHnt26L3MFOgY0HVqPmDUvG0KUx6cjf9RDiUGp/M9q"
        )
        .is_err());

        assert!(is_401(pre_process(
            &RequestHandlerOpts {
                basic_auth: "jq:$2y$05$32zazJ1yzhlDHnt26L3MFOgY0HVqPmDUvG0KUx6cjf9RDiUGp/M9q"
                    .into(),
                ..Default::default()
            },
            &make_request("GET", "abcd")
        )));
    }
}
