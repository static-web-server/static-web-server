// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Configurable access logging with format string support.
//!
//! Supports a `%{token}` syntax for user-defined log formats.
//! When no format is configured, the line is logged with structured fields instead, named after
//! the OpenTelemetry semantic conventions (see [`fields`]).
//!
//! The line is written when hyper drops the response body: once it is sent, or when the client
//! goes away. So `%{bytes}` and `%{duration}` cover the whole transfer.

use bytes::Bytes;
use http_body::{Frame, SizeHint};
use http_body_util::BodyExt;
use hyper::header::{HOST, HeaderValue, REFERER, USER_AGENT};
use hyper::{Method, Request, Response, StatusCode, Uri, Version};
use std::io;
use std::net::{IpAddr, SocketAddr};
use std::pin::Pin;
use std::task::{Context, Poll, ready};
use std::time::{Duration, Instant};

use crate::{body::Body, handler_opts::RequestHandlerOpts, health, logger::addr};

mod fields;
pub(super) mod format;
mod render;

use fields::ResponseFields;
use format::AccessLogFormat;

pub(crate) use fields::url_path_query;

/// The target of the line of a custom format and of the level check in [`enabled`], kept from the
/// former module path for log filters.
const TARGET: &str = "static_web_server::access_logs";

/// The target of the default line, kept from `log_addr` for log filters.
const DEFAULT_TARGET: &str = addr::TARGET;

/// Whether the line of `req` passes the log level filter, so it is worth building.
pub(crate) fn enabled<T>(opts: &RequestHandlerOpts, req: &Request<T>) -> bool {
    // Health check requests are logged in debug mode only
    if opts.health && health::is_health_endpoint(req.uri()) {
        tracing::enabled!(target: TARGET, tracing::Level::DEBUG)
    } else {
        tracing::enabled!(target: TARGET, tracing::Level::INFO)
    }
}

/// The request values of an access log line, copied before the pipeline runs: rewrites replace
/// the URI and `Host`, and the line is written after the handler returns.
///
/// IPv4-mapped IPv6 addresses (`::ffff:a.b.c.d`) are kept as IPv4, so a dual-stack listener logs
/// the same address as an IPv4 one.
#[derive(Debug)]
pub(crate) struct AccessLog {
    /// `None` logs the default line with structured fields.
    format: Option<AccessLogFormat>,
    method: Method,
    uri: Uri,
    host: Option<HeaderValue>,
    version: Version,
    user_agent: Option<HeaderValue>,
    referer: Option<HeaderValue>,
    remote_ip: Option<IpAddr>,
    x_real_ip: Option<IpAddr>,
    forwarded_for: Option<IpAddr>,
    health: bool,
    start: Instant,
}

impl AccessLog {
    /// Copies the request values the access log uses.
    pub(crate) fn new<T>(
        format: Option<&AccessLogFormat>,
        opts: &RequestHandlerOpts,
        req: &Request<T>,
        remote_addr: Option<SocketAddr>,
    ) -> Self {
        let headers = req.headers();
        // The default line logs only the addresses its options enable
        let custom = format.is_some();
        Self {
            format: format.cloned(),
            method: req.method().clone(),
            uri: req.uri().clone(),
            host: headers.get(HOST).cloned(),
            version: req.version(),
            user_agent: headers.get(USER_AGENT).cloned(),
            referer: headers.get(REFERER).cloned(),
            remote_ip: remote_addr
                .filter(|_| custom || opts.log_remote_address)
                .map(|addr| addr.ip().to_canonical()),
            x_real_ip: (custom || opts.log_x_real_ip)
                .then(|| addr::x_real_ip(opts, req, remote_addr))
                .flatten()
                .map(|ip| ip.to_canonical()),
            forwarded_for: (custom || opts.log_forwarded_for)
                .then(|| addr::forwarded_for(opts, req, remote_addr))
                .flatten()
                .map(|ip| ip.to_canonical()),
            health: opts.health && health::is_health_endpoint(req.uri()),
            start: Instant::now(),
        }
    }

    /// Wraps the response body, so the line is logged when hyper drops it.
    pub(crate) fn wrap(self, resp: Response<Body>) -> Response<Body> {
        let status = resp.status();
        // Like CLF `%b`, these log `-` since no body is sent
        let sends_body = self.method != Method::HEAD
            && status != StatusCode::NO_CONTENT
            && status != StatusCode::NOT_MODIFIED;
        resp.map(|inner| {
            LoggedBody {
                inner,
                log: self,
                status: status.as_u16(),
                bytes: 0,
                sends_body,
            }
            .boxed()
        })
    }

    /// Writes the line of a response with `status`, sending `body_size` bytes, `None` when the
    /// response has no body.
    fn write(&self, status: u16, body_size: Option<u64>) {
        let duration = duration_secs(self.start.elapsed());
        let Some(format) = &self.format else {
            self.write_default(&ResponseFields::new(self, status, body_size, duration));
            return;
        };
        let ctx = render::LogContext {
            method: self.method.as_str(),
            uri: &self.uri,
            host: self.host.as_ref(),
            version: render::http_version_str(self.version),
            status,
            bytes: body_size.unwrap_or(0),
            duration_secs: duration,
            remote_ip: self.remote_ip,
            user_agent: self.user_agent.as_ref(),
            referer: self.referer.as_ref(),
            x_real_ip: self.x_real_ip,
            forwarded_for: self.forwarded_for,
        };
        let line = render::render(format, &ctx);

        // Log health check requests in debug mode only
        if self.health {
            tracing::debug!(target: TARGET, "{line}");
        } else {
            tracing::info!(target: TARGET, "{line}");
        }
    }

    fn write_default(&self, fields: &ResponseFields<'_>) {
        let client_address = fields.client_address.as_ref().map(tracing::field::display);
        let peer_address = fields.peer_address.as_ref().map(tracing::field::display);

        macro_rules! response {
            ($level:ident) => {
                tracing::$level!(
                    target: DEFAULT_TARGET,
                    {
                        http.request.method = fields.method,
                        url.path = fields.path,
                        url.query = fields.query,
                        http.response.status_code = fields.status,
                        http.response.body.size = fields.body_size,
                        duration = fields.duration,
                        user_agent.original = fields.user_agent.as_deref(),
                        client.address = client_address,
                        network.peer.address = peer_address,
                    },
                    "response"
                )
            };
        }

        // Log health check requests in debug mode only
        if self.health {
            response!(debug);
        } else {
            response!(info);
        }
    }
}

/// The duration in seconds, rounded to microseconds, so `%{duration}` and the `duration` field of
/// the default line log the same value.
fn duration_secs(elapsed: Duration) -> f64 {
    let micros = elapsed.as_nanos().saturating_add(500) / 1000;
    micros as f64 / 1e6
}

/// A response body that counts the data bytes it yields and writes the access log line on drop.
struct LoggedBody {
    inner: Body,
    log: AccessLog,
    status: u16,
    bytes: u64,
    sends_body: bool,
}

impl http_body::Body for LoggedBody {
    type Data = Bytes;
    type Error = io::Error;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, io::Error>>> {
        let this = self.get_mut();
        let frame = ready!(Pin::new(&mut this.inner).poll_frame(cx));
        if let Some(data) = frame.as_ref().and_then(|f| f.as_ref().ok()?.data_ref()) {
            this.bytes += data.len() as u64;
        }
        Poll::Ready(frame)
    }

    fn is_end_stream(&self) -> bool {
        self.inner.is_end_stream()
    }

    fn size_hint(&self) -> SizeHint {
        self.inner.size_hint()
    }
}

impl Drop for LoggedBody {
    fn drop(&mut self) {
        let body_size = self.sends_body.then_some(self.bytes);
        self.log.write(self.status, body_size);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing::Level;
    use tracing_subscriber::{filter::Targets, prelude::*};

    use crate::logger::register_test_subscribers;

    #[test]
    fn duration_secs_rounds_to_microseconds() {
        let cases = [
            (Duration::ZERO, 0.0, "0.000000"),
            (Duration::from_nanos(412_499), 0.000412, "0.000412"),
            (Duration::from_nanos(412_500), 0.000413, "0.000413"),
            (Duration::from_nanos(1_234_567_890), 1.234568, "1.234568"),
        ];
        for (elapsed, secs, text) in cases {
            let duration = duration_secs(elapsed);
            assert_eq!(duration, secs, "{elapsed:?}");
            assert_eq!(format!("{duration:.6}"), text, "the `%{{duration}}` value");
        }
    }

    #[test]
    fn enabled_follows_a_filter_on_the_log_target() {
        register_test_subscribers();
        let filter = Targets::new().with_target("static_web_server::access_logs", Level::INFO);
        let layer = tracing_subscriber::fmt::layer()
            .with_writer(std::io::sink)
            .with_filter(filter);
        tracing::subscriber::with_default(tracing_subscriber::registry().with(layer), || {
            assert!(enabled(&RequestHandlerOpts::default(), &Request::new(())));
        });
    }
}
