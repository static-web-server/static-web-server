// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! The static file module which powers the web server.
//!

// Part of the file is borrowed and adapted at a convenience from
// https://github.com/seanmonstar/warp/blob/master/src/filters/fs.rs

use futures_util::{
    future,
    future::{Either, Future},
};
use headers::{AcceptRanges, HeaderMap, HeaderMapExt, HeaderValue};
use hyper::{header::CONTENT_ENCODING, header::CONTENT_LENGTH, Body, Method, Response, StatusCode};
use std::fs::{File, Metadata};
use std::io;
use std::path::{Path, PathBuf};

use crate::conditional_headers::ConditionalHeaders;
use crate::file_path::{sanitize_path, PathExt};
use crate::Result;
use crate::{
    file_response::response_body,
    http_ext::{MethodExt, HTTP_SUPPORTED_METHODS},
};

#[cfg(feature = "compression")]
use crate::compression_static;

#[cfg(feature = "directory-listing")]
use crate::{
    directory_listing,
    directory_listing::{DirListFmt, DirListOpts},
};

const DEFAULT_INDEX_FILES: &[&str; 1] = &["index.html"];

/// Defines all options needed by the static-files handler.
pub struct HandleOpts<'a> {
    /// Request method.
    pub method: &'a Method,
    /// Request headers.
    pub headers: &'a HeaderMap<HeaderValue>,
    /// Request base path.
    pub base_path: &'a PathBuf,
    /// Request base path.
    pub uri_path: &'a str,
    /// Index files.
    pub index_files: &'a [&'a str],
    /// Request URI query.
    pub uri_query: Option<&'a str>,
    /// Directory listing feature.
    #[cfg(feature = "directory-listing")]
    #[cfg_attr(docsrs, doc(cfg(feature = "directory-listing")))]
    pub dir_listing: bool,
    /// Directory listing order feature.
    #[cfg(feature = "directory-listing")]
    #[cfg_attr(docsrs, doc(cfg(feature = "directory-listing")))]
    pub dir_listing_order: u8,
    /// Directory listing format feature.
    #[cfg(feature = "directory-listing")]
    #[cfg_attr(docsrs, doc(cfg(feature = "directory-listing")))]
    pub dir_listing_format: &'a DirListFmt,
    /// Redirect trailing slash feature.
    pub redirect_trailing_slash: bool,
    /// Compression static feature.
    pub compression_static: bool,
    /// Ignore hidden files feature.
    pub ignore_hidden_files: bool,
}

/// Static file response with additional data.
pub struct StaticFileResponse {
    /// Inner HTTP response.
    pub resp: Response<Body>,
    /// If the inner HTTP response is already pre-compressed.
    pub is_precompressed: bool,
    /// The file path of the inner HTTP response.
    pub file_path: PathBuf,
}

/// The server entry point to handle incoming requests which map to specific files
/// on file system and return a file response.
pub async fn handle<'a>(opts: &HandleOpts<'a>) -> Result<StaticFileResponse, StatusCode> {
    let method = opts.method;
    let uri_path = opts.uri_path;

    // Check if current HTTP method for incoming request is supported
    if !method.is_allowed() {
        return Err(StatusCode::METHOD_NOT_ALLOWED);
    }

    let headers_opt = opts.headers;
    let mut file_path = sanitize_path(opts.base_path, uri_path)?;

    let FileMetadata {
        file_path,
        metadata,
        is_dir,
        precompressed_variant,
    } = get_composed_file_metadata(
        &mut file_path,
        headers_opt,
        opts.compression_static,
        opts.index_files,
    )
    .await?;

    // Check for a hidden file/directory (dotfile) and ignore it if feature enabled
    if opts.ignore_hidden_files && file_path.is_hidden() {
        return Err(StatusCode::NOT_FOUND);
    }

    let resp_file_path = file_path.to_owned();

    // `is_precompressed` relates to `opts.compression_static` value
    let is_precompressed = precompressed_variant.is_some();

    // Check for a trailing slash on the current directory path
    // and redirect if that path doesn't end with the slash char
    if is_dir && opts.redirect_trailing_slash && !uri_path.ends_with('/') {
        let uri = [uri_path, "/"].concat();
        let loc = match HeaderValue::from_str(uri.as_str()) {
            Ok(val) => val,
            Err(err) => {
                tracing::error!("invalid header value from current uri: {:?}", err);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

        let mut resp = Response::new(Body::empty());
        resp.headers_mut().insert(hyper::header::LOCATION, loc);
        *resp.status_mut() = StatusCode::PERMANENT_REDIRECT;

        tracing::trace!("uri doesn't end with a slash so redirecting permanently");
        return Ok(StaticFileResponse {
            resp,
            is_precompressed,
            file_path: resp_file_path,
        });
    }

    // Respond with the permitted communication methods
    if method.is_options() {
        let mut resp = Response::new(Body::empty());
        *resp.status_mut() = StatusCode::NO_CONTENT;
        resp.headers_mut()
            .typed_insert(headers::Allow::from_iter(HTTP_SUPPORTED_METHODS.clone()));
        resp.headers_mut().typed_insert(AcceptRanges::bytes());

        return Ok(StaticFileResponse {
            resp,
            is_precompressed,
            file_path: resp_file_path,
        });
    }

    // Directory listing
    // Check if "directory listing" feature is enabled,
    // if current path is a valid directory and
    // if it does not contain an `index.html` file (if a proper auto index is generated)
    #[cfg(feature = "directory-listing")]
    if is_dir && opts.dir_listing && !file_path.exists() {
        let resp = directory_listing::auto_index(DirListOpts {
            method,
            current_path: uri_path,
            uri_query: opts.uri_query,
            filepath: file_path,
            dir_listing_order: opts.dir_listing_order,
            dir_listing_format: opts.dir_listing_format,
            ignore_hidden_files: opts.ignore_hidden_files,
        })
        .await?;

        return Ok(StaticFileResponse {
            resp,
            is_precompressed,
            file_path: resp_file_path,
        });
    }

    // Check for a pre-compressed file variant if present under the `opts.compression_static` context
    if let Some(precompressed_meta) = precompressed_variant {
        let (precomp_path, precomp_ext) = precompressed_meta;
        let mut resp = file_reply(headers_opt, file_path, &metadata, Some(precomp_path)).await?;

        // Prepare corresponding headers to let know how to decode the payload
        resp.headers_mut().remove(CONTENT_LENGTH);
        resp.headers_mut()
            .insert(CONTENT_ENCODING, precomp_ext.parse().unwrap());

        return Ok(StaticFileResponse {
            resp,
            is_precompressed,
            file_path: resp_file_path,
        });
    }

    let resp = file_reply(headers_opt, file_path, &metadata, None).await?;

    Ok(StaticFileResponse {
        resp,
        is_precompressed,
        file_path: resp_file_path,
    })
}

/// It defines a composed file metadata structure containing the current file
/// and its optional compressed variant.
struct FileMetadata<'a> {
    /// The current file path reference.
    pub file_path: &'a PathBuf,
    /// The metadata of current `file_path` by default.
    /// Note that if `precompressed_variant` has some value
    /// then the `metadata` value will correspond to the `precompressed_variant`.
    pub metadata: Metadata,
    // If either `file_path` or `precompressed_variant` is a directory.
    pub is_dir: bool,
    // The precompressed file variant for the current `file_path`.
    pub precompressed_variant: Option<(PathBuf, &'a str)>,
}

/// Returns the final composed metadata containing
/// the current `file_path` with its file metadata
/// as well as its optional pre-compressed variant.
async fn get_composed_file_metadata<'a>(
    mut file_path: &'a mut PathBuf,
    _headers: &'a HeaderMap<HeaderValue>,
    _compression_static: bool,
    mut index_files: &'a [&'a str],
) -> Result<FileMetadata<'a>, StatusCode> {
    tracing::trace!("getting metadata for file {}", file_path.display());

    match file_metadata(file_path) {
        Ok((mut metadata, is_dir)) => {
            if is_dir {
                // Try every index file variant in order
                if index_files.is_empty() {
                    index_files = DEFAULT_INDEX_FILES;
                }
                let mut index_found = false;
                for index in index_files {
                    // Append a HTML index page by default if it's a directory path (`autoindex`)
                    tracing::debug!("dir: appending {} to the directory path", index);
                    file_path.push(index);

                    // Pre-compressed variant check for the autoindex
                    #[cfg(feature = "compression")]
                    if _compression_static {
                        if let Some(p) =
                            compression_static::precompressed_variant(file_path, _headers).await
                        {
                            return Ok(FileMetadata {
                                file_path,
                                metadata: p.metadata,
                                is_dir: false,
                                precompressed_variant: Some((p.file_path, p.extension)),
                            });
                        }
                    }

                    // Otherwise, just fallback to finding the index.html
                    // and overwrite the current `meta`
                    // Also noting that it's still a directory request
                    if let Ok(meta_res) = file_metadata(file_path) {
                        (metadata, _) = meta_res;
                        index_found = true;
                        break;
                    }

                    // We remove only the appended index file
                    file_path.pop();
                    let new_meta: Option<Metadata>;
                    (file_path, new_meta) = suffix_file_html_metadata(file_path);
                    if let Some(new_meta) = new_meta {
                        metadata = new_meta;
                        index_found = true;
                        break;
                    }
                }

                // In case no index was found then we append the last index
                // of the list to preserve the previous behavior
                if !index_found && !index_files.is_empty() {
                    file_path.push(index_files.last().unwrap());
                }
            } else {
                // Fallback pre-compressed variant check for the specific file
                #[cfg(feature = "compression")]
                if _compression_static {
                    if let Some(p) =
                        compression_static::precompressed_variant(file_path, _headers).await
                    {
                        return Ok(FileMetadata {
                            file_path,
                            metadata: p.metadata,
                            is_dir: false,
                            precompressed_variant: Some((p.file_path, p.extension)),
                        });
                    }
                }
            }

            Ok(FileMetadata {
                file_path,
                metadata,
                is_dir,
                precompressed_variant: None,
            })
        }
        Err(err) => {
            // Pre-compressed variant check for the file not found
            #[cfg(feature = "compression")]
            if _compression_static {
                if let Some(p) =
                    compression_static::precompressed_variant(file_path, _headers).await
                {
                    return Ok(FileMetadata {
                        file_path,
                        metadata: p.metadata,
                        is_dir: false,
                        precompressed_variant: Some((p.file_path, p.extension)),
                    });
                }
            }

            // Otherwise, if the file path doesn't exist then
            // we try to find the path suffixed with `.html`.
            // For example: `/posts/article` will fallback to `/posts/article.html`
            let new_meta: Option<Metadata>;
            (file_path, new_meta) = suffix_file_html_metadata(file_path);

            #[cfg(feature = "compression")]
            match new_meta {
                Some(new_meta) => {
                    return Ok(FileMetadata {
                        file_path,
                        metadata: new_meta,
                        is_dir: false,
                        precompressed_variant: None,
                    })
                }
                _ => {
                    // Last pre-compressed variant check or the suffixed file not found
                    if _compression_static {
                        if let Some(p) =
                            compression_static::precompressed_variant(file_path, _headers).await
                        {
                            return Ok(FileMetadata {
                                file_path,
                                metadata: p.metadata,
                                is_dir: false,
                                precompressed_variant: Some((p.file_path, p.extension)),
                            });
                        }
                    }
                }
            }
            #[cfg(not(feature = "compression"))]
            if let Some(new_meta) = new_meta {
                return Ok(FileMetadata {
                    file_path,
                    metadata: new_meta,
                    is_dir: false,
                    precompressed_variant: None,
                });
            }

            Err(err)
        }
    }
}

#[inline]
/// Try to find the file system metadata for the given file path or returns an `Not Found` error.
pub fn file_metadata(file_path: &Path) -> Result<(Metadata, bool), StatusCode> {
    match std::fs::metadata(file_path) {
        Ok(meta) => {
            let is_dir = meta.is_dir();
            tracing::trace!("file found: {:?}", file_path);
            Ok((meta, is_dir))
        }
        Err(err) => {
            tracing::debug!("file not found: {:?} {:?}", file_path, err);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

/// Reply with the corresponding file content taking into account
/// its precompressed variant if any.
/// The `path` param should contains always the original requested file path and
/// the `meta` param value should corresponds to it.
/// However, if `path_precompressed` contains some value then
/// the `meta` param  value will belong to the `path_precompressed` (precompressed file variant).
fn file_reply<'a>(
    headers: &'a HeaderMap<HeaderValue>,
    path: &'a PathBuf,
    meta: &'a Metadata,
    path_precompressed: Option<PathBuf>,
) -> impl Future<Output = Result<Response<Body>, StatusCode>> + Send + 'a {
    let conditionals = ConditionalHeaders::new(headers);
    let file_path = path_precompressed.as_ref().unwrap_or(path);

    match File::open(file_path) {
        Ok(file) => Either::Left(response_body(file, path, meta, conditionals)),
        Err(err) => {
            let status = match err.kind() {
                io::ErrorKind::NotFound => {
                    tracing::debug!("file can't be opened or not found: {:?}", path.display());
                    StatusCode::NOT_FOUND
                }
                io::ErrorKind::PermissionDenied => {
                    tracing::warn!("file permission denied: {:?}", path.display());
                    StatusCode::FORBIDDEN
                }
                _ => {
                    tracing::error!("file open error (path={:?}): {} ", path.display(), err);
                    StatusCode::INTERNAL_SERVER_ERROR
                }
            };
            Either::Right(future::err(status))
        }
    }
}

/// Returns the result of trying to append a `.html` to the file path.
/// * If the suffixed html path exists, it mutates the path to the suffixed one and returns the `Metadata`
/// * If the suffixed html path doesn't exist, it reverts the path to it's original value
fn suffix_file_html_metadata(file_path: &mut PathBuf) -> (&mut PathBuf, Option<Metadata>) {
    tracing::debug!("file: appending .html to the path");

    if let Some(filename) = file_path.file_name() {
        let owned_filename = filename.to_os_string();
        let mut owned_filename_with_html = owned_filename.clone();

        owned_filename_with_html.push(".html");
        file_path.set_file_name(owned_filename_with_html);

        if let Ok(meta_res) = file_metadata(file_path) {
            let (meta, _) = meta_res;
            return (file_path, Some(meta));
        }

        file_path.set_file_name(owned_filename);
    }

    (file_path, None)
}
