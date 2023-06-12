// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Request handler module intended to manage incoming HTTP requests.
//!

use headers::HeaderValue;
use hyper::{header::WWW_AUTHENTICATE, Body, Request, Response, StatusCode};
use std::{future::Future, net::IpAddr, net::SocketAddr, path::PathBuf, sync::Arc};

#[cfg(feature = "compression")]
use crate::compression;

use crate::{
    basic_auth, control_headers, cors, custom_headers, error_page,
    exts::http::MethodExt,
    fallback_page, redirects, rewrites, security_headers,
    settings::Advanced,
    static_files::{self, HandleOpts},
    Error, Result,
};

#[cfg(feature = "directory-listing")]
use crate::directory_listing::DirListFmt;

/// It defines options for a request handler.
pub struct RequestHandlerOpts {
    // General options
    /// Root directory of static files.
    pub root_dir: PathBuf,
    /// Compression feature.
    pub compression: bool,
    /// Compression static feature.
    pub compression_static: bool,
    /// Directory listing feature.
    #[cfg(feature = "directory-listing")]
    pub dir_listing: bool,
    /// Directory listing order feature.
    #[cfg(feature = "directory-listing")]
    pub dir_listing_order: u8,
    #[cfg(feature = "directory-listing")]
    /// Directory listing format feature.
    pub dir_listing_format: DirListFmt,
    /// CORS feature.
    pub cors: Option<cors::Configured>,
    /// Security headers feature.
    pub security_headers: bool,
    /// Cache control headers feature.
    pub cache_control_headers: bool,
    /// Page for 404 errors.
    pub page404: Vec<u8>,
    /// Page for 50x errors.
    pub page50x: Vec<u8>,
    /// Page fallback feature.
    pub page_fallback: Vec<u8>,
    /// Basic auth feature.
    pub basic_auth: String,
    /// Log remote address feature.
    pub log_remote_address: bool,
    /// Redirect trailing slash feature.
    pub redirect_trailing_slash: bool,
    /// Ignore hidden files feature.
    pub ignore_hidden_files: bool,

    /// Advanced options from the config file.
    pub advanced_opts: Option<Advanced>,
}

/// It defines the main request handler used by the Hyper service request.
pub struct RequestHandler {
    /// Request handler options.
    pub opts: Arc<RequestHandlerOpts>,
}

impl RequestHandler {
    /// Main entry point for incoming requests.
    pub fn handle<'a>(
        &'a self,
        req: &'a mut Request<Body>,
        remote_addr: Option<SocketAddr>,
    ) -> impl Future<Output = Result<Response<Body>, Error>> + Send + 'a {
        let method = req.method();
        let headers = req.headers();
        let uri = req.uri();

        let base_path = &self.opts.root_dir;
        let mut uri_path = uri.path();
        let uri_query = uri.query();
        #[cfg(feature = "directory-listing")]
        let dir_listing = self.opts.dir_listing;
        #[cfg(feature = "directory-listing")]
        let dir_listing_order = self.opts.dir_listing_order;
        #[cfg(feature = "directory-listing")]
        let dir_listing_format = &self.opts.dir_listing_format;
        let log_remote_addr = self.opts.log_remote_address;
        let redirect_trailing_slash = self.opts.redirect_trailing_slash;
        let compression_static = self.opts.compression_static;
        let ignore_hidden_files = self.opts.ignore_hidden_files;

        let mut cors_headers: Option<http::HeaderMap> = None;

        // Log request information with its remote address if available
        let mut remote_addr_str = String::new();
        if log_remote_addr {
            remote_addr_str.push_str(" remote_addr=");
            remote_addr_str.push_str(&remote_addr.map_or("".to_owned(), |v| v.to_string()));

            if let Some(client_ip_address) = headers
                .get("X-Forwarded-For")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.split(',').next())
                .and_then(|s| s.trim().parse::<IpAddr>().ok())
            {
                remote_addr_str.push_str(" real_remote_ip=");
                remote_addr_str.push_str(&client_ip_address.to_string())
            }
        }
        tracing::info!(
            "incoming request: method={} uri={}{}",
            method,
            uri,
            remote_addr_str,
        );

        async move {
            // Reject in case of incoming HTTP request method is not allowed
            if !method.is_allowed() {
                return error_page::error_response(
                    uri,
                    method,
                    &StatusCode::METHOD_NOT_ALLOWED,
                    &self.opts.page404,
                    &self.opts.page50x,
                );
            }

            // CORS
            if let Some(cors) = &self.opts.cors {
                match cors.check_request(method, headers) {
                    Ok((headers, state)) => {
                        tracing::debug!("cors state: {:?}", state);
                        cors_headers = Some(headers);
                    }
                    Err(err) => {
                        tracing::error!("cors error kind: {:?}", err);
                        return error_page::error_response(
                            uri,
                            method,
                            &StatusCode::FORBIDDEN,
                            &self.opts.page404,
                            &self.opts.page50x,
                        );
                    }
                };
            }

            // `Basic` HTTP Authorization Schema
            if !self.opts.basic_auth.is_empty() {
                if let Some((user_id, password)) = self.opts.basic_auth.split_once(':') {
                    if let Err(err) = basic_auth::check_request(headers, user_id, password) {
                        tracing::warn!("basic authentication failed {:?}", err);
                        let mut resp = error_page::error_response(
                            uri,
                            method,
                            &StatusCode::UNAUTHORIZED,
                            &self.opts.page404,
                            &self.opts.page50x,
                        )?;
                        resp.headers_mut().insert(
                            WWW_AUTHENTICATE,
                            "Basic realm=\"Static Web Server\", charset=\"UTF-8\""
                                .parse()
                                .unwrap(),
                        );
                        return Ok(resp);
                    }
                } else {
                    tracing::error!("invalid basic authentication `user_id:password` pairs");
                    return error_page::error_response(
                        uri,
                        method,
                        &StatusCode::INTERNAL_SERVER_ERROR,
                        &self.opts.page404,
                        &self.opts.page50x,
                    );
                }
            }

            // Advanced options
            if let Some(advanced) = &self.opts.advanced_opts {
                // Redirects
                if let Some(parts) = redirects::get_redirection(uri_path, &advanced.redirects) {
                    let (uri_dest, status) = parts;
                    match HeaderValue::from_str(uri_dest) {
                        Ok(loc) => {
                            let mut resp = Response::new(Body::empty());
                            resp.headers_mut().insert(hyper::header::LOCATION, loc);
                            *resp.status_mut() = *status;
                            tracing::trace!(
                                "uri matches redirect pattern, redirecting with status {}",
                                status.canonical_reason().unwrap_or_default()
                            );
                            return Ok(resp);
                        }
                        Err(err) => {
                            tracing::error!("invalid header value from current uri: {:?}", err);
                            return error_page::error_response(
                                uri,
                                method,
                                &StatusCode::INTERNAL_SERVER_ERROR,
                                &self.opts.page404,
                                &self.opts.page50x,
                            );
                        }
                    };
                }

                // Rewrites
                if let Some(uri) = rewrites::rewrite_uri_path(uri_path, &advanced.rewrites) {
                    uri_path = uri
                }
            }

            // Static files
            match static_files::handle(&HandleOpts {
                method,
                headers,
                base_path,
                uri_path,
                uri_query,
                #[cfg(feature = "directory-listing")]
                dir_listing,
                #[cfg(feature = "directory-listing")]
                dir_listing_order,
                #[cfg(feature = "directory-listing")]
                dir_listing_format,
                redirect_trailing_slash,
                compression_static,
                ignore_hidden_files,
            })
            .await
            {
                Ok((mut resp, _is_precompressed)) => {
                    // Append CORS headers if they are present
                    if let Some(cors_headers) = cors_headers {
                        if !cors_headers.is_empty() {
                            for (k, v) in cors_headers.iter() {
                                resp.headers_mut().insert(k, v.to_owned());
                            }
                            resp.headers_mut().remove(http::header::ALLOW);
                        }
                    }

                    // Compression content encoding varies so use a `Vary` header
                    #[cfg(feature = "compression")]
                    if self.opts.compression || compression_static {
                        resp.headers_mut().append(
                            hyper::header::VARY,
                            hyper::header::HeaderValue::from_name(hyper::header::ACCEPT_ENCODING),
                        );
                    }

                    // Auto compression based on the `Accept-Encoding` header
                    #[cfg(feature = "compression")]
                    if self.opts.compression && !_is_precompressed {
                        resp = match compression::auto(method, headers, resp) {
                            Ok(res) => res,
                            Err(err) => {
                                tracing::error!("error during body compression: {:?}", err);
                                return error_page::error_response(
                                    uri,
                                    method,
                                    &StatusCode::INTERNAL_SERVER_ERROR,
                                    &self.opts.page404,
                                    &self.opts.page50x,
                                );
                            }
                        };
                    }

                    // Append `Cache-Control` headers for web assets
                    if self.opts.cache_control_headers {
                        control_headers::append_headers(uri_path, &mut resp);
                    }

                    // Append security headers
                    if self.opts.security_headers {
                        security_headers::append_headers(&mut resp);
                    }

                    // Add/update custom headers
                    if let Some(advanced) = &self.opts.advanced_opts {
                        custom_headers::append_headers(uri_path, &advanced.headers, &mut resp)
                    }

                    Ok(resp)
                }
                Err(status) => {
                    // Check for a fallback response
                    if method.is_get()
                        && status == StatusCode::NOT_FOUND
                        && !self.opts.page_fallback.is_empty()
                    {
                        // We use all modules as usual when the `page-fallback` feature is enabled
                        let mut resp = fallback_page::fallback_response(&self.opts.page_fallback);

                        // Append CORS headers if they are present
                        if let Some(cors_headers) = cors_headers {
                            if !cors_headers.is_empty() {
                                for (k, v) in cors_headers.iter() {
                                    resp.headers_mut().insert(k, v.to_owned());
                                }
                                resp.headers_mut().remove(http::header::ALLOW);
                            }
                        }

                        // Compression content encoding varies so use a `Vary` header
                        #[cfg(feature = "compression")]
                        if self.opts.compression || compression_static {
                            resp.headers_mut().append(
                                hyper::header::VARY,
                                hyper::header::HeaderValue::from_name(
                                    hyper::header::ACCEPT_ENCODING,
                                ),
                            );
                        }

                        // Auto compression based on the `Accept-Encoding` header
                        #[cfg(feature = "compression")]
                        if self.opts.compression {
                            resp = match compression::auto(method, headers, resp) {
                                Ok(res) => res,
                                Err(err) => {
                                    tracing::error!("error during body compression: {:?}", err);
                                    return error_page::error_response(
                                        uri,
                                        method,
                                        &StatusCode::INTERNAL_SERVER_ERROR,
                                        &self.opts.page404,
                                        &self.opts.page50x,
                                    );
                                }
                            };
                        }

                        // Append `Cache-Control` headers for web assets
                        if self.opts.cache_control_headers {
                            control_headers::append_headers(uri_path, &mut resp);
                        }

                        // Append security headers
                        if self.opts.security_headers {
                            security_headers::append_headers(&mut resp);
                        }

                        // Add/update custom headers
                        if let Some(advanced) = &self.opts.advanced_opts {
                            custom_headers::append_headers(uri_path, &advanced.headers, &mut resp)
                        }

                        return Ok(resp);
                    }

                    // Otherwise return a response error
                    error_page::error_response(
                        uri,
                        method,
                        &status,
                        &self.opts.page404,
                        &self.opts.page50x,
                    )
                }
            }
        }
    }
}
