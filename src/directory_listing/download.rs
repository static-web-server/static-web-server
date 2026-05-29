// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Compress content of a directory into a tarball
//!

use async_compression::tokio::write::GzipEncoder;
use async_tar::Builder;
use clap::ValueEnum;
use headers::{ContentType, HeaderMapExt};
use http::{HeaderValue, Method, Response};
use mime_guess::Mime;
use std::fmt::Display;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio_util::compat::TokioAsyncWriteCompatExt;
use tokio_util::io::ReaderStream;

use crate::Result;
use crate::body::Body;
use crate::exts::http::MethodExt;
use crate::handler::RequestHandlerOpts;

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
    pub follow_symlinks: bool,
    /// Ignore hidden files (dotfiles).
    pub include_hidden: bool,
}

/// Initializes directory listing download
pub fn init(formats: &Vec<DirDownloadFmt>, handler_opts: &mut RequestHandlerOpts) {
    for fmt in formats {
        // Use naive implementation since the list is not expected to be long
        if !handler_opts.dir_listing_download.contains(fmt) {
            tracing::info!(format = %fmt, "directory listing download format");
            handler_opts.dir_listing_download.push(fmt.to_owned());
        }
    }
    tracing::info!(
        enabled = !handler_opts.dir_listing_download.is_empty(),
        "directory listing download"
    );
}

/// It implements `AsyncWrite` backed by a Tokio duplex writer used as the `GzipEncoder` write target.
pub struct ChannelBuffer {
    writer: tokio::io::DuplexStream,
}

impl tokio::io::AsyncWrite for ChannelBuffer {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        std::pin::Pin::new(&mut self.get_mut().writer).poll_write(cx, buf)
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        std::pin::Pin::new(&mut self.get_mut().writer).poll_flush(cx)
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        std::pin::Pin::new(&mut self.get_mut().writer).poll_shutdown(cx)
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
    let mut resp = Response::new(crate::body::empty());

    resp.headers_mut().typed_insert(ContentType::from(
        Mime::from_str("application/gzip").unwrap_or(mime_guess::mime::APPLICATION_OCTET_STREAM),
    ));
    // SECURITY: Build a safe `Content-Disposition` value that combines an
    // ASCII-safe quoted-string `filename=...` (for legacy user agents) and
    // an RFC 5987 `filename*=UTF-8''<percent-encoded>` (for modern UAs).
    //
    // The previous implementation interpolated the directory name into the
    // quoted-string without escaping `"` or `\`, producing a malformed
    // header for any directory name containing those characters. While
    // `HeaderValue::from_str` blocks CRLF, malformed Content-Disposition
    // can still confuse downstream proxies and browsers.
    let archive_name_str = archive_name.to_string_lossy();
    let ascii_safe = sanitize_filename_for_quoted_string(&archive_name_str);
    let percent_encoded = rfc5987_encode_filename(&archive_name_str);
    let hvals =
        format!("attachment; filename=\"{ascii_safe}\"; filename*=UTF-8''{percent_encoded}");
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

    let (read_half, write_half) = tokio::io::duplex(64 * 1024);
    let body = crate::body::stream(ReaderStream::new(read_half));
    tokio::task::spawn(archive(
        path.as_ref().into(),
        src_path.as_ref().into(),
        ChannelBuffer { writer: write_half },
        opts.follow_symlinks,
        !opts.include_hidden,
    ));
    *resp.body_mut() = body;

    resp
}

/// Sanitize a filename for use inside an HTTP `Content-Disposition`
/// `filename="..."` quoted-string. Strips characters that would break the
/// quoted-string framing (`"`, `\`) or HTTP header parsing (`\r`, `\n`, NUL,
/// and other ASCII control bytes), and replaces any non-ASCII byte with
/// `_`. The lossy ASCII filename is paired with an RFC 5987 `filename*=`
/// variant carrying the full UTF-8 name (see `rfc5987_encode_filename`).
#[doc(hidden)]
pub fn sanitize_filename_for_quoted_string(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for ch in name.chars() {
        match ch {
            '"' | '\\' => out.push('_'),
            c if (c as u32) < 0x20 || c == '\x7f' => out.push('_'),
            c if c.is_ascii() => out.push(c),
            _ => out.push('_'),
        }
    }
    if out.is_empty() {
        out.push_str("download");
    }
    out
}

/// Percent-encode a filename per RFC 5987 (the `attr-char` production
/// from RFC 8187). Used as the `filename*=UTF-8''<value>` parameter so
/// non-ASCII filenames survive transit to modern user agents.
#[doc(hidden)]
pub fn rfc5987_encode_filename(name: &str) -> String {
    // RFC 8187 `attr-char` allows: ALPHA / DIGIT and `! # $ & + - . ^ _ ` | ~`
    // Everything else (including `"`, `\`, space, control bytes, and any
    // non-ASCII byte) is percent-encoded as `%HH`.
    fn is_attr_char(b: u8) -> bool {
        b.is_ascii_alphanumeric()
            || matches!(
                b,
                b'!' | b'#' | b'$' | b'&' | b'+' | b'-' | b'.' | b'^' | b'_' | b'`' | b'|' | b'~'
            )
    }
    let mut out = String::with_capacity(name.len());
    for &b in name.as_bytes() {
        if is_attr_char(b) {
            out.push(b as char);
        } else {
            use std::fmt::Write;
            let _ = write!(out, "%{b:02X}");
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{rfc5987_encode_filename, sanitize_filename_for_quoted_string};

    /// SECURITY: A directory name containing `"` or `\` must NOT break out
    /// of the `Content-Disposition` quoted-string framing.
    #[test]
    fn sanitize_strips_quote_and_backslash() {
        let out = sanitize_filename_for_quoted_string("evil\".tar.gz");
        assert!(!out.contains('"'));
        let out2 = sanitize_filename_for_quoted_string("a\\b.tar.gz");
        assert!(!out2.contains('\\'));
    }

    /// SECURITY: Control bytes (CR/LF/NUL) must not survive into a header
    /// value \u2014 they could be reflected if a downstream proxy mishandles
    /// `Content-Disposition`.
    #[test]
    fn sanitize_strips_control_bytes() {
        let out = sanitize_filename_for_quoted_string("a\r\nb\tc\x00d");
        for ch in out.chars() {
            assert!(
                ch as u32 >= 0x20 && ch != '\x7f',
                "control byte leaked: {:?}",
                ch
            );
        }
    }

    /// Non-ASCII characters are dropped from the quoted-string variant
    /// (browsers fall back to the `filename*=UTF-8''...` parameter for
    /// these).
    #[test]
    fn sanitize_replaces_non_ascii() {
        let out = sanitize_filename_for_quoted_string("rep\u{00f6}rt.tar.gz");
        assert!(out.is_ascii());
        assert!(out.starts_with("rep_rt") || out.starts_with("rep__rt"));
    }

    #[test]
    fn sanitize_never_empty() {
        assert_eq!(sanitize_filename_for_quoted_string(""), "download");
    }

    /// RFC 5987 / RFC 8187 attr-char alphabet must round-trip unchanged.
    #[test]
    fn rfc5987_preserves_attr_char_alphabet() {
        let input = "abcXYZ0189!#$&+-.^_`|~";
        assert_eq!(rfc5987_encode_filename(input), input);
    }

    /// Everything outside attr-char must be percent-encoded \u2014 in
    /// particular, `"`, `\`, space, CR, LF, and any non-ASCII byte.
    #[test]
    fn rfc5987_encodes_unsafe_bytes() {
        assert_eq!(rfc5987_encode_filename("a b"), "a%20b");
        assert_eq!(rfc5987_encode_filename("a\"b"), "a%22b");
        assert_eq!(rfc5987_encode_filename("a\\b"), "a%5Cb");
        assert_eq!(rfc5987_encode_filename("a\r\nb"), "a%0D%0Ab");
        // UTF-8 `\u{00f6}` = 0xC3 0xB6
        assert_eq!(rfc5987_encode_filename("\u{00f6}"), "%C3%B6");
    }

    // Property-based regression tests for Content-Disposition helpers.
    //
    // These encode the security invariants that protect the
    // `Content-Disposition` header against quoted-string framing breaks,
    // header smuggling via control bytes, and ambiguous user-agent
    // parsing of non-ASCII filenames.
    use proptest::prelude::*;

    fn is_attr_char(b: u8) -> bool {
        b.is_ascii_alphanumeric()
            || matches!(
                b,
                b'!' | b'#' | b'$' | b'&' | b'+' | b'-' | b'.' | b'^' | b'_' | b'`' | b'|' | b'~'
            )
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 256, ..ProptestConfig::default()
        })]

        /// `sanitize_filename_for_quoted_string` MUST always yield a
        /// non-empty ASCII string with no quoted-string-breaking or
        /// control bytes, for any UTF-8 input.
        #[test]
        fn prop_sanitize_filename_invariants(name in "\\PC{0,256}") {
            let out = sanitize_filename_for_quoted_string(&name);
            prop_assert!(!out.is_empty(), "output must never be empty");
            prop_assert!(out.is_ascii(), "output must be pure ASCII");
            for ch in out.chars() {
                prop_assert!(
                    ch != '"' && ch != '\\',
                    "quoted-string break byte leaked: {:?}",
                    ch
                );
                let code = ch as u32;
                prop_assert!(
                    code >= 0x20 && code != 0x7f,
                    "control byte leaked: {:?}",
                    ch
                );
            }
        }

        /// Sanitization is idempotent: a single pass already produces a
        /// fixed point of the transform.
        #[test]
        fn prop_sanitize_filename_is_idempotent(name in "\\PC{0,256}") {
            let once = sanitize_filename_for_quoted_string(&name);
            let twice = sanitize_filename_for_quoted_string(&once);
            prop_assert_eq!(once, twice);
        }

        /// `rfc5987_encode_filename` MUST emit only attr-char bytes or
        /// well-formed `%HH` percent-escapes, for any UTF-8 input.
        #[test]
        fn prop_rfc5987_encode_only_safe_alphabet(name in "\\PC{0,256}") {
            let out = rfc5987_encode_filename(&name);
            let bytes = out.as_bytes();
            let mut i = 0;
            while i < bytes.len() {
                let b = bytes[i];
                if b == b'%' {
                    // Must be followed by exactly two uppercase hex digits.
                    prop_assert!(i + 2 < bytes.len(), "truncated percent-escape at {i}");
                    let h1 = bytes[i + 1];
                    let h2 = bytes[i + 2];
                    let is_hex_upper = |c: u8| c.is_ascii_digit() || (b'A'..=b'F').contains(&c);
                    prop_assert!(
                        is_hex_upper(h1) && is_hex_upper(h2),
                        "non-uppercase-hex percent-escape: %{}{}",
                        h1 as char,
                        h2 as char
                    );
                    i += 3;
                } else {
                    prop_assert!(
                        is_attr_char(b),
                        "non-attr-char byte leaked: 0x{:02X}",
                        b
                    );
                    i += 1;
                }
            }
        }

        /// Inputs already drawn from the attr-char alphabet must
        /// round-trip unchanged.
        #[test]
        fn prop_rfc5987_attr_char_inputs_roundtrip(s in "[A-Za-z0-9!#\\$&+\\-\\.\\^_`|~]{0,128}") {
            prop_assert_eq!(rfc5987_encode_filename(&s).clone(), s);
        }
    }
}
