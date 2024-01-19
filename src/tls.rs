// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! The module handles requests over TLS via [Rustls](tokio_rustls::rustls).
//!

// Most of the file is borrowed from https://github.com/seanmonstar/warp/blob/master/src/tls.rs

use futures_util::ready;
use hyper::server::accept::Accept;
use hyper::server::conn::{AddrIncoming, AddrStream};
use std::fs::File;
use std::future::Future;
use std::io::{self, BufReader, Cursor, Read};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio_rustls::rustls::{pki_types::PrivateKeyDer, Error as TlsError, ServerConfig};

use crate::transport::Transport;

/// Represents errors that can occur building the TlsConfig
#[derive(Debug)]
pub enum TlsConfigError {
    /// Error type for I/O operations
    Io(io::Error),
    /// An Error parsing the Certificate
    CertParseError,
    /// Identity PEM is invalid
    InvalidIdentityPem,
    /// An error from an empty key
    EmptyKey,
    /// Unknown private key format
    UnknownPrivateKeyFormat,
    /// An error from an invalid key
    InvalidKey(TlsError),
}

impl std::fmt::Display for TlsConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TlsConfigError::Io(err) => err.fmt(f),
            TlsConfigError::CertParseError => write!(f, "certificate parse error"),
            TlsConfigError::InvalidIdentityPem => write!(f, "identity PEM is invalid"),
            TlsConfigError::UnknownPrivateKeyFormat => write!(f, "unknown private key format"),
            TlsConfigError::EmptyKey => write!(f, "key contains no private key"),
            TlsConfigError::InvalidKey(err) => write!(f, "key contains an invalid key, {err}"),
        }
    }
}

impl std::error::Error for TlsConfigError {}

/// Builder to set the configuration for the Tls server.
pub struct TlsConfigBuilder {
    cert: Box<dyn Read + Send + Sync>,
    key: Box<dyn Read + Send + Sync>,
}

impl std::fmt::Debug for TlsConfigBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        f.debug_struct("TlsConfigBuilder").finish()
    }
}

impl TlsConfigBuilder {
    /// Create a new TlsConfigBuilder
    pub fn new() -> TlsConfigBuilder {
        TlsConfigBuilder {
            key: Box::new(io::empty()),
            cert: Box::new(io::empty()),
        }
    }

    /// sets the Tls key via File Path, returns `TlsConfigError::IoError` if the file cannot be open
    pub fn key_path(mut self, path: impl AsRef<Path>) -> Self {
        self.key = Box::new(LazyFile {
            path: path.as_ref().into(),
            file: None,
        });
        self
    }

    /// sets the Tls key via bytes slice
    pub fn key(mut self, key: &[u8]) -> Self {
        self.key = Box::new(Cursor::new(Vec::from(key)));
        self
    }

    /// Specify the file path for the TLS certificate to use.
    pub fn cert_path(mut self, path: impl AsRef<Path>) -> Self {
        self.cert = Box::new(LazyFile {
            path: path.as_ref().into(),
            file: None,
        });
        self
    }

    /// sets the Tls certificate via bytes slice
    pub fn cert(mut self, cert: &[u8]) -> Self {
        self.cert = Box::new(Cursor::new(Vec::from(cert)));
        self
    }

    /// Builds TLS configuration.
    pub fn build(mut self) -> Result<ServerConfig, TlsConfigError> {
        let mut cert_rdr = BufReader::new(self.cert);
        let cert = rustls_pemfile::certs(&mut cert_rdr)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_e| TlsConfigError::CertParseError)?;

        // convert it to Vec<u8> to allow reading it again if key is RSA
        let mut key_buf = Vec::new();
        self.key
            .read_to_end(&mut key_buf)
            .map_err(TlsConfigError::Io)?;

        if key_buf.is_empty() {
            return Err(TlsConfigError::EmptyKey);
        }

        let mut key: Option<PrivateKeyDer<'_>> = None;
        let mut reader = Cursor::new(key_buf);
        for item in std::iter::from_fn(|| rustls_pemfile::read_one(&mut reader).transpose()) {
            match item.map_err(|_e| TlsConfigError::InvalidIdentityPem)? {
                // rsa pkcs1 key
                rustls_pemfile::Item::Pkcs1Key(k) => key = Some(k.into()),
                // pkcs8 key
                rustls_pemfile::Item::Pkcs8Key(k) => key = Some(k.into()),
                // sec1 ec key
                rustls_pemfile::Item::Sec1Key(k) => key = Some(k.into()),
                // unknown format
                _ => return Err(TlsConfigError::UnknownPrivateKeyFormat),
            }
        }

        let key = match key {
            Some(k) => k,
            _ => return Err(TlsConfigError::EmptyKey),
        };

        let mut config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(cert, key)
            .map_err(TlsConfigError::InvalidKey)?;
        config.alpn_protocols = vec!["h2".into(), "http/1.1".into()];
        Ok(config)
    }
}

impl Default for TlsConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

struct LazyFile {
    path: PathBuf,
    file: Option<File>,
}

impl LazyFile {
    fn lazy_read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.file.is_none() {
            self.file = Some(File::open(&self.path)?);
        }

        self.file.as_mut().unwrap().read(buf)
    }
}

impl Read for LazyFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.lazy_read(buf).map_err(|err| {
            let kind = err.kind();
            io::Error::new(
                kind,
                format!("error reading file ({:?}): {}", self.path.display(), err),
            )
        })
    }
}

impl Transport for TlsStream {
    fn remote_addr(&self) -> Option<SocketAddr> {
        Some(self.remote_addr)
    }
}

enum State {
    Handshaking(tokio_rustls::Accept<AddrStream>),
    Streaming(tokio_rustls::server::TlsStream<AddrStream>),
}

/// TlsStream implements AsyncRead/AsyncWrite handshaking tokio_rustls::Accept first.
///
/// tokio_rustls::server::TlsStream doesn't expose constructor methods,
/// so we have to TlsAcceptor::accept and handshake to have access to it.
pub struct TlsStream {
    state: State,
    remote_addr: SocketAddr,
}

impl TlsStream {
    fn new(stream: AddrStream, config: Arc<ServerConfig>) -> TlsStream {
        let remote_addr = stream.remote_addr();
        let accept = tokio_rustls::TlsAcceptor::from(config).accept(stream);
        TlsStream {
            state: State::Handshaking(accept),
            remote_addr,
        }
    }
}

impl AsyncRead for TlsStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let pin = self.get_mut();
        match pin.state {
            State::Handshaking(ref mut accept) => match ready!(Pin::new(accept).poll(cx)) {
                Ok(mut stream) => {
                    let result = Pin::new(&mut stream).poll_read(cx, buf);
                    pin.state = State::Streaming(stream);
                    result
                }
                Err(err) => Poll::Ready(Err(err)),
            },
            State::Streaming(ref mut stream) => Pin::new(stream).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for TlsStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let pin = self.get_mut();
        match pin.state {
            State::Handshaking(ref mut accept) => match ready!(Pin::new(accept).poll(cx)) {
                Ok(mut stream) => {
                    let result = Pin::new(&mut stream).poll_write(cx, buf);
                    pin.state = State::Streaming(stream);
                    result
                }
                Err(err) => Poll::Ready(Err(err)),
            },
            State::Streaming(ref mut stream) => Pin::new(stream).poll_write(cx, buf),
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.state {
            State::Handshaking(_) => Poll::Ready(Ok(())),
            State::Streaming(ref mut stream) => Pin::new(stream).poll_flush(cx),
        }
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.state {
            State::Handshaking(_) => Poll::Ready(Ok(())),
            State::Streaming(ref mut stream) => Pin::new(stream).poll_shutdown(cx),
        }
    }
}

/// Type to intercept Tls incoming connections.
pub struct TlsAcceptor {
    config: Arc<ServerConfig>,
    incoming: AddrIncoming,
}

impl TlsAcceptor {
    /// Creates a new Tls interceptor.
    pub fn new(config: ServerConfig, incoming: AddrIncoming) -> TlsAcceptor {
        TlsAcceptor {
            config: Arc::new(config),
            incoming,
        }
    }
}

impl Accept for TlsAcceptor {
    type Conn = TlsStream;
    type Error = io::Error;

    fn poll_accept(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Conn, Self::Error>>> {
        let pin = self.get_mut();
        match ready!(Pin::new(&mut pin.incoming).poll_accept(cx)) {
            Some(Ok(sock)) => Poll::Ready(Some(Ok(TlsStream::new(sock, pin.config.clone())))),
            Some(Err(e)) => Poll::Ready(Some(Err(e))),
            None => Poll::Ready(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_cert_key_rsa_pkcs1() {
        TlsConfigBuilder::new()
            .cert_path("tests/tls/local.dev_cert.rsa_pkcs1.pem")
            .key_path("tests/tls/local.dev_key.rsa_pkcs1.pem")
            .build()
            .unwrap();
    }

    #[test]
    fn bytes_cert_key_rsa_pkcs1() {
        let cert = include_str!("../tests/tls/local.dev_cert.rsa_pkcs1.pem");
        let key = include_str!("../tests/tls/local.dev_key.rsa_pkcs1.pem");

        TlsConfigBuilder::new()
            .key(key.as_bytes())
            .cert(cert.as_bytes())
            .build()
            .unwrap();
    }

    #[test]
    fn file_cert_key_pkcs8() {
        TlsConfigBuilder::new()
            .cert_path("tests/tls/local.dev_cert.pkcs8.pem")
            .key_path("tests/tls/local.dev_key.pkcs8.pem")
            .build()
            .unwrap();
    }

    #[test]
    fn bytes_cert_key_pkcs8() {
        let cert = include_str!("../tests/tls/local.dev_cert.pkcs8.pem");
        let key = include_str!("../tests/tls/local.dev_key.pkcs8.pem");

        TlsConfigBuilder::new()
            .key(key.as_bytes())
            .cert(cert.as_bytes())
            .build()
            .unwrap();
    }

    #[test]
    fn file_cert_key_sec1_ec() {
        TlsConfigBuilder::new()
            .cert_path("tests/tls/local.dev_cert.sec1_ec.pem")
            .key_path("tests/tls/local.dev_key.sec1_ec.pem")
            .build()
            .unwrap();
    }

    #[test]
    fn bytes_cert_key_sec1_ec() {
        let cert = include_str!("../tests/tls/local.dev_cert.sec1_ec.pem");
        let key = include_str!("../tests/tls/local.dev_key.sec1_ec.pem");

        TlsConfigBuilder::new()
            .key(key.as_bytes())
            .cert(cert.as_bytes())
            .build()
            .unwrap();
    }
}
