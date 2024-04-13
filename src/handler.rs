// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Request handler module intended to manage incoming HTTP requests.
//!

use headers::{ContentType, HeaderMap, HeaderMapExt, HeaderValue};
use hyper::{Body, Request, Response, StatusCode};
use std::{future::Future, net::IpAddr, net::SocketAddr, path::PathBuf, sync::Arc};

#[cfg(any(
    feature = "compression",
    feature = "compression-gzip",
    feature = "compression-brotli",
    feature = "compression-zstd",
    feature = "compression-deflate"
))]
use crate::compression;

#[cfg(feature = "basic-auth")]
use {crate::basic_auth, hyper::header::WWW_AUTHENTICATE};

#[cfg(feature = "fallback-page")]
use crate::fallback_page;

use crate::{
    control_headers, cors, custom_headers, error_page,
    http_ext::MethodExt,
    maintenance_mode, redirects, rewrites, security_headers,
    settings::{file::RedirectsKind, Advanced},
    static_files::{self, HandleOpts},
    virtual_hosts, Error, Result,
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
    #[cfg_attr(docsrs, doc(cfg(feature = "directory-listing")))]
    pub dir_listing: bool,
    /// Directory listing order feature.
    #[cfg(feature = "directory-listing")]
    #[cfg_attr(docsrs, doc(cfg(feature = "directory-listing")))]
    pub dir_listing_order: u8,
    #[cfg(feature = "directory-listing")]
    #[cfg_attr(docsrs, doc(cfg(feature = "directory-listing")))]
    /// Directory listing format feature.
    pub dir_listing_format: DirListFmt,
    /// CORS feature.
    pub cors: Option<cors::Configured>,
    /// Security headers feature.
    pub security_headers: bool,
    /// Cache control headers feature.
    pub cache_control_headers: bool,
    /// Page for 404 errors.
    pub page404: PathBuf,
    /// Page for 50x errors.
    pub page50x: PathBuf,
    /// Page fallback feature.
    #[cfg(feature = "fallback-page")]
    #[cfg_attr(docsrs, doc(cfg(feature = "fallback-page")))]
    pub page_fallback: Vec<u8>,
    /// Basic auth feature.
    #[cfg(feature = "basic-auth")]
    #[cfg_attr(docsrs, doc(cfg(feature = "basic-auth")))]
    pub basic_auth: String,
    /// Index files feature.
    pub index_files: Vec<String>,
    /// Log remote address feature.
    pub log_remote_address: bool,
    /// Redirect trailing slash feature.
    pub redirect_trailing_slash: bool,
    /// Ignore hidden files feature.
    pub ignore_hidden_files: bool,
    /// Health endpoint feature.
    pub health: bool,
    /// Metrics endpoint feature (experimental).
    #[cfg(all(unix, feature = "experimental"))]
    pub experimental_metrics: bool,
    /// Maintenance mode feature.
    pub maintenance_mode: bool,
    /// Custom HTTP status for when entering into maintenance mode.
    pub maintenance_mode_status: StatusCode,
    /// Custom maintenance mode HTML file.
    pub maintenance_mode_file: PathBuf,

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

        let mut base_path = &self.opts.root_dir;
        let mut uri_path = uri.path().to_owned();
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
        let health = self.opts.health;
        #[cfg(all(unix, feature = "experimental"))]
        let experimental_metrics = self.opts.experimental_metrics;
        let index_files: Vec<&str> = self.opts.index_files.iter().map(|s| s.as_str()).collect();

        let mut cors_headers: Option<HeaderMap> = None;

        let health_request =
            health && uri_path == "/health" && (method.is_get() || method.is_head());

        #[cfg(all(unix, feature = "experimental"))]
        let metrics_request =
            experimental_metrics && uri_path == "/metrics" && (method.is_get() || method.is_head());

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

        // Health endpoint logs
        if health_request {
            tracing::debug!(
                "incoming request: method={} uri={}{}",
                method,
                uri,
                remote_addr_str,
            );
        } else {
            tracing::info!(
                "incoming request: method={} uri={}{}",
                method,
                uri,
                remote_addr_str,
            );
        }

        let host = headers
            .get(http::header::HOST)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        async move {
            // Health endpoint check
            if health_request {
                let body = if method.is_get() {
                    Body::from("OK")
                } else {
                    Body::empty()
                };
                let mut resp = Response::new(body);
                resp.headers_mut().typed_insert(ContentType::html());
                return Ok(resp);
            }

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

            // Metrics endpoint check
            #[cfg(all(unix, feature = "experimental"))]
            if metrics_request {
                use prometheus::Encoder;

                let body = if method.is_get() {
                    let encoder = prometheus::TextEncoder::new();
                    let mut buffer = Vec::new();
                    encoder
                        .encode(&prometheus::default_registry().gather(), &mut buffer)
                        .unwrap();
                    let data = String::from_utf8(buffer).unwrap();
                    Body::from(data)
                } else {
                    Body::empty()
                };
                let mut resp = Response::new(body);
                resp.headers_mut()
                    .typed_insert(ContentType::from(mime_guess::mime::TEXT_PLAIN_UTF_8));
                return Ok(resp);
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

            #[cfg(feature = "basic-auth")]
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

            // Maintenance Mode
            if self.opts.maintenance_mode {
                return maintenance_mode::get_response(
                    method,
                    &self.opts.maintenance_mode_status,
                    &self.opts.maintenance_mode_file,
                );
            }

            // Advanced options
            if let Some(advanced) = &self.opts.advanced_opts {
                // Redirects
                let mut uri_host = uri.host().unwrap_or(host).to_owned();
                if let Some(uri_port) = uri.port_u16() {
                    uri_host.push_str(&format!(":{}", uri_port));
                }
                if let Some(redirects) = redirects::get_redirection(
                    &uri_host,
                    uri_path.clone().as_str(),
                    advanced.redirects.as_deref(),
                ) {
                    // Redirects: Handle replacements (placeholders)
                    if let Some(regex_caps) = redirects.source.captures(uri_path.as_str()) {
                        let caps_range = 0..regex_caps.len();
                        let caps = caps_range
                            .clone()
                            .filter_map(|i| regex_caps.get(i).map(|s| s.as_str()))
                            .collect::<Vec<&str>>();

                        let patterns = caps_range
                            .map(|i| format!("${}", i))
                            .collect::<Vec<String>>();

                        let dest = redirects.destination.as_str();

                        tracing::debug!("url redirects glob pattern: {:?}", patterns);
                        tracing::debug!("url redirects regex equivalent: {}", redirects.source);
                        tracing::debug!("url redirects glob pattern captures: {:?}", caps);
                        tracing::debug!("url redirects glob pattern destination: {:?}", dest);

                        if let Ok(ac) = aho_corasick::AhoCorasick::new(patterns) {
                            if let Ok(dest) = ac.try_replace_all(dest, &caps) {
                                tracing::debug!(
                                    "url redirects glob pattern destination replaced: {:?}",
                                    dest
                                );
                                uri_path = dest;
                            }
                        }
                    }

                    match HeaderValue::from_str(uri_path.as_str()) {
                        Ok(loc) => {
                            let mut resp = Response::new(Body::empty());
                            resp.headers_mut().insert(hyper::header::LOCATION, loc);
                            *resp.status_mut() = redirects.kind;
                            tracing::trace!(
                                "uri matches redirects glob pattern, redirecting with status '{}'",
                                redirects.kind
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
                if let Some(rewrite) = rewrites::rewrite_uri_path(
                    uri_path.clone().as_str(),
                    advanced.rewrites.as_deref(),
                ) {
                    // Rewrites: Handle replacements (placeholders)
                    if let Some(regex_caps) = rewrite.source.captures(uri_path.as_str()) {
                        let caps_range = 0..regex_caps.len();
                        let caps = caps_range
                            .clone()
                            .filter_map(|i| regex_caps.get(i).map(|s| s.as_str()))
                            .collect::<Vec<&str>>();

                        let patterns = caps_range
                            .map(|i| format!("${}", i))
                            .collect::<Vec<String>>();

                        let dest = rewrite.destination.as_str();

                        tracing::debug!("url rewrites glob pattern: {:?}", patterns);
                        tracing::debug!("url rewrites regex equivalent: {}", rewrite.source);
                        tracing::debug!("url rewrites glob pattern captures: {:?}", caps);
                        tracing::debug!("url rewrites glob pattern destination: {:?}", dest);

                        if let Ok(ac) = aho_corasick::AhoCorasick::new(patterns) {
                            if let Ok(dest) = ac.try_replace_all(dest, &caps) {
                                tracing::debug!(
                                    "url rewrites glob pattern destination replaced: {:?}",
                                    dest
                                );
                                uri_path = dest;
                            }
                        }
                    }

                    // Rewrites: Handle redirections
                    if let Some(redirect_type) = &rewrite.redirect {
                        let loc = match HeaderValue::from_str(uri_path.as_str()) {
                            Ok(val) => val,
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
                        let mut resp = Response::new(Body::empty());
                        resp.headers_mut().insert(hyper::header::LOCATION, loc);
                        *resp.status_mut() = match redirect_type {
                            RedirectsKind::Permanent => StatusCode::MOVED_PERMANENTLY,
                            RedirectsKind::Temporary => StatusCode::FOUND,
                        };
                        return Ok(resp);
                    }
                }

                // If the "Host" header matches any virtual_host, change the root directory
                if let Some(root) =
                    virtual_hosts::get_real_root(headers, advanced.virtual_hosts.as_deref())
                {
                    base_path = root;
                }
            }

            let uri_path = &uri_path;
            let index_files = index_files.as_ref();

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
                index_files,
            })
            .await
            {
                Ok(result) => {
                    let _is_precompressed = result.is_precompressed;
                    let mut resp = result.resp;

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
                    #[cfg(any(
                        feature = "compression",
                        feature = "compression-gzip",
                        feature = "compression-brotli",
                        feature = "compression-zstd",
                        feature = "compression-deflate"
                    ))]
                    if self.opts.compression || compression_static {
                        resp.headers_mut().append(
                            hyper::header::VARY,
                            HeaderValue::from_name(hyper::header::ACCEPT_ENCODING),
                        );
                    }

                    // Auto compression based on the `Accept-Encoding` header
                    #[cfg(any(
                        feature = "compression",
                        feature = "compression-gzip",
                        feature = "compression-brotli",
                        feature = "compression-zstd",
                        feature = "compression-deflate"
                    ))]
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
                        custom_headers::append_headers(
                            uri_path,
                            advanced.headers.as_deref(),
                            &mut resp,
                            Some(&result.file_path),
                        )
                    }

                    Ok(resp)
                }
                Err(status) => {
                    // Check for a fallback response
                    #[cfg(feature = "fallback-page")]
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
                        #[cfg(any(
                            feature = "compression",
                            feature = "compression-gzip",
                            feature = "compression-brotli",
                            feature = "compression-zstd",
                            feature = "compression-deflate"
                        ))]
                        if self.opts.compression || compression_static {
                            resp.headers_mut().append(
                                hyper::header::VARY,
                                HeaderValue::from_name(hyper::header::ACCEPT_ENCODING),
                            );
                        }

                        // Auto compression based on the `Accept-Encoding` header
                        #[cfg(any(
                            feature = "compression",
                            feature = "compression-gzip",
                            feature = "compression-brotli",
                            feature = "compression-zstd",
                            feature = "compression-deflate"
                        ))]
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
                            custom_headers::append_headers(
                                uri_path,
                                advanced.headers.as_deref(),
                                &mut resp,
                                None,
                            )
                        }

                        return Ok(resp);
                    }

                    // Otherwise return an error response
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
