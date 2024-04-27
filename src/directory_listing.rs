// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! It provides directory listig and auto-index support.
//!

use chrono::{DateTime, Local, Utc};
use clap::ValueEnum;
use futures_util::{future, future::Either};
use headers::{ContentLength, ContentType, HeaderMapExt};
use humansize::FormatSize;
use hyper::{Body, Method, Response, StatusCode};
use mime_guess::mime;
use percent_encoding::{percent_decode_str, utf8_percent_encode, NON_ALPHANUMERIC};
use serde::{Serialize, Serializer};
use std::future::Future;
use std::io;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{http_ext::MethodExt, Context, Result};

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
}

/// Provides directory listing support for the current request.
/// Note that this function highly depends on `static_files::composed_file_metadata()` function
/// which must be called first. See `static_files::handle()` for more details.
pub fn auto_index(
    opts: DirListOpts<'_>,
) -> impl Future<Output = Result<Response<Body>, StatusCode>> + Send + '_ {
    // Note: it's safe to call `parent()` here since `filepath`
    // value always refer to a path with file ending and under
    // a root directory boundary.
    // See `composed_file_metadata()` function which sanitizes the requested
    // path before to be delegated here.
    let filepath = opts.filepath;
    let parent = filepath.parent().unwrap_or(filepath);

    match std::fs::read_dir(parent) {
        Ok(dir_reader) => Either::Left(async move {
            let is_head = opts.method.is_head();
            match read_dir_entries(
                dir_reader,
                opts.current_path,
                opts.uri_query,
                is_head,
                opts.dir_listing_order,
                opts.dir_listing_format,
                opts.ignore_hidden_files,
            )
            .await
            {
                Ok(resp) => Ok(resp),
                Err(err) => {
                    tracing::error!("error after try to read directory entries: {:?}", err);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }),
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
            Either::Right(future::err(status))
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
    name: String,
    #[serde(skip_serializing)]
    name_encoded: String,
    #[serde(serialize_with = "serialize_mtime")]
    mtime: Option<DateTime<Local>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<u64>,
    r#type: FileType,
    #[serde(skip_serializing)]
    uri: Option<String>,
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

/// It reads a list of directory entries and create an index page content.
/// Otherwise it returns a status error.
async fn read_dir_entries(
    dir_reader: std::fs::ReadDir,
    base_path: &str,
    uri_query: Option<&str>,
    is_head: bool,
    mut order_code: u8,
    content_format: &DirListFmt,
    ignore_hidden_files: bool,
) -> Result<Response<Body>> {
    let mut dirs_count: usize = 0;
    let mut files_count: usize = 0;
    let mut file_entries: Vec<FileEntry> = vec![];

    for dir_entry in dir_reader {
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

        let name = match dir_entry
            .file_name()
            .into_string()
            .map_err(|err| anyhow::anyhow!(err.into_string().unwrap_or_default()))
        {
            Ok(s) => s,
            Err(err) => {
                tracing::error!(
                    "unable to resolve name for file or directory entry (skipped): {:?}",
                    err
                );
                continue;
            }
        };

        // Check and ignore the current hidden file/directory (dotfile) if feature enabled
        if ignore_hidden_files && name.starts_with('.') {
            continue;
        }

        let mut name_encoded = utf8_percent_encode(&name, NON_ALPHANUMERIC).to_string();
        let mut size = None;

        if meta.is_dir() {
            name_encoded.push('/');
            dirs_count += 1;
        } else if meta.is_file() {
            size = Some(meta.len());
            files_count += 1;
        } else if meta.file_type().is_symlink() {
            // NOTE: we resolve the symlink path below to just know if is a directory or not.
            // Hwever, we are still showing the symlink name but not the resolved name.

            let symlink = dir_entry.path();
            let symlink = match symlink.canonicalize() {
                Ok(v) => v,
                Err(err) => {
                    tracing::error!(
                        "unable to resolve `{}` symlink path (skipped): {:?}",
                        symlink.display(),
                        err
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
                        err
                    );
                    continue;
                }
            };
            if symlink_meta.is_dir() {
                name_encoded.push('/');
                dirs_count += 1;
            } else {
                size = Some(meta.len());
                files_count += 1;
            }
        } else {
            continue;
        }

        let mut uri = None;
        // NOTE: Use relative paths by default independently of
        // the "redirect trailing slash" feature.
        // However, when "redirect trailing slash" is disabled
        // and a request path doesn't contain a trailing slash then
        // entries should contain the "parent/entry-name" as a link format.
        // Otherwise, we just use the "entry-name" as a link (default behavior).
        // Note that in both cases, we add a trailing slash if the entry is a directory.
        if !base_path.ends_with('/') {
            let base_path = Path::new(base_path);
            let parent_dir = base_path.parent().unwrap_or(base_path);
            let mut base_dir = base_path;
            if base_path != parent_dir {
                base_dir = match base_path.strip_prefix(parent_dir) {
                    Ok(v) => v,
                    Err(err) => {
                        tracing::error!(
                            "unable to strip parent path prefix for `{}` (skipped): {:?}",
                            base_path.display(),
                            err
                        );
                        continue;
                    }
                };
            }

            let mut base_str = String::new();
            if !base_dir.starts_with("/") {
                let base_dir = base_dir.to_str().unwrap_or_default();
                if !base_dir.is_empty() {
                    base_str.push_str(base_dir);
                }
                base_str.push('/');
            }

            base_str.push_str(&name_encoded);
            uri = Some(base_str);
        }

        let mtime = match parse_last_modified(meta.modified()?) {
            Ok(local_dt) => Some(local_dt),
            Err(err) => {
                tracing::error!("error determining the file's last modified: {:?}", err);
                None
            }
        };
        let r#type = if meta.is_dir() {
            FileType::Directory
        } else {
            FileType::File
        };

        file_entries.push(FileEntry {
            name,
            name_encoded,
            mtime,
            size,
            r#type,
            uri,
        });
    }

    // Check the query request uri for a sorting type. E.g https://blah/?sort=5
    if let Some(q) = uri_query {
        let mut parts = form_urlencoded::parse(q.as_bytes());
        if parts.count() > 0 {
            // NOTE: we just pick up the first value (pair)
            if let Some(sort) = parts.next() {
                if sort.0 == "sort" && !sort.1.trim().is_empty() {
                    match sort.1.parse::<u8>() {
                        Ok(code) => order_code = code,
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
    let content = match content_format {
        DirListFmt::Json => {
            // JSON
            resp.headers_mut()
                .typed_insert(ContentType::from(mime::APPLICATION_JSON));

            json_auto_index(&mut file_entries, order_code)?
        }
        // HTML (default)
        _ => {
            resp.headers_mut()
                .typed_insert(ContentType::from(mime::TEXT_HTML_UTF_8));

            html_auto_index(
                base_path,
                dirs_count,
                files_count,
                &mut file_entries,
                order_code,
            )?
        }
    };

    resp.headers_mut()
        .typed_insert(ContentLength(content.len() as u64));

    // We skip the body for HEAD requests
    if is_head {
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
) -> Result<String> {
    use maud::{html, DOCTYPE};

    let sort_attrs = sort_file_entries(entries, order_code);
    let current_path = percent_decode_str(base_path).decode_utf8()?.to_string();

    Ok(html! {
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
                                a href=(entry.uri.as_ref().unwrap_or(&entry.name_encoded)) {
                                    (entry.name)
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
                                (match entry.size.unwrap_or(0) {
                                    0 => "-".to_owned(),
                                    size => size.format_size(humansize::DECIMAL)
                                })
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
    }.into())
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
            files.sort_by_cached_key(|f| f.name.to_lowercase());
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

/// Return the last modified `DateTime` in local timescale.
fn parse_last_modified(modified: SystemTime) -> Result<DateTime<Local>> {
    let since_epoch = modified.duration_since(UNIX_EPOCH)?;
    // HTTP times don't have nanosecond precision, so we truncate
    // the modification time.
    // Converting to i64 should be safe until we get beyond the
    // planned lifetime of the universe
    //
    // TODO: Investigate how to write a test for this. Changing
    // the modification time of a file with greater than second
    // precision appears to be something that only is possible to
    // do on Linux.
    let utc_dt = DateTime::from_timestamp(since_epoch.as_secs() as i64, since_epoch.subsec_nanos());
    match utc_dt {
        Some(utc_dt) => Ok(utc_dt.with_timezone(&Local)),
        None => Err(anyhow!(
            "out-of-range number of seconds and/or invalid nanosecond"
        )),
    }
}
