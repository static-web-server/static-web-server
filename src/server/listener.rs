// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Module to create TCP and Unix Domain Socket listeners.
//!
//! Provides functions for binding to TCP addresses (with optional inherited
//! file descriptors from systemd/supervisors) and for Unix Domain Sockets
//! (with support for permission setting and forceful stale-socket cleanup).
//!

use listenfd::ListenFd;
use std::net::{IpAddr, SocketAddr, TcpListener};

use crate::settings::cli::General;
use crate::{Context, Error, Result};

/// Create a TCP listener bound to the address specified in `general`.
///
/// If the `general.fd` field is `Some`, the listener is obtained from an
/// inherited file descriptor (e.g. passed by systemd or a similar supervisor).
/// Otherwise a new [`TcpListener`] is bound to the host/port combination from
/// the configuration.
pub(crate) fn create_tcp_listener(general: &General) -> Result<(TcpListener, String), Error> {
    let (listener, bound_addr) = match general.fd {
        Some(fd) => {
            let listener = ListenFd::from_env()
                .take_tcp_listener(fd)?
                .with_context(|| "failed to convert inherited 'fd' into a 'tcp' listener")?;
            tracing::info!(
                fd,
                "converted inherited file descriptor to a 'tcp' listener"
            );
            (listener, format!("@FD({fd})"))
        }
        None => {
            let ip = general
                .host
                .parse::<IpAddr>()
                .with_context(|| format!("failed to parse {} address", general.host))?;
            let addr = SocketAddr::from((ip, general.port));
            let listener = TcpListener::bind(addr)
                .with_context(|| format!("failed to bind to {addr} address"))?;
            tracing::info!(addr = %addr, "server bound to tcp socket");
            (listener, addr.to_string())
        }
    };
    Ok((listener, bound_addr))
}

/// Create a Unix Domain Socket listener bound to `path`.
///
/// - If `force` is `true`, a pre-existing socket file at `path` is removed
///   before binding. This makes restarts after an unclean shutdown predictable.
///   We deliberately only unlink **socket** files (not regular files) to avoid
///   accidentally clobbering unrelated data on disk.
/// - If `mode` is provided, the socket file's permission bits are set after
///   binding. We chmod *after* `bind(2)` rather than relying on the process
///   umask so that operators can pin permissions independently of how
///   static-web-server was launched.
#[cfg(unix)]
pub(crate) fn create_unix_listener(
    path: &std::path::Path,
    mode: Option<u32>,
    force: bool,
) -> Result<(tokio::net::UnixListener, std::path::PathBuf, String), Error> {
    use std::os::unix::fs::{FileTypeExt, PermissionsExt};

    // Reject paths that are too long for the platform's `sockaddr_un.sun_path`
    // up-front so we produce a clear error instead of a cryptic `EINVAL` from
    // `bind(2)`. The conservative cross-Unix limit is ~104 bytes (macOS); Linux
    // allows ~108. We check against the smaller bound to remain portable.
    const SUN_PATH_MAX: usize = 104;
    if path.as_os_str().len() >= SUN_PATH_MAX {
        return Err(anyhow!(
            "unix socket path '{}' is too long ({} bytes, limit is {})",
            path.display(),
            path.as_os_str().len(),
            SUN_PATH_MAX - 1
        ));
    }

    // Optionally clean up a stale socket file from a previous run. We only
    // remove the path if it is a socket — never a regular file, directory or
    // symlink — to keep the operation safe in case of misconfiguration.
    if force {
        match std::fs::symlink_metadata(path) {
            Ok(meta) if meta.file_type().is_socket() => {
                std::fs::remove_file(path).with_context(|| {
                    format!(
                        "failed to remove stale unix socket file '{}'",
                        path.display()
                    )
                })?;
                tracing::info!(path = %path.display(), "removed stale unix socket file");
            }
            Ok(_) => {
                return Err(anyhow!(
                    "refusing to overwrite non-socket file at '{}' (use a different path or remove it manually)",
                    path.display()
                ));
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => {
                return Err(Error::new(err).context(format!(
                    "failed to inspect unix socket path '{}'",
                    path.display()
                )));
            }
        }
    }

    let listener = tokio::net::UnixListener::bind(path).with_context(|| {
        format!(
            "failed to bind unix socket at '{}' (tip: pass --unix-socket-force to remove a stale socket)",
            path.display()
        )
    })?;

    if let Some(bits) = mode {
        let perms = std::fs::Permissions::from_mode(bits);
        std::fs::set_permissions(path, perms).with_context(|| {
            format!(
                "failed to set permissions {:o} on unix socket '{}'",
                bits,
                path.display()
            )
        })?;
        tracing::info!(path = %path.display(), mode = format!("{bits:o}"), "set unix socket permissions");
    }

    let addr_str = format!("unix:{}", path.display());
    tracing::info!(path = %path.display(), "server bound to unix socket");
    Ok((listener, path.to_path_buf(), addr_str))
}
