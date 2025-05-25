// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Compress content of a directory into a tarball
//!

use async_compression::tokio::write::GzipEncoder;
use async_tar::Builder;
use bytes::BytesMut;
use clap::ValueEnum;
use headers::{ContentType, HeaderMapExt};
use http::{HeaderValue, Method, Response};
use hyper::{body::Sender, Body};
use mime_guess::Mime;
use std::str::FromStr;
use std::{ffi::OsString, path::Path};
use tokio::io::{self, AsyncWriteExt};
use tokio_util::compat::TokioAsyncWriteCompatExt;

use crate::handler::RequestHandlerOpts;
use crate::http_ext::MethodExt;

/// query parameter key to download directory as tar.gz
pub const DOWNLOAD_PARAM_KEY: &str = "download";

/// Download format for directory
#[derive(Debug, Serialize, Deserialize, Clone, ValueEnum, Eq, Hash, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DirDownloadFmt {
    /// Disable directory download
    None,
    /// Gunzip-compressed tarball (.tar.gz)
    Targz,
}

/// Directory download options.
pub struct DirDownloadOpts<'a> {
    /// Request method.
    pub method: &'a Method,
    /// Prevent following symlinks for files and directories.
    pub disable_symlinks: bool,
}

/// Initializes directory listing download
pub fn init(formats: &Vec<DirDownloadFmt>, handler_opts: &mut RequestHandlerOpts) {
    let mut is_none = false;
    for fmt in formats {
        if *fmt == DirDownloadFmt::None {
            is_none = true;
            continue;
        }
        if is_none {
            panic!("Setting directory-listing-download to {:?} when None is also specified is not allowed", *fmt);
        }
        // Use naive implementation since the list is not expected to be long
        if !handler_opts.dir_listing_download.contains(fmt) {
            handler_opts.dir_listing_download.push(fmt.to_owned());
        }
    }
}

/// impl AsyncWrite for hyper::Body::Sender
pub struct ChannelBuffer {
    s: Sender,
}

impl tokio::io::AsyncWrite for ChannelBuffer {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::result::Result<usize, std::io::Error>> {
        // TODO: what kind of error may be encountered?
        let this = self.get_mut();
        let b = BytesMut::from(buf);
        match this.s.poll_ready(cx) {
            std::task::Poll::Ready(r) => match r {
                Ok(()) => match this.s.try_send_data(b.freeze()) {
                    Ok(_) => std::task::Poll::Ready(Ok(buf.len())),
                    Err(_) => std::task::Poll::Pending,
                },
                Err(e) => std::task::Poll::Ready(Err(io::Error::new(io::ErrorKind::BrokenPipe, e))),
            },
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::result::Result<(), std::io::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::result::Result<(), std::io::Error>> {
        std::task::Poll::Ready(Ok(()))
    }
}

fn archive_dir<P, Q>(path: P, src_path: Q, follow_symlinks: bool) -> Body
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    let (tx, body) = Body::channel();
    let cb = ChannelBuffer { s: tx };

    let p: OsString = path.as_ref().into();
    let sp: OsString = src_path.as_ref().into();
    tokio::task::spawn(async move {
        let gz = GzipEncoder::with_quality(cb, async_compression::Level::Default);
        let mut a = Builder::new(gz.compat_write());
        a.follow_symlinks(follow_symlinks);
        let mut res = a.append_dir_all(p, sp).await;
        if res.is_ok() {
            res = a.finish().await;
        }
        let mut gz_inner = a.into_inner().await.unwrap().into_inner();
        if res.is_ok() {
            // this is required to emit gzip CRC trailer
            res = gz_inner.shutdown().await;
        }
        if res.is_err() {
            let cb_inner = gz_inner.into_inner();
            cb_inner.s.abort();
        }
    });

    body
}

/// Reply with archived directory content in compressed tarball format.
/// The content from `src_path` on server filesystem will be stored to `path`
/// within the tarball.
/// An async task will be spawned to asynchronously write compressed data to the
/// response body.
pub fn archive_reply<P, Q>(path: P, src_path: Q, opts: DirDownloadOpts<'_>) -> Response<Body>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    let archive_name = path.as_ref().with_extension("tar.gz");
    let mut resp = Response::new(Body::empty());
    let hvals = format!(
        "attachment; filename=\"{}\"",
        archive_name.to_string_lossy()
    );
    resp.headers_mut().typed_insert(ContentType::from(
        Mime::from_str("application/gzip").unwrap(),
    ));
    match HeaderValue::from_str(hvals.as_str()) {
        Ok(hval) => {
            resp.headers_mut()
                .insert(hyper::header::CONTENT_DISPOSITION, hval);
        }
        Err(err) => {
            tracing::error!("cant make content disposition from {}: {:?}", hvals, err);
        }
    }

    // We skip the body for HEAD requests
    if opts.method.is_head() {
        return resp;
    }

    *resp.body_mut() = archive_dir(path, src_path, !opts.disable_symlinks);

    resp
}
