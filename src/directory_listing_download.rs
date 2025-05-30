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
use std::fmt::Display;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;
use std::task::Poll::{Pending, Ready};
use tokio::fs;
use tokio::io;
use tokio::io::AsyncWriteExt;
use tokio_util::compat::TokioAsyncWriteCompatExt;

use crate::handler::RequestHandlerOpts;
use crate::http_ext::MethodExt;
use crate::Result;

/// query parameter key to download directory as tar.gz
pub const DOWNLOAD_PARAM_KEY: &str = "download";

/// Download format for directory
#[derive(Debug, Serialize, Deserialize, Clone, ValueEnum, Eq, Hash, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DirDownloadFmt {
    /// Gunzip-compressed tarball (.tar.gz)
    Targz,
}

impl Display for DirDownloadFmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

/// Directory download options.
pub struct DirDownloadOpts<'a> {
    /// Request method.
    pub method: &'a Method,
    /// Prevent following symlinks for files and directories.
    pub disable_symlinks: bool,
    /// Ignore hidden files (dotfiles).
    pub ignore_hidden_files: bool,
}

/// Initializes directory listing download
pub fn init(formats: &Vec<DirDownloadFmt>, handler_opts: &mut RequestHandlerOpts) {
    for fmt in formats {
        // Use naive implementation since the list is not expected to be long
        if !handler_opts.dir_listing_download.contains(fmt) {
            tracing::info!("directory listing download: enabled format {}", &fmt);
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
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        let this = self.get_mut();
        let b = BytesMut::from(buf);
        match this.s.poll_ready(cx) {
            Ready(r) => match r {
                Ok(()) => match this.s.try_send_data(b.freeze()) {
                    Ok(_) => Ready(Ok(buf.len())),
                    Err(_) => Pending,
                },
                Err(e) => Ready(Err(io::Error::new(io::ErrorKind::BrokenPipe, e))),
            },
            Pending => Pending,
        }
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        std::task::Poll::Ready(Ok(()))
    }
}

async fn archive(
    path: PathBuf,
    src_path: PathBuf,
    cb: ChannelBuffer,
    follow_symlinks: bool,
    ignore_hidden: bool,
) -> Result {
    let gz = GzipEncoder::with_quality(cb, async_compression::Level::Default);
    let mut a = Builder::new(gz.compat_write());
    a.follow_symlinks(follow_symlinks);

    // NOTE: Since it is not possible to handle error gracefully, we will
    // just stop writing when error occurs. It is also not possible to call
    // sender.abort() as it is protected behind the Builder to ensure
    // finish() is successfully called.

    // adapted from async_tar::Builder::append_dir_all
    let mut stack = vec![(src_path.to_path_buf(), true, false)];
    while let Some((src, is_dir, is_symlink)) = stack.pop() {
        let dest = path.join(src.strip_prefix(&src_path)?);

        // In case of a symlink pointing to a directory, is_dir is false, but src.is_dir() will return true
        if is_dir || (is_symlink && follow_symlinks && src.is_dir()) {
            let mut entries = fs::read_dir(&src).await?;
            while let Some(entry) = entries.next_entry().await? {
                // Check and ignore the current hidden file/directory (dotfile) if feature enabled
                let name = entry.file_name();
                if ignore_hidden && name.as_encoded_bytes().first().is_some_and(|c| *c == b'.') {
                    continue;
                }

                let file_type = entry.file_type().await?;
                stack.push((entry.path(), file_type.is_dir(), file_type.is_symlink()));
            }
            if dest != Path::new("") {
                a.append_dir(&dest, &src).await?;
            }
        } else {
            // use append_path_with_name to handle symlink
            a.append_path_with_name(src, &dest).await?;
        }
    }

    a.finish().await?;
    // this is required to emit gzip CRC trailer
    a.into_inner().await?.into_inner().shutdown().await?;

    Ok(())
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

    resp.headers_mut().typed_insert(ContentType::from(
        // since this satisfies the required format: `*/*`, it should not fail
        Mime::from_str("application/gzip").unwrap(),
    ));
    let hvals = format!(
        "attachment; filename=\"{}\"",
        archive_name.to_string_lossy()
    );
    match HeaderValue::from_str(hvals.as_str()) {
        Ok(hval) => {
            resp.headers_mut()
                .insert(hyper::header::CONTENT_DISPOSITION, hval);
        }
        Err(err) => {
            // not fatal, most browser is able to handle the download since
            // content-type is set
            tracing::error!("can't make content disposition from {}: {:?}", hvals, err);
        }
    }

    // We skip the body for HEAD requests
    if opts.method.is_head() {
        return resp;
    }

    let (tx, body) = Body::channel();
    tokio::task::spawn(archive(
        path.as_ref().into(),
        src_path.as_ref().into(),
        ChannelBuffer { s: tx },
        !opts.disable_symlinks,
        opts.ignore_hidden_files,
    ));
    *resp.body_mut() = body;

    resp
}
