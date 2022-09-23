use futures_util::future::Either;
use futures_util::{future, FutureExt};
use headers::{ContentLength, ContentType, HeaderMapExt};
use humansize::{file_size_opts, FileSize};
use hyper::{Body, Method, Response, StatusCode};
use mime_guess::mime;
use percent_encoding::percent_decode_str;
use std::cmp::Ordering;
use std::future::Future;
use std::io;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::Result;

/// Provides directory listing support for the current request.
/// Note that this function highly depends on `static_files::get_composed_metadata()` function
/// which must be called first. See `static_files::handle()` for more details.
pub fn auto_index<'a>(
    method: &'a Method,
    current_path: &'a str,
    uri_query: Option<&'a str>,
    filepath: &'a Path,
    dir_listing_order: u8,
) -> impl Future<Output = Result<Response<Body>, StatusCode>> + Send + 'a {
    let is_head = method == Method::HEAD;

    // Note: it's safe to call `parent()` here since `filepath`
    // value always refer to a path with file ending and under
    // a root directory boundary.
    // See `get_composed_metadata()` function which sanitizes the requested
    // path before to be delegated here.
    let parent = filepath.parent().unwrap_or(filepath);

    tokio::fs::read_dir(parent).then(move |res| match res {
        Ok(entries) => Either::Left(async move {
            match read_dir_entries(entries, current_path, uri_query, is_head, dir_listing_order)
                .await
            {
                Ok(resp) => Ok(resp),
                Err(err) => {
                    tracing::error!(
                        "error during directory entries reading (path={:?}): {} ",
                        parent.display(),
                        err
                    );
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }),
        Err(err) => {
            let status = match err.kind() {
                io::ErrorKind::NotFound => {
                    tracing::debug!("entry file not found: {:?}", filepath.display());
                    StatusCode::NOT_FOUND
                }
                io::ErrorKind::PermissionDenied => {
                    tracing::warn!("entry file permission denied: {:?}", filepath.display());
                    StatusCode::FORBIDDEN
                }
                _ => {
                    tracing::error!(
                        "directory entries error (filepath={:?}): {} ",
                        filepath.display(),
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
const FOOTER: &str = r#"<footer>Powered by <a target="_blank" href="https://github.com/joseluisq/static-web-server">static-web-server</a> | MIT &amp; Apache 2.0</footer>"#;

/// It reads a list of directory entries and create an index page content.
/// Otherwise it returns a status error.
async fn read_dir_entries(
    mut file_entries: tokio::fs::ReadDir,
    base_path: &str,
    uri_query: Option<&str>,
    is_head: bool,
    mut dir_listing_order: u8,
) -> Result<Response<Body>> {
    let mut dirs_count: usize = 0;
    let mut files_count: usize = 0;
    let mut files_found: Vec<(String, String, u64, Option<String>)> = vec![];

    while let Some(entry) = file_entries.next_entry().await? {
        let meta = entry.metadata().await?;

        let mut name = entry
            .file_name()
            .into_string()
            .map_err(|err| anyhow::anyhow!(err.into_string().unwrap_or_default()))?;

        let mut filesize = 0_u64;

        if meta.is_dir() {
            name += "/";
            dirs_count += 1;
        } else if meta.is_file() {
            filesize = meta.len();
            files_count += 1;
        } else if meta.file_type().is_symlink() {
            let m = tokio::fs::symlink_metadata(entry.path().canonicalize()?).await?;
            if m.is_dir() {
                name += "/";
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
                base_dir = base_path.strip_prefix(parent_dir)?;
            }
            let mut base_str = String::new();
            if !base_dir.starts_with("/") {
                let base_dir = base_dir.to_str().unwrap_or_default();
                if !base_dir.is_empty() {
                    base_str.push_str(base_dir);
                }
                base_str.push('/');
            }
            base_str.push_str(&name);
            uri = Some(base_str);
        }

        let modified = match parse_last_modified(meta.modified()?) {
            Ok(tm) => tm.to_local().strftime("%F %T")?.to_string(),
            Err(err) => {
                tracing::error!("error determining file last modified: {:?}", err);
                String::from("-")
            }
        };
        files_found.push((name, modified, filesize, uri));
    }

    // Check the query request uri for a sorting type. E.g https://blah/?sort=5
    if let Some(q) = uri_query {
        let mut parts = form_urlencoded::parse(q.as_bytes());
        if parts.count() > 0 {
            // NOTE: we just pick up the first value (pair)
            if let Some(sort) = parts.next() {
                if sort.0 == "sort" && !sort.1.trim().is_empty() {
                    match sort.1.parse::<u8>() {
                        Ok(order_code) => dir_listing_order = order_code,
                        Err(err) => {
                            tracing::debug!(
                                "sorting: query value error when converting to u8: {:?}",
                                err
                            );
                        }
                    }
                }
            }
        }
    }

    let html = create_auto_index(
        base_path,
        dirs_count,
        files_count,
        dir_listing_order,
        &mut files_found,
    )?;

    let mut resp = Response::new(Body::empty());
    resp.headers_mut()
        .typed_insert(ContentType::from(mime::TEXT_HTML_UTF_8));
    resp.headers_mut()
        .typed_insert(ContentLength(html.len() as u64));

    // We skip the body for HEAD requests
    if is_head {
        return Ok(resp);
    }

    *resp.body_mut() = Body::from(html);

    Ok(resp)
}

/// Create an auto index html content.
fn create_auto_index(
    base_path: &str,
    dirs_count: usize,
    files_count: usize,
    dir_listing_order: u8,
    files_found: &mut Vec<(String, String, u64, Option<String>)>,
) -> Result<String> {
    // Sorting the files by an specific order code and create the table header
    let table_header = create_table_header(sort_files(files_found, dir_listing_order));

    // Prepare table row
    let mut table_row = String::new();
    if base_path != "/" {
        table_row = String::from(r#"<tr><td colspan="3"><a href="../">../</a></td></tr>"#);
    }

    for file in files_found {
        let (file_name, file_modified, file_size, uri) = file;
        let mut filesize_str = file_size
            .file_size(file_size_opts::DECIMAL)
            .map_err(anyhow::Error::msg)?;

        if *file_size == 0_u64 {
            filesize_str = String::from("-");
        }

        let file_uri = uri.clone().unwrap_or_else(|| file_name.to_owned());

        table_row = format!(
            "{}<tr><td><a href=\"{}\">{}</a></td><td>{}</td><td align=\"right\">{}</td></tr>",
            table_row, file_uri, file_name, file_modified, filesize_str
        );
    }

    let current_path = percent_decode_str(base_path).decode_utf8()?.to_owned();
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
        "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><title>Index of {}</title>{}</head><body><h1>Index of {}</h1>{}<hr><table>{}{}</table><hr>{}</body></html>",
        current_path, STYLE, current_path, summary, table_header, table_row, FOOTER
    );

    Ok(html_page)
}

/// Create a table header providing the sorting attributes.
fn create_table_header(sorting_attrs: (String, String, String)) -> String {
    let (name, last_modified, size) = sorting_attrs;
    format!(
        r#"<thead><tr><th><a href="?sort={}">Name</a></th><th style="width:160px;"><a href="?sort={}">Last modified</a></th><th style="width:120px;text-align:right;"><a href="?sort={}">Size</a></th></tr></thead>"#,
        name, last_modified, size,
    )
}

/// Sort a list of files by an specific order code.
fn sort_files(
    files: &mut [(String, String, u64, Option<String>)],
    order_code: u8,
) -> (String, String, String) {
    // Default sorting type values
    let mut name = "0".to_owned();
    let mut last_modified = "2".to_owned();
    let mut size = "4".to_owned();

    files.sort_by(|a, b| match order_code {
        // Name (asc, desc)
        0 => {
            name = "1".to_owned();
            a.0.to_lowercase().cmp(&b.0.to_lowercase())
        }
        1 => {
            name = "0".to_owned();
            b.0.to_lowercase().cmp(&a.0.to_lowercase())
        }

        // Modified (asc, desc)
        2 => {
            last_modified = "3".to_owned();
            a.1.cmp(&b.1)
        }
        3 => {
            last_modified = "2".to_owned();
            b.1.cmp(&a.1)
        }

        // File size (asc, desc)
        4 => {
            size = "5".to_owned();
            a.2.cmp(&b.2)
        }
        5 => {
            size = "4".to_owned();
            b.2.cmp(&a.2)
        }

        // Unordered
        _ => Ordering::Equal,
    });

    (name, last_modified, size)
}

fn parse_last_modified(modified: SystemTime) -> Result<time::Tm, Box<dyn std::error::Error>> {
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
    let ts = time::Timespec::new(since_epoch.as_secs() as i64, 0);
    Ok(time::at_utc(ts))
}
