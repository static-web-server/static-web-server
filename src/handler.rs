// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Request handler module intended to manage incoming HTTP requests.
//!

use hyper::{Request, Response, StatusCode};
use std::{future::Future, net::SocketAddr, path::PathBuf, sync::Arc};

use crate::body::Body;

#[cfg(any(
    feature = "compression",
    feature = "compression-gzip",
    feature = "compression-brotli",
    feature = "compression-zstd",
    feature = "compression-deflate"
))]
use crate::compression;

use crate::compression_static;

#[cfg(feature = "basic-auth")]
use crate::basic_auth;

#[cfg(feature = "fallback-page")]
use crate::fallback_page;

#[cfg(feature = "metrics")]
use crate::metrics;

use crate::{
    Error, Result, access_logs, control_headers, cors, custom_headers, error_page,
    exts::http::MethodExt,
    health, maintenance_mode,
    negotiation::image as image_negotiation,
    redirects, rewrites, security_headers,
    static_files::{self, HandleOpts},
    text_charset, virtual_hosts,
};

pub use crate::handler_opts::RequestHandlerOpts;

/// It defines the main request handler used by the Hyper service request.
pub struct RequestHandler {
    /// Request handler options.
    pub opts: Arc<RequestHandlerOpts>,
}

impl RequestHandler {
    /// Main entry point for incoming requests.
    pub fn handle<'a, B>(
        &'a self,
        req: &'a mut Request<B>,
        remote_addr: Option<SocketAddr>,
    ) -> impl Future<Output = Result<Response<Body>, Error>> + Send + 'a
    where
        B: Send + 'a,
    {
        let mut base_path = &self.opts.root_dir;
        #[cfg(feature = "directory-listing")]
        let dir_listing = self.opts.dir_listing;
        #[cfg(feature = "directory-listing")]
        let dir_listing_order = self.opts.dir_listing_order;
        #[cfg(feature = "directory-listing")]
        let dir_listing_format = &self.opts.dir_listing_format;
        #[cfg(feature = "directory-listing-download")]
        let dir_listing_download = &self.opts.dir_listing_download;
        let redirect_trailing_slash = self.opts.redirect_trailing_slash;
        let compression_static = self.opts.compression_static;
        let etag = self.opts.etag;
        let include_hidden = self.opts.include_hidden;
        let follow_symlinks = self.opts.follow_symlinks;
        let index_files: Vec<&str> = self.opts.index_files.iter().map(|s| s.as_str()).collect();
        #[cfg(feature = "mem-cache")]
        let memory_cache = self.opts.memory_cache.as_ref();

        // Rewrites replace the request URI and `Host`, so keep the ones the client sent for the
        // access log
        let access_log = access_logs::enabled(&self.opts, req).then(|| {
            let format = self.opts.access_log_format.as_ref();
            access_logs::AccessLog::new(format, &self.opts, req, remote_addr)
        });

        async move {
            #[cfg(feature = "metrics")]
            let req_start = std::time::Instant::now();
            // Health probes are excluded, like they are from `info` logs
            #[cfg(feature = "metrics")]
            let metrics_enabled = self.opts.metrics_enabled
                && !(self.opts.health && health::is_health_endpoint(req.uri()));

            #[cfg(feature = "metrics")]
            if metrics_enabled {
                metrics::inc_requests_inflight();
            }

            // Early responses serve no file; custom headers with `scope = "all"` still reach them
            let result: Result<Early, Error> = async {
                // Reject if the HTTP request method is not allowed
                if !req.method().is_allowed() {
                    return error_page::error_response(
                        req.uri(),
                        req.method(),
                        &StatusCode::METHOD_NOT_ALLOWED,
                        &self.opts.page404,
                        &self.opts.page50x,
                    )
                    .map(early);
                }

                // Health endpoints check (`/livez` and `/readyz`)
                if let Some(result) = health::pre_process(&self.opts, req) {
                    return result.map(early);
                }

                // CORS
                let cors = self.opts.cors.as_ref();
                if let Some(result) =
                    cors::pre_process(cors, &self.opts.page404, &self.opts.page50x, req)
                {
                    return result.map(early);
                }

                // `Basic` HTTP Authorization Schema
                #[cfg(feature = "basic-auth")]
                if let Some(response) = basic_auth::pre_process(&self.opts, req) {
                    return response.map(early);
                }

                // Metrics endpoint check
                #[cfg(feature = "metrics")]
                if let Some(result) = metrics::pre_process(&self.opts, req) {
                    return result.map(early);
                }

                // Maintenance Mode
                if let Some(response) = maintenance_mode::pre_process(&self.opts, req) {
                    return response.map(early);
                }

                // Redirects
                if let Some(result) = redirects::pre_process(&self.opts, req) {
                    return result.map(early);
                }

                // Rewrites
                if let Some(result) = rewrites::pre_process(&self.opts, req) {
                    return result.map(early);
                }

                // Advanced options
                if let Some(advanced) = &self.opts.advanced_opts {
                    // If the "Host" header matches any virtual_host, change the root directory
                    if let Some(root) =
                        virtual_hosts::get_real_root(req, advanced.virtual_hosts.as_deref())
                    {
                        base_path = root;
                    }
                }

                let index_files = index_files.as_ref();

                // Check for markdown content negotiation (only if enabled)
                let uri_path_md = if self.opts.accept_markdown {
                    crate::markdown::pre_process(
                        req,
                        base_path,
                        req.uri().path(),
                        self.opts.include_hidden,
                    )
                } else {
                    None
                };

                // Check for image format negotiation (only if enabled)
                let uri_path_image = if self.opts.image_negotiation && uri_path_md.is_none() {
                    image_negotiation::pre_process(
                        req,
                        base_path,
                        req.uri().path(),
                        self.opts.include_hidden,
                    )
                } else {
                    None
                };
                let uri_path = uri_path_md
                    .as_deref()
                    .or(uri_path_image.as_deref())
                    .unwrap_or(req.uri().path());

                // Static files
                let (resp, file_path, is_dir_listing) = match static_files::handle(&HandleOpts {
                    method: req.method(),
                    headers: req.headers(),
                    #[cfg(feature = "mem-cache")]
                    memory_cache,
                    base_path,
                    uri_path,
                    uri_query: req.uri().query(),
                    #[cfg(feature = "directory-listing")]
                    dir_listing,
                    #[cfg(feature = "directory-listing")]
                    dir_listing_order,
                    #[cfg(feature = "directory-listing")]
                    dir_listing_format,
                    #[cfg(feature = "directory-listing-download")]
                    dir_listing_download,
                    redirect_trailing_slash,
                    compression_static,
                    etag,
                    include_hidden,
                    index_files,
                    follow_symlinks,
                })
                .await
                {
                    Ok(result) => (result.resp, result.file_path, result.is_dir_listing),
                    Err(status) => {
                        #[cfg(feature = "fallback-page")]
                        let will_serve_fallback = fallback_page::can_serve(&self.opts, req, status);
                        #[cfg(not(feature = "fallback-page"))]
                        let will_serve_fallback = false;

                        let resp = if will_serve_fallback {
                            error_page::error_response_without_logging(
                                req.uri(),
                                req.method(),
                                &status,
                                &self.opts.page404,
                                &self.opts.page50x,
                            )?
                        } else {
                            error_page::error_response(
                                req.uri(),
                                req.method(),
                                &status,
                                &self.opts.page404,
                                &self.opts.page50x,
                            )?
                        };

                        (resp, None, false)
                    }
                };

                // Check for a fallback response
                #[cfg(feature = "fallback-page")]
                let resp = fallback_page::post_process(&self.opts, req, resp)?;

                // Append CORS headers if they are present
                let resp = cors::post_process(self.opts.cors.as_ref(), req, resp)?;

                // Add `Vary: Accept` and set Content-Type for markdown files
                let resp = crate::markdown::post_process(
                    uri_path_md.is_some(),
                    is_dir_listing,
                    file_path.as_deref(),
                    &self.opts,
                    resp,
                )?;

                // Add `Vary: Accept` for images with format variants
                let resp = image_negotiation::post_process(&self.opts, req, resp)?;

                // Declare a default charset for `text/*` responses
                let resp = text_charset::post_process(&self.opts, resp)?;

                // Add a `Vary` header if static compression is used
                let resp = compression_static::post_process(&self.opts, req, resp)?;

                // Auto compression based on the `Accept-Encoding` header
                #[cfg(any(
                    feature = "compression",
                    feature = "compression-gzip",
                    feature = "compression-brotli",
                    feature = "compression-zstd",
                    feature = "compression-deflate"
                ))]
                let resp = compression::post_process(&self.opts, req, resp)?;

                // Append `Cache-Control` headers for web assets
                let resp = control_headers::post_process(&self.opts, req, resp)?;

                // Append security headers
                let resp = security_headers::post_process(&self.opts, req, resp)?;

                Ok((resp, file_path, false))
            }
            .await;

            // Add/update custom headers
            let result = result.and_then(|(resp, file_path, early)| {
                custom_headers::post_process(&self.opts, req, resp, file_path.as_ref(), early)
            });

            // The access log line is written once the response body is sent
            let result = match access_log {
                Some(access_log) => result.map(|resp| access_log.wrap(resp)),
                None => result,
            };

            #[cfg(feature = "metrics")]
            if metrics_enabled {
                metrics::dec_requests_inflight();
                if let Ok(ref resp) = result {
                    let bytes = resp
                        .headers()
                        .get(http::header::CONTENT_LENGTH)
                        .and_then(|v| v.to_str().ok())
                        .and_then(|v| v.parse::<u64>().ok())
                        .unwrap_or(0);
                    metrics::record_request(
                        &self.opts,
                        req,
                        resp.status(),
                        bytes,
                        req_start.elapsed().as_secs_f64(),
                    );
                }
            }

            result
        }
    }
}

/// A response together with the served file path and whether it was returned early.
type Early = (Response<Body>, Option<PathBuf>, bool);

/// Marks a response that was returned before the static file handler ran.
fn early(resp: Response<Body>) -> Early {
    (resp, None, true)
}
