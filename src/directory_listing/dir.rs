// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

use chrono::{DateTime, Local};
use clap::ValueEnum;
use headers::{ContentLength, ContentType, HeaderMapExt};
use http::Method;
use hyper::Response;
use mime_guess::mime;
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, percent_encode};
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::body::Body;
use crate::directory_listing::autoindex::{html_auto_index, json_auto_index};
use crate::directory_listing::file::{FileEntry, FileType};
use crate::{Context, Result};

#[cfg(feature = "directory-listing-download")]
use crate::directory_listing::download::DirDownloadFmt;

/// Non-alphanumeric characters to be percent-encoded
/// excluding the "unreserved characters" because allowed in a URI.
/// See 2.3.  Unreserved Characters - <https://www.ietf.org/rfc/rfc3986.txt>
const PERCENT_ENCODE_SET: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'_')
    .remove(b'-')
    .remove(b'.')
    .remove(b'~');

/// Directory listing output format for file entries.
#[derive(Debug, Serialize, Deserialize, Clone, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum DirListFmt {
    /// HTML format to display (default).
    Html,
    /// JSON format to display.
    Json,
}

/// Directory listing options.
pub struct DirListOpts<'a> {
    /// Request method.
    pub root_path: &'a Path,
    /// Request method.
    pub method: &'a Method,
    /// Current Request path.
    pub current_path: &'a str,
    /// URI Request query
    pub uri_query: Option<&'a str>,
    /// Request file path.
    pub filepath: &'a Path,
    /// Directory listing order.
    pub dir_listing_order: u8,
    /// Directory listing format.
    pub dir_listing_format: &'a DirListFmt,
    #[cfg(feature = "directory-listing-download")]
    /// Directory listing download.
    pub dir_listing_download: &'a [DirDownloadFmt],
    /// Ignore hidden files (dotfiles).
    pub include_hidden: bool,
    /// Prevent following symlinks for files and directories.
    pub follow_symlinks: bool,
}

/// Defines read directory entries.
pub(crate) struct DirEntryOpts<'a> {
    pub(crate) root_path: &'a Path,
    pub(crate) dir_reader: std::fs::ReadDir,
    pub(crate) base_path: &'a str,
    pub(crate) uri_query: Option<&'a str>,
    pub(crate) is_head: bool,
    pub(crate) order_code: u8,
    pub(crate) content_format: &'a DirListFmt,
    pub(crate) include_hidden: bool,
    pub(crate) follow_symlinks: bool,
    #[cfg(feature = "directory-listing-download")]
    pub(crate) download: &'a [DirDownloadFmt],
}

/// It reads a list of directory entries and create an index page content.
/// Otherwise it returns a status error.
pub(crate) fn read_dir_entries(mut opt: DirEntryOpts<'_>) -> Result<Response<Body>> {
    let mut dirs_count: usize = 0;
    let mut files_count: usize = 0;
    // The root directory is canonicalized once at startup (see
    // `server/opts.rs`). To avoid an extra `canonicalize()` syscall per
    // request, we resolve the absolute form lazily — only when a symlink
    // entry is actually encountered (the uncommon case).
    let mut root_path_abs: Option<std::path::PathBuf> = None;
    let (entries_hint, _) = opt.dir_reader.size_hint();
    let mut file_entries: Vec<FileEntry> = Vec::with_capacity(entries_hint);

    for dir_entry in opt.dir_reader {
        let dir_entry = dir_entry.with_context(|| "unable to read directory entry")?;
        let meta = match dir_entry.metadata() {
            Ok(m) => m,
            Err(err) => {
                tracing::error!(
                    "unable to resolve metadata for file or directory entry (skipped): {:?}",
                    err
                );
                continue;
            }
        };

        let name = dir_entry.file_name();

        // Check and ignore the current hidden file/directory (dotfile) if feature enabled
        if !opt.include_hidden && name.as_encoded_bytes().first().is_some_and(|c| *c == b'.') {
            continue;
        }

        let (r#type, size) = if meta.is_dir() {
            dirs_count += 1;
            (FileType::Directory, None)
        } else if meta.is_file() {
            files_count += 1;
            (FileType::File, Some(meta.len()))
        } else if opt.follow_symlinks && meta.file_type().is_symlink() {
            // NOTE: we resolve the symlink path below to just know if is a directory or not.
            // However, we are still showing the symlink name but not the resolved name.

            let symlink_path = dir_entry.path();
            let symlink_path = match symlink_path.canonicalize() {
                Ok(v) => v,
                Err(err) => {
                    tracing::error!(
                        "unable resolve symlink path for `{}` (skipped): {:?}",
                        symlink_path.display(),
                        err,
                    );
                    continue;
                }
            };
            if !symlink_path.starts_with(root_path_abs.get_or_insert_with(|| {
                opt.root_path
                    .canonicalize()
                    .unwrap_or_else(|_| opt.root_path.to_path_buf())
            })) {
                tracing::warn!(
                    "unable to follow symlink {}, access denied",
                    symlink_path.display()
                );
                continue;
            }
            let symlink_meta = match std::fs::symlink_metadata(&symlink_path) {
                Ok(v) => v,
                Err(err) => {
                    tracing::error!(
                        "unable to resolve metadata for `{}` symlink (skipped): {:?}",
                        symlink_path.display(),
                        err,
                    );
                    continue;
                }
            };
            if symlink_meta.is_dir() {
                dirs_count += 1;
                (FileType::Directory, None)
            } else {
                files_count += 1;
                (FileType::File, Some(symlink_meta.len()))
            }
        } else {
            continue;
        };

        let name_encoded = percent_encode(name.as_encoded_bytes(), PERCENT_ENCODE_SET).to_string();

        // NOTE: Use relative paths by default independently of
        // the "redirect trailing slash" feature.
        // However, when "redirect trailing slash" is disabled
        // and a request path doesn't contain a trailing slash then
        // entries should contain the "parent/entry-name" as a link format.
        // Otherwise, we just use the "entry-name" as a link (default behavior).
        // Note that in both cases, we add a trailing slash if the entry is a directory.
        let mut uri = if !opt.base_path.ends_with('/') && !opt.base_path.is_empty() {
            let parent = opt
                .base_path
                .rsplit_once('/')
                .map(|(_, parent)| parent)
                .unwrap_or(opt.base_path);
            format!("{parent}/{name_encoded}")
        } else {
            name_encoded
        };

        if r#type == FileType::Directory {
            uri.push('/');
        }

        let mtime = meta.modified().ok().map(DateTime::<Local>::from);

        let entry = FileEntry {
            name,
            mtime,
            size,
            r#type,
            uri,
        };
        file_entries.push(entry);
    }

    // Check the query request uri for a sorting type. E.g https://blah/?sort=5
    if let Some(q) = opt.uri_query {
        // NOTE: we just pick up the first `sort` pair.
        // Avoid calling `.count()` (which consumes the iterator) and then
        // re-parsing the query string a second time.
        if let Some(code) = form_urlencoded::parse(q.as_bytes())
            .find(|(key, _)| key == "sort")
            .and_then(|(_, value)| value.trim().parse::<u8>().ok())
        {
            opt.order_code = code;
        }
    }

    let mut resp = Response::new(crate::body::empty());

    // Handle directory listing content format
    let content = match opt.content_format {
        DirListFmt::Json => {
            // JSON
            resp.headers_mut()
                .typed_insert(ContentType::from(mime::APPLICATION_JSON));

            json_auto_index(&mut file_entries, opt.order_code)?
        }
        // HTML (default)
        _ => {
            resp.headers_mut()
                .typed_insert(ContentType::from(mime::TEXT_HTML_UTF_8));

            html_auto_index(
                opt.base_path,
                dirs_count,
                files_count,
                &mut file_entries,
                opt.order_code,
                #[cfg(feature = "directory-listing-download")]
                opt.download,
            )
        }
    };

    resp.headers_mut()
        .typed_insert(ContentLength(content.len() as u64));

    // We skip the body for HEAD requests
    if opt.is_head {
        return Ok(resp);
    }

    *resp.body_mut() = crate::body::full(content);

    Ok(resp)
}
