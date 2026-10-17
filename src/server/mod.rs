// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Server module to construct a multi-threaded HTTP or HTTP/2 web server.

use hyper::server::conn::http1::Builder as Http1Builder;
use hyper_util::rt::{TokioIo, TokioTimer};
use std::net::TcpListener;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch::Receiver;

use self::idle_timeout::IdleTimeout;
use crate::handler::RequestHandler;
use crate::service::RouterService;
use crate::settings::cli::General;
use crate::{Context, Result, Settings};

mod browser;
mod http1;
#[cfg(feature = "tls")]
mod http1_tls;
#[cfg(feature = "http2")]
mod http2;
mod idle_timeout;
mod listener;
mod opts;
#[cfg(feature = "tls")]
mod redirect;
#[cfg(unix)]
mod uds;

/// Settings shared by the TCP server implementations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ServerRunConfig<'a> {
    /// Listener address displayed in startup logs.
    pub addr_str: &'a str,
    /// Number of Tokio worker threads.
    pub threads: usize,
    /// Whether to open the listener URL in the default browser.
    pub open: bool,
    /// Optional URL path opened in the browser.
    pub open_path: Option<&'a str>,
    /// Connection settings.
    pub conn: ConnConfig,
}

/// Connection settings shared by all accept loops.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(test, derive(Default))]
pub(crate) struct ConnConfig {
    /// Time without read or write progress after which a connection is closed, or `None` for no limit.
    /// HTTP/1 request headers must also arrive within this time.
    pub idle_timeout: Option<Duration>,
}

impl ConnConfig {
    /// Create the connection settings from the general settings.
    fn new(general: &General) -> Self {
        Self {
            idle_timeout: (general.idle_timeout > 0)
                .then(|| Duration::from_secs(general.idle_timeout.into())),
        }
    }

    /// Create an HTTP/1 connection builder with these settings.
    ///
    /// The idle timeout alone doesn't limit a request header that arrives a byte at a time,
    /// so Hyper's header read timeout is set to the same value. It requires a timer.
    fn http1_builder(&self) -> Http1Builder {
        let mut builder = Http1Builder::new();
        builder
            .timer(TokioTimer::new())
            .header_read_timeout(self.idle_timeout);
        builder
    }

    /// Wrap an accepted connection's IO for Hyper, applying the idle timeout.
    fn io<T>(&self, io: T) -> TokioIo<IdleTimeout<T>> {
        TokioIo::new(IdleTimeout::new(io, self.idle_timeout))
    }
}

/// TLS configuration shared by the HTTP/1+TLS and HTTP/2+TLS server modes.
#[cfg(feature = "tls")]
pub(crate) struct TlsConfig {
    /// Path to the TLS certificate file.
    pub tls_cert: std::path::PathBuf,
    /// Path to the TLS private key file.
    pub tls_key: std::path::PathBuf,
    /// Enable HTTP to HTTPS redirect server.
    pub https_redirect: bool,
    /// Target hostname used in HTTPS redirect responses.
    pub https_redirect_host: String,
    /// Port the HTTP redirect server binds on.
    pub https_redirect_from_port: u16,
    /// Comma-separated list of hosts allowed to be redirected.
    pub https_redirect_from_hosts: String,
    /// Server host address (needed to bind the redirect listener).
    pub host: String,
    /// HTTPS port (used in redirect target URLs).
    pub port: u16,
    /// Resolved 404 error page (used by the redirect server).
    pub page404: std::path::PathBuf,
    /// Resolved 50x error page (used by the redirect server).
    pub page50x: std::path::PathBuf,
}

/// Shutdown context passed to each server sub-module so they can respond to
/// both OS signals and optional programmatic cancellation.
pub(crate) struct ShutdownCtx {
    /// Grace period in seconds before the server is forcefully terminated.
    pub grace_period: u8,
    /// Optional programmatic cancel receiver.
    pub cancel_recv: Option<Receiver<()>>,
    #[cfg(windows)]
    /// Whether the server is running as a Windows service.
    pub windows_service: bool,
    #[cfg(windows)]
    /// Ctrl+C watch receiver used when not running as a Windows service.
    pub ctrl_c_recv: Receiver<()>,
    #[cfg(windows)]
    /// Background task that listens for Ctrl+C and signals the watch channel.
    pub ctrlc_task: tokio::task::JoinHandle<crate::Result<()>>,
}

/// Time to wait before accepting again after an accept error that isn't about a single connection.
const ACCEPT_ERROR_DELAY: Duration = Duration::from_secs(1);

/// Wait before the next `accept` if `err` isn't about a single connection.
///
/// Errors such as running out of file descriptors (`EMFILE`) leave the pending connection in the listen backlog,
/// so accepting again right away fails the same way and spins the accept loop.
/// Like Hyper 0.14's `AddrIncoming`, wait for [`ACCEPT_ERROR_DELAY`] after such errors.
async fn accept_error_backoff(err: &std::io::Error) {
    if let Some(delay) = accept_error_delay(err) {
        tokio::time::sleep(delay).await;
    }
}

/// Return how long to wait before the next `accept` after `err`, if at all.
fn accept_error_delay(err: &std::io::Error) -> Option<Duration> {
    use std::io::ErrorKind;
    match err.kind() {
        ErrorKind::ConnectionRefused
        | ErrorKind::ConnectionAborted
        | ErrorKind::ConnectionReset => None,
        _ => Some(ACCEPT_ERROR_DELAY),
    }
}

/// A multi-threaded HTTP or HTTP/2 web server.
pub struct Server {
    opts: Settings,
    worker_threads: usize,
    max_blocking_threads: usize,
    /// Optional pre-bound TCP listener injected by the caller.
    /// When set, the server uses this listener instead of creating one
    /// from the `--host` / `--port` settings.
    pre_bound_listener: Option<(TcpListener, String)>,
}

impl Server {
    /// Create a new multi-threaded server instance.
    pub fn new(opts: Settings) -> Result<Server> {
        let cpus = std::thread::available_parallelism()
            .with_context(|| {
                "unable to get current platform cpus or lack of permissions to query available parallelism"
            })?
            .get();
        let worker_threads = match opts.general.threads_multiplier {
            0 | 1 => cpus,
            n => cpus * n,
        };
        let max_blocking_threads = opts.general.max_blocking_threads;
        Ok(Server {
            opts,
            worker_threads,
            max_blocking_threads,
            pre_bound_listener: None,
        })
    }

    /// Attach a pre-bound TCP listener. The server will use this listener
    /// instead of creating one from the `--host` / `--port` settings.
    /// The listener must already be in a listening state (created via
    /// [`std::net::TcpListener::bind`]).
    pub fn with_pre_bound_listener(mut self, listener: std::net::TcpListener) -> Self {
        let addr = listener
            .local_addr()
            .map(|a| a.to_string())
            .unwrap_or_else(|_| "pre-bound".into());
        self.pre_bound_listener = Some((listener, addr));
        self
    }

    /// Run the multi-threaded `Server` as standalone.
    ///
    /// It accepts an optional [`cancel`] parameter to shut down the server
    /// gracefully on demand as a complement to the termination signals handling.
    ///
    /// [`cancel`]: <https://docs.rs/tokio/latest/tokio/sync/watch/struct.Receiver.html>
    pub fn run_standalone(self, cancel: Option<Receiver<()>>) -> Result {
        self.run_server_on_rt(cancel, || {}, true)
    }

    /// Run the multi-threaded `Server` which will be used by a Windows service.
    ///
    /// It accepts an optional [`cancel`] parameter to shut down the server
    /// gracefully on demand and a `cancel_fn` that will be executed right after
    /// the server shuts down.
    ///
    /// [`cancel`]: <https://docs.rs/tokio/latest/tokio/sync/watch/struct.Receiver.html>
    #[cfg(windows)]
    pub fn run_as_service<F>(self, cancel: Option<Receiver<()>>, cancel_fn: F) -> Result
    where
        F: FnOnce(),
    {
        self.run_server_on_rt(cancel, cancel_fn, true)
    }

    /// Build and run the multi-threaded `Server` on the Tokio runtime.
    ///
    /// Setting `exit_on_error` to `true` will exit the entire process if
    /// the server fails to start (previous behaviour).
    pub fn run_server_on_rt<F>(
        self,
        cancel_recv: Option<Receiver<()>>,
        cancel_fn: F,
        exit_on_error: bool,
    ) -> Result
    where
        F: FnOnce(),
    {
        tracing::debug!(
            %self.worker_threads,
            "initializing tokio runtime with multi-threaded scheduler"
        );

        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(self.worker_threads)
            .max_blocking_threads(self.max_blocking_threads)
            .thread_name("static-web-server")
            .enable_all()
            .build()?;

        let res = rt.block_on(async {
            tracing::trace!("tokio runtime initialized");
            self.start_server(cancel_recv, cancel_fn).await
        });

        if let Err(err) = &res {
            tracing::error!("server failed to start up: {:?}", err);
            if exit_on_error {
                std::process::exit(1)
            }
        }
        res
    }

    /// Start the Hyper server (HTTP/1 or HTTP/2 + TLS) and block until shutdown.
    ///
    /// This method orchestrates listener creation, options initialization, and
    /// delegates to the appropriate server sub-module.
    async fn start_server<F>(self, cancel_recv: Option<Receiver<()>>, cancel_fn: F) -> Result
    where
        F: FnOnce(),
    {
        tracing::trace!("starting web server");
        tracing::info!(
            name = env!("CARGO_PKG_NAME"),
            version = env!("CARGO_PKG_VERSION"),
            "starting Static Web Server"
        );

        let general = self.opts.general;
        let advanced = self.opts.advanced;
        let pre_bound = self.pre_bound_listener;

        tracing::info!(log_level = %general.log_level, "log level");
        if general.config_file.is_file() {
            tracing::info!(path = %general.config_file.display(), "config file used");
        } else {
            tracing::debug!(
                "config file path not found or not a regular file: {}",
                general.config_file.display()
            );
        }

        // Choose listener kind: Unix Domain Socket (when --unix-socket is set,
        // Unix only) or a TCP socket otherwise. Clap already enforces mutual
        // exclusion with host/port/fd/tls, but we still resolve the listener
        // here so the dispatch below can branch on it.
        #[cfg(unix)]
        let unix_listener_info = if let Some(path) = general.unix_socket.as_ref() {
            use crate::server::listener::create_unix_listener;

            Some(create_unix_listener(
                path,
                general.unix_socket_mode,
                general.unix_socket_force,
            )?)
        } else {
            None
        };

        // The TCP listener is only bound when no UDS path was provided. Binding
        // both would either waste a port or fail with a host parse error on
        // platforms where `host` is required.
        // When a pre-bound listener was injected (e.g. by tests), use it
        // instead of creating a new one — this avoids TOCTOU port races.
        #[cfg(unix)]
        let tcp_listener_info = if unix_listener_info.is_none() {
            Some(match pre_bound {
                Some(pre) => pre,
                None => crate::server::listener::create_tcp_listener(&general)?,
            })
        } else {
            None
        };
        #[cfg(not(unix))]
        let tcp_listener_info = Some(match pre_bound {
            Some(pre) => pre,
            None => crate::server::listener::create_tcp_listener(&general)?,
        });

        tracing::info!(
            worker_threads = self.worker_threads,
            "runtime worker threads"
        );
        tracing::info!(
            max_blocking_threads = general.max_blocking_threads,
            "runtime max blocking threads"
        );
        tracing::info!(
            grace_period_seconds = general.grace_period,
            "grace period before graceful shutdown"
        );
        tracing::info!(
            idle_timeout_seconds = general.idle_timeout,
            "connection idle timeout"
        );
        let conn = ConnConfig::new(&general);

        // Initialize request handler options from configuration
        let opts_result = opts::init(&general, advanced)?;
        let router_service = RouterService::new(RequestHandler {
            opts: Arc::from(opts_result.handler_opts),
        });

        // Windows: spawn a background task that bridges Ctrl+C into a watch channel
        #[cfg(windows)]
        let (sender, ctrl_c_recv) = tokio::sync::watch::channel(());
        #[cfg(windows)]
        let windows_service = general.windows_service;
        #[cfg(windows)]
        let ctrlc_task = tokio::spawn(async move {
            if !windows_service {
                tracing::info!("installing graceful shutdown ctrl+c signal handler");
                if let Err(err) = tokio::signal::ctrl_c().await {
                    return Err(
                        crate::Error::new(err).context("failed to install ctrl+c signal handler")
                    );
                }
                tracing::info!("graceful shutdown ctrl+c signal received");
                let _ = sender.send(());
            }
            Ok::<_, crate::Error>(())
        });

        let ctx = ShutdownCtx {
            grace_period: general.grace_period,
            cancel_recv,
            #[cfg(windows)]
            windows_service,
            #[cfg(windows)]
            ctrl_c_recv,
            #[cfg(windows)]
            ctrlc_task,
        };

        // Unix Domain Socket dispatch (Unix only, no TLS). Clap already forbids
        // combining `--unix-socket` with TLS so we never reach the TLS branch
        // below when a UDS listener is present.
        #[cfg(unix)]
        if let Some((unix_listener, socket_path, addr_str)) = unix_listener_info {
            let run_cfg = ServerRunConfig {
                addr_str: &addr_str,
                threads: self.worker_threads,
                open: false,
                open_path: None,
                conn,
            };
            return uds::run(
                unix_listener,
                socket_path,
                router_service,
                run_cfg,
                ctx,
                cancel_fn,
            )
            .await;
        }

        // Safe to unwrap: when no UDS listener was created, `tcp_listener_info`
        // is `Some` by construction above.
        let (tcp_listener, addr_str) = tcp_listener_info.unwrap();

        let run_cfg = ServerRunConfig {
            addr_str: &addr_str,
            threads: self.worker_threads,
            open: general.open,
            open_path: general.open_path.as_deref(),
            conn,
        };

        // Dispatch to a TLS-enabled server (HTTP/1+TLS or HTTP/2+TLS) when --tls is set
        #[cfg(feature = "tls")]
        if general.tls {
            let tls_cert = general
                .tls_cert
                .ok_or_else(|| anyhow!("TLS cert file path is required when --tls is enabled"))?;
            let tls_key = general
                .tls_key
                .ok_or_else(|| anyhow!("TLS key file path is required when --tls is enabled"))?;

            let tls_cfg = TlsConfig {
                tls_cert,
                tls_key,
                https_redirect: general.https_redirect,
                https_redirect_host: general.https_redirect_host,
                https_redirect_from_port: general.https_redirect_from_port,
                https_redirect_from_hosts: general.https_redirect_from_hosts,
                host: general.host,
                port: general.port,
                page404: opts_result.page404,
                page50x: opts_result.page50x,
            };

            // If HTTP/2 is also enabled, use the HTTP/2+TLS accept loop
            #[cfg(feature = "http2")]
            if general.http2 {
                return http2::run(
                    tcp_listener,
                    router_service,
                    run_cfg,
                    tls_cfg,
                    ctx,
                    cancel_fn,
                )
                .await;
            }

            // Otherwise serve HTTP/1 over TLS
            return http1_tls::run(
                tcp_listener,
                router_service,
                run_cfg,
                tls_cfg,
                ctx,
                cancel_fn,
            )
            .await;
        }

        // Plain HTTP/1 (no TLS by default)
        http1::run(tcp_listener, router_service, run_cfg, ctx, cancel_fn).await
    }
}

#[cfg(test)]
mod tests {
    use super::{ACCEPT_ERROR_DELAY, ConnConfig, accept_error_delay};
    use std::io::{Error, ErrorKind};
    use std::time::Duration;

    /// Create the connection settings from the given command line arguments.
    fn conn_config(args: &[&str]) -> ConnConfig {
        let settings = crate::Settings::get_unparsed(false, args).expect("settings parse");
        ConnConfig::new(&settings.general)
    }

    #[test]
    fn conn_config_idle_timeout_defaults_to_30_seconds() {
        let conn = conn_config(&["static-web-server"]);
        assert_eq!(conn.idle_timeout, Some(Duration::from_secs(30)));
    }

    #[test]
    fn conn_config_idle_timeout_zero_disables_it() {
        // `None` also disables Hyper's header read timeout instead of keeping its default.
        let conn = conn_config(&["static-web-server", "--idle-timeout", "0"]);
        assert_eq!(conn.idle_timeout, None);
    }

    #[test]
    fn accept_error_delay_skips_per_connection_errors() {
        for kind in [
            ErrorKind::ConnectionRefused,
            ErrorKind::ConnectionAborted,
            ErrorKind::ConnectionReset,
        ] {
            assert_eq!(accept_error_delay(&Error::from(kind)), None);
        }
    }

    #[test]
    fn accept_error_delay_waits_on_other_errors() {
        // EMFILE on Unix: the process ran out of file descriptors.
        let emfile = Error::from_raw_os_error(24);
        assert_eq!(accept_error_delay(&emfile), Some(ACCEPT_ERROR_DELAY));
    }
}
