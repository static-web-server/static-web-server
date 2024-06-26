// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! It provides directory listing and auto-index support.
//!

use chrono::{DateTime, Local, Utc};
use clap::ValueEnum;
use headers::{ContentLength, ContentType, HeaderMapExt};
use hyper::{Body, Method, Response, StatusCode};
use mime_guess::mime;
use percent_encoding::{percent_decode_str, percent_encode, AsciiSet, NON_ALPHANUMERIC};
use serde::{Serialize, Serializer};
use std::ffi::{OsStr, OsString};
use std::io;
use std::path::Path;

use crate::{handler::RequestHandlerOpts, http_ext::MethodExt, Context, Result};

/// Non-alphanumeric characters to be percent-encoded
/// excluding the "unreserved characters" because allowed in a URI.
/// See 2.3.  Unreserved Characters - https://www.ietf.org/rfc/rfc3986.txt
const PERCENT_ENCODE_SET: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'_')
    .remove(b'-')
    .remove(b'.')
    .remove(b'~');

#[derive(Debug, Serialize, Deserialize, Clone, ValueEnum)]
#[serde(rename_all = "lowercase")]
/// Directory listing output format for file entries.
pub enum DirListFmt {
    /// HTML format to display (default).
    Html,
    /// JSON format to display.
    Json,
}

/// Directory listing options.
pub struct DirListOpts<'a> {
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
    /// Ignore hidden files (dotfiles).
    pub ignore_hidden_files: bool,
    /// Prevent following symlinks for files and directories.
    pub disable_symlinks: bool,
}

/// Initializes directory listings.
pub fn init(enabled: bool, order: u8, format: DirListFmt, handler_opts: &mut RequestHandlerOpts) {
    handler_opts.dir_listing = enabled;
    server_info!("directory listing: enabled={enabled}");

    handler_opts.dir_listing_order = order;
    server_info!("directory listing order code: {order}");

    handler_opts.dir_listing_format = format;
    server_info!(
        "directory listing format: {:?}",
        handler_opts.dir_listing_format
    );
}

/// Provides directory listing support for the current request.
/// Note that this function highly depends on `static_files::composed_file_metadata()` function
/// which must be called first. See `static_files::handle()` for more details.
pub fn auto_index(opts: DirListOpts<'_>) -> Result<Response<Body>, StatusCode> {
    // Note: it's safe to call `parent()` here since `filepath`
    // value always refer to a path with file ending and under
    // a root directory boundary.
    // See `composed_file_metadata()` function which sanitizes the requested
    // path before to be delegated here.
    let filepath = opts.filepath;
    let parent = filepath.parent().unwrap_or(filepath);

    match std::fs::read_dir(parent) {
        Ok(dir_reader) => {
            let dir_opts = DirEntryOpts {
                dir_reader,
                base_path: opts.current_path,
                uri_query: opts.uri_query,
                is_head: opts.method.is_head(),
                order_code: opts.dir_listing_order,
                content_format: opts.dir_listing_format,
                ignore_hidden_files: opts.ignore_hidden_files,
                disable_symlinks: opts.disable_symlinks,
            };
            match read_dir_entries(dir_opts) {
                Ok(resp) => Ok(resp),
                Err(err) => {
                    tracing::error!("error after try to read directory entries: {:?}", err);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        Err(err) => {
            let status = match err.kind() {
                io::ErrorKind::NotFound => {
                    tracing::debug!(
                        "entry file not found (path: {}): {:?}",
                        filepath.display(),
                        err
                    );
                    StatusCode::NOT_FOUND
                }
                io::ErrorKind::PermissionDenied => {
                    tracing::error!(
                        "entry file permission denied (path: {}): {:?}",
                        filepath.display(),
                        err
                    );
                    StatusCode::FORBIDDEN
                }
                _ => {
                    tracing::error!(
                        "unable to read parent directory (parent={}): {:?}",
                        parent.display(),
                        err
                    );
                    StatusCode::INTERNAL_SERVER_ERROR
                }
            };
            Err(status)
        }
    }
}

const DATETIME_FORMAT_UTC: &str = "%FT%TZ";
const DATETIME_FORMAT_LOCAL: &str = "%F %T";

#[derive(Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum FileType {
    Directory,
    File,
}

/// Defines a file entry and its properties.
#[derive(Serialize)]
struct FileEntry {
    #[serde(serialize_with = "serialize_name")]
    name: OsString,
    #[serde(serialize_with = "serialize_mtime")]
    mtime: Option<DateTime<Local>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<u64>,
    r#type: FileType,
    #[serde(skip_serializing)]
    uri: String,
}

impl FileEntry {
    fn is_dir(&self) -> bool {
        self.r#type == FileType::Directory
    }
}

/// Defines sorting attributes for file entries.
struct SortingAttr<'a> {
    name: &'a str,
    last_modified: &'a str,
    size: &'a str,
}

/// Defines read directory entries.
struct DirEntryOpts<'a> {
    dir_reader: std::fs::ReadDir,
    base_path: &'a str,
    uri_query: Option<&'a str>,
    is_head: bool,
    order_code: u8,
    content_format: &'a DirListFmt,
    ignore_hidden_files: bool,
    disable_symlinks: bool,
}

/// It reads a list of directory entries and create an index page content.
/// Otherwise it returns a status error.
fn read_dir_entries(mut opt: DirEntryOpts<'_>) -> Result<Response<Body>> {
    let mut dirs_count: usize = 0;
    let mut files_count: usize = 0;
    let mut file_entries: Vec<FileEntry> = vec![];

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
        if opt.ignore_hidden_files && name.as_encoded_bytes().first().is_some_and(|c| *c == b'.') {
            continue;
        }

        let (r#type, size) = if meta.is_dir() {
            dirs_count += 1;
            (FileType::Directory, None)
        } else if meta.is_file() {
            files_count += 1;
            (FileType::File, Some(meta.len()))
        } else if !opt.disable_symlinks && meta.file_type().is_symlink() {
            // NOTE: we resolve the symlink path below to just know if is a directory or not.
            // However, we are still showing the symlink name but not the resolved name.

            let symlink = dir_entry.path();
            let symlink = match symlink.canonicalize() {
                Ok(v) => v,
                Err(err) => {
                    tracing::error!(
                        "unable to resolve `{}` symlink path (skipped): {:?}",
                        symlink.display(),
                        err,
                    );
                    continue;
                }
            };

            let symlink_meta = match std::fs::symlink_metadata(&symlink) {
                Ok(v) => v,
                Err(err) => {
                    tracing::error!(
                        "unable to resolve metadata for `{}` symlink (skipped): {:?}",
                        symlink.display(),
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
        let mut parts = form_urlencoded::parse(q.as_bytes());
        if parts.count() > 0 {
            // NOTE: we just pick up the first value (pair)
            if let Some(sort) = parts.next() {
                if sort.0 == "sort" && !sort.1.trim().is_empty() {
                    match sort.1.parse::<u8>() {
                        Ok(code) => opt.order_code = code,
                        Err(err) => {
                            tracing::error!(
                                "sorting: query value error when converting to u8: {:?}",
                                err
                            );
                        }
                    }
                }
            }
        }
    }

    let mut resp = Response::new(Body::empty());

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
            )
        }
    };

    resp.headers_mut()
        .typed_insert(ContentLength(content.len() as u64));

    // We skip the body for HEAD requests
    if opt.is_head {
        return Ok(resp);
    }

    *resp.body_mut() = Body::from(content);

    Ok(resp)
}

/// Create an auto index in JSON format.
fn json_auto_index(entries: &mut [FileEntry], order_code: u8) -> Result<String> {
    sort_file_entries(entries, order_code);

    Ok(serde_json::to_string(entries)?)
}

/// Serialize FileEntry::name
fn serialize_name<S: Serializer>(name: &OsStr, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&name.to_string_lossy())
}

/// Serialize FileEntry::mtime field
fn serialize_mtime<S: Serializer>(
    mtime: &Option<DateTime<Local>>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    match mtime {
        Some(dt) => serializer.serialize_str(
            &dt.with_timezone(&Utc)
                .format(DATETIME_FORMAT_UTC)
                .to_string(),
        ),
        None => serializer.serialize_str(""),
    }
}

/// Create an auto index in HTML format.
fn html_auto_index<'a>(
    base_path: &'a str,
    dirs_count: usize,
    files_count: usize,
    entries: &'a mut [FileEntry],
    order_code: u8,
) -> String {
    use maud::{html, DOCTYPE};

    let sort_attrs = sort_file_entries(entries, order_code);
    let current_path = percent_decode_str(base_path).decode_utf8_lossy();

    html! {
        (DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width,minimum-scale=1,initial-scale=1";
                title {
                    "Index of " (current_path)
                }
                style {
                    "html{background-color:#fff;-moz-osx-font-smoothing:grayscale;-webkit-font-smoothing:antialiased;min-width:20rem;text-rendering:optimizeLegibility;-webkit-text-size-adjust:100%;-moz-text-size-adjust:100%;text-size-adjust:100%}:after,:before{box-sizing:border-box;}body{padding:1rem;font-family:Consolas,'Liberation Mono',Menlo,monospace;font-size:.75rem;max-width:70rem;margin:0 auto;color:#4a4a4a;font-weight:400;line-height:1.5}h1{margin:0;padding:0;font-size:1rem;line-height:1.25;margin-bottom:0.5rem;}table{width:100%;table-layout:fixed;border-spacing: 0;}hr{border-style: none;border-bottom: solid 1px gray;}table th,table td{padding:.15rem 0;white-space:nowrap;vertical-align:top}table th a,table td a{display:inline-block;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;max-width:95%;vertical-align:top;}table tr:hover td{background-color:#f5f5f5}footer{padding-top:0.5rem}table tr th{text-align:left;}@media (max-width:30rem){table th:first-child{width:20rem;}}"
                }
            }
            body {
                h1 {
                    "Index of " (current_path)
                }
            }
            p {
                small {
                    "directories: " (dirs_count) ", files: " (files_count)
                }
            }
            hr;
            div style="overflow-x: auto;" {
                table {
                    thead {
                        tr {
                            th {
                                a href={ "?sort=" (sort_attrs.name) } {
                                    "Name"
                                }
                            }
                            th style="width:10rem;" {
                                a href={ "?sort=" (sort_attrs.last_modified) } {
                                    "Last modified"
                                }
                            }
                            th style="width:6rem;text-align:right;" {
                                a href={ "?sort=" (sort_attrs.size) } {
                                    "Size"
                                }
                            }
                        }
                    }

                    @if base_path != "/" {
                        tr {
                            td colspan="3" {
                                a href="../" {
                                    "../"
                                }
                            }
                        }
                    }

                    @for entry in entries {
                        tr {
                            td {
                                a href=(entry.uri) {
                                    (entry.name.to_string_lossy())
                                    @if entry.is_dir() {
                                        "/"
                                    }
                                }
                            }
                            td {
                                (entry.mtime.map_or("-".to_owned(), |local_dt| {
                                    local_dt.format(DATETIME_FORMAT_LOCAL).to_string()
                                }))
                            }
                            td align="right" {
                                (entry.size.map(format_file_size).unwrap_or("-".into()))
                            }
                        }
                    }
                }
            }
            hr;
            footer {
                small {
                    "Powered by Static Web Server (SWS) / static-web-server.net"
                }
            }
        }
    }.into()
}

/// Sort a list of file entries by a specific order code.
fn sort_file_entries(files: &mut [FileEntry], order_code: u8) -> SortingAttr<'_> {
    // Default sorting type values
    let mut name = "0";
    let mut last_modified = "2";
    let mut size = "4";

    match order_code {
        0 | 1 => {
            // Name (asc, desc)
            files.sort_by_cached_key(|f| f.name.to_string_lossy().to_lowercase());
            if order_code == 1 {
                files.reverse();
            } else {
                name = "1";
            }
        }
        2 | 3 => {
            // Modified (asc, desc)
            files.sort_by_key(|f| f.mtime);
            if order_code == 3 {
                files.reverse();
            } else {
                last_modified = "3";
            }
        }
        4 | 5 => {
            // File size (asc, desc)
            files.sort_by_key(|f| f.size);
            if order_code == 5 {
                files.reverse();
            } else {
                size = "5";
            }
        }
        _ => {
            // Unsorted
        }
    }

    SortingAttr {
        name,
        last_modified,
        size,
    }
}

/// Formats the file size in bytes to a human-readable string
fn format_file_size(size: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB"];
    let mut size_tmp = size;

    if size_tmp < 1024 {
        // return the size with Byte
        return format!("{} {}", size_tmp, UNITS[0]);
    }

    for unit in &UNITS[1..UNITS.len() - 1] {
        if size_tmp < 1024 * 1024 {
            // return the size divided by 1024 with the unit
            return format!("{:.2} {}", size_tmp as f64 / 1024.0, unit);
        }
        size_tmp >>= 10;
    }

    // if size is too large, return the largest unit
    format!("{:.2} {}", size_tmp as f64 / 1024.0, UNITS[UNITS.len() - 1])
}

#[cfg(test)]
mod tests {
    use super::format_file_size;

    #[test]
    fn handle_byte() {
        let size = 128;
        assert_eq!("128 B", format_file_size(size))
    }

    #[test]
    fn handle_kibibyte() {
        let size = 1024;
        assert_eq!("1.00 KiB", format_file_size(size))
    }

    #[test]
    fn handle_mebibyte() {
        let size = 1048576;
        assert_eq!("1.00 MiB", format_file_size(size))
    }

    #[test]
    fn handle_gibibyte() {
        let size = 1073741824;
        assert_eq!("1.00 GiB", format_file_size(size))
    }

    #[test]
    fn handle_tebibyte() {
        let size = 1099511627776;
        assert_eq!("1.00 TiB", format_file_size(size))
    }

    #[test]
    fn handle_pebibyte() {
        let size = 1125899906842624;
        assert_eq!("1.00 PiB", format_file_size(size))
    }

    #[test]
    fn handle_large() {
        let size = u64::MAX;
        assert_eq!("16384.00 PiB", format_file_size(size))
    }
}
