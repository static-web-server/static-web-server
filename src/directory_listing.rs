// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! It provides directory listig and auto-index support.
//!

use chrono::{DateTime, Local};
use clap::ValueEnum;
use futures_util::{future, future::Either};
use headers::{ContentLength, ContentType, HeaderMapExt};
use hyper::{Body, Method, Response, StatusCode};
use mime_guess::mime;
use percent_encoding::{percent_decode_str, utf8_percent_encode, NON_ALPHANUMERIC};
use sailfish::TemplateOnce;
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

/// Defines a file entry and its properties.
struct FileEntry {
    name: String,
    name_encoded: String,
    modified: Option<DateTime<Local>>,
    filesize: u64,
    is_dir: bool,
    uri: Option<String>,
}

/// Defines sorting attributes for file entries.
struct SortingAttr {
    name: &'static str,
    last_modified: &'static str,
    size: &'static str,
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
        let mut filesize = 0_u64;

        if meta.is_dir() {
            name_encoded.push('/');
            dirs_count += 1;
        } else if meta.is_file() {
            filesize = meta.len();
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
                filesize = meta.len();
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

        let modified = match parse_last_modified(meta.modified()?) {
            Ok(local_dt) => Some(local_dt),
            Err(err) => {
                tracing::error!("error determining the file's last modified: {:?}", err);
                None
            }
        };
        let is_dir = meta.is_dir();

        file_entries.push(FileEntry {
            name,
            name_encoded,
            modified,
            filesize,
            is_dir,
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
    #[derive(TemplateOnce)]
    #[template(path = "directory_listing.json", rm_whitespace = true, escape = false)]
    struct DirListing<'a> {
        entries: &'a [FileEntry],
    }

    sort_file_entries(entries, order_code);

    Ok(DirListing { entries }.render_once()?)
}

/// Quotes a string value.
fn json_quote_str(s: &str) -> String {
    let mut r = String::from("\"");
    for c in s.chars() {
        match c {
            '\\' => r.push_str("\\\\"),
            '\u{0008}' => r.push_str("\\b"),
            '\u{000c}' => r.push_str("\\f"),
            '\n' => r.push_str("\\n"),
            '\r' => r.push_str("\\r"),
            '\t' => r.push_str("\\t"),
            '"' => r.push_str("\\\""),
            c if c.is_control() => r.push_str(format!("\\u{:04x}", c as u32).as_str()),
            c => r.push(c),
        };
    }
    r.push('\"');
    r
}

/// Create an auto index in HTML format.
fn html_auto_index<'a>(
    base_path: &'a str,
    dirs_count: usize,
    files_count: usize,
    entries: &'a mut [FileEntry],
    order_code: u8,
) -> Result<String> {
    #[derive(TemplateOnce)]
    #[template(path = "directory_listing.html", rm_whitespace = true)]
    struct DirListing<'a> {
        base_path: &'a str,
        current_path: String,
        dirs_count: usize,
        files_count: usize,
        sort_attrs: SortingAttr,
        entries: &'a [FileEntry],
    }

    let sort_attrs = sort_file_entries(entries, order_code);
    let current_path = percent_decode_str(base_path).decode_utf8()?.to_string();

    Ok(DirListing {
        base_path,
        current_path,
        dirs_count,
        files_count,
        sort_attrs,
        entries,
    }
    .render_once()?)
}

/// Sort a list of file entries by a specific order code.
fn sort_file_entries(files: &mut [FileEntry], order_code: u8) -> SortingAttr {
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
            files.sort_by_key(|f| f.modified);
            if order_code == 3 {
                files.reverse();
            } else {
                last_modified = "3";
            }
        }
        4 | 5 => {
            // File size (asc, desc)
            files.sort_by_key(|f| f.filesize);
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
