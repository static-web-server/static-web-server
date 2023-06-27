// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! It provides directory listig and auto-index support.
//!

use chrono::{DateTime, Local, NaiveDateTime, Utc};
use clap::ValueEnum;
use futures_util::future::Either;
use futures_util::{future, FutureExt};
use headers::{ContentLength, ContentType, HeaderMapExt};
use humansize::FormatSize;
use hyper::{Body, Method, Response, StatusCode};
use mime_guess::mime;
use percent_encoding::{percent_decode_str, utf8_percent_encode, NON_ALPHANUMERIC};
use std::cmp::Ordering;
use std::future::Future;
use std::io;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{exts::http::MethodExt, Context, Result};

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

    tokio::fs::read_dir(parent).then(move |res| match res {
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
    })
}

const STYLE: &str = r#"<style>html{background-color:#fff;-moz-osx-font-smoothing:grayscale;-webkit-font-smoothing:antialiased;min-width:20rem;text-rendering:optimizeLegibility;-webkit-text-size-adjust:100%;-moz-text-size-adjust:100%;text-size-adjust:100%}body{padding:1rem;font-family:Consolas,'Liberation Mono',Menlo,monospace;font-size:.875rem;max-width:70rem;margin:0 auto;color:#4a4a4a;font-weight:400;line-height:1.5}h1{margin:0;padding:0;font-size:1.375rem;line-height:1.25;margin-bottom:0.5rem;}table{width:100%;border-spacing: 0;}table th,table td{padding:.2rem .5rem;white-space:nowrap;vertical-align:top}table th a,table td a{display:inline-block;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;max-width:95%;vertical-align:top}table tr:hover td{background-color:#f5f5f5}footer{padding-top:0.5rem}table tr th{text-align:left;}</style>"#;
const FOOTER: &str = r#"<footer>Powered by <a target="_blank" href="https://static-web-server.net">Static Web Server</a> | MIT &amp; Apache 2.0</footer>"#;

const DATETIME_FORMAT_UTC: &str = "%FT%TZ";
const DATETIME_FORMAT_LOCAL: &str = "%F %T";

/// Defines a file entry and its properties.
struct FileEntry {
    name: String,
    name_encoded: String,
    modified: Option<DateTime<Local>>,
    filesize: u64,
    uri: Option<String>,
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
    mut dir_reader: tokio::fs::ReadDir,
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

    while let Some(dir_entry) = dir_reader
        .next_entry()
        .await
        .with_context(|| "unable to read directory entry")?
    {
        let meta = match dir_entry.metadata().await {
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

            let symlink_meta = match tokio::fs::symlink_metadata(&symlink).await {
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
        file_entries.push(FileEntry {
            name,
            name_encoded,
            modified,
            filesize,
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

    let mut json = String::from('[');

    for entry in entries {
        let file_size = &entry.filesize;
        let file_name = &entry.name;
        let is_empty = *file_size == 0_u64;
        let file_type = if is_empty { "directory" } else { "file" };
        let file_modified = &entry.modified;

        json.push('{');
        json.push_str(format!("\"name\":{},", json_quote_str(file_name.as_str())).as_str());
        json.push_str(format!("\"type\":\"{file_type}\",").as_str());

        let file_modified_str = file_modified.map_or("".to_owned(), |local_dt| {
            local_dt
                .with_timezone(&Utc)
                .format(DATETIME_FORMAT_UTC)
                .to_string()
        });
        json.push_str(format!("\"mtime\":\"{file_modified_str}\"").as_str());

        if !is_empty {
            json.push_str(format!(",\"size\":{file_size}").as_str());
        }
        json.push_str("},");
    }

    // Strip trailing comma out in case of available items
    if json.len() > 1 {
        json.pop();
    }

    json.push(']');

    Ok(json)
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
    let sort_attrs = sort_file_entries(entries, order_code);

    // Create the table header specifying every order code column
    let table_header = format!(
        r#"<thead><tr><th><a href="?sort={}">Name</a></th><th style="width:160px;"><a href="?sort={}">Last modified</a></th><th style="width:120px;text-align:right;"><a href="?sort={}">Size</a></th></tr></thead>"#,
        sort_attrs.name, sort_attrs.last_modified, sort_attrs.size,
    );

    // Prepare table row template
    let mut table_row = String::new();
    if base_path != "/" {
        table_row = String::from(r#"<tr><td colspan="3"><a href="../">../</a></td></tr>"#);
    }

    for entry in entries {
        let file_name = &entry.name_encoded;
        let file_modified = &entry.modified;
        let file_uri = &entry.uri.clone().unwrap_or_else(|| file_name.to_owned());
        let file_name_decoded = percent_decode_str(file_name).decode_utf8()?.to_string();
        let mut filesize_str = entry.filesize.format_size(humansize::DECIMAL);

        if entry.filesize == 0 {
            filesize_str = String::from("-");
        }

        let file_modified_str = file_modified.map_or("-".to_owned(), |local_dt| {
            local_dt.format(DATETIME_FORMAT_LOCAL).to_string()
        });

        table_row = format!(
            "{table_row}<tr><td><a href=\"{file_uri}\">{file_name_decoded}</a></td><td>{file_modified_str}</td><td align=\"right\">{filesize_str}</td></tr>"
        );
    }

    let current_path = percent_decode_str(base_path).decode_utf8()?.to_string();
    let dirs_str = if dirs_count == 1 {
        "directory"
    } else {
        "directories"
    };
    let summary = format!(
        "<div>{} {}, {} {}</div>",
        dirs_count, dirs_str, files_count, "file(s)"
    );

    let html_page = format!(
        "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><title>Index of {current_path}</title>{STYLE}</head><body><h1>Index of {current_path}</h1>{summary}<hr><table>{table_header}{table_row}</table><hr>{FOOTER}</body></html>"
    );

    Ok(html_page)
}

/// Sort a list of file entries by a specific order code.
fn sort_file_entries(files: &mut [FileEntry], order_code: u8) -> SortingAttr<'_> {
    // Default sorting type values
    let mut name = "0";
    let mut last_modified = "2";
    let mut size = "4";

    files.sort_by(|a, b| match order_code {
        // Name (asc, desc)
        0 => {
            name = "1";
            a.name.to_lowercase().cmp(&b.name.to_lowercase())
        }
        1 => {
            name = "0";
            b.name.to_lowercase().cmp(&a.name.to_lowercase())
        }

        // Modified (asc, desc)
        2 => {
            last_modified = "3";
            a.modified.cmp(&b.modified)
        }
        3 => {
            last_modified = "2";
            b.modified.cmp(&a.modified)
        }

        // File size (asc, desc)
        4 => {
            size = "5";
            a.filesize.cmp(&b.filesize)
        }
        5 => {
            size = "4";
            b.filesize.cmp(&a.filesize)
        }

        // Unordered
        _ => Ordering::Equal,
    });

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

    let utc_dt =
        NaiveDateTime::from_timestamp_opt(since_epoch.as_secs() as i64, since_epoch.subsec_nanos());

    match utc_dt {
        Some(utc_dt) => Ok(DateTime::<Utc>::from_utc(utc_dt, Utc).with_timezone(&Local)),
        None => Err(anyhow!(
            "out-of-range number of seconds and/or invalid nanosecond"
        )),
    }
}
