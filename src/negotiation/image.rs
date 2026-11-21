// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Image format negotiation module.
//!
//! A request for a JPEG, PNG or GIF image is served from a variant in a newer
//! format, stored next to the original with the format extension appended
//! (`photo.jpg.avif`, `photo.jpg.webp`), when the client names that format in
//! its `Accept` header.

use headers::HeaderMapExt;
use hyper::{Request, Response};
use std::path::Path;

use crate::Error;
use crate::body::Body;
use crate::exts::headers::Accept;
use crate::exts::http::{MethodExt, append_vary_accept};
use crate::handler_opts::RequestHandlerOpts;

use super::try_variant_uri;

/// File extensions and media types of the images that can have variants.
const SOURCE_TYPES: [(&str, &str); 4] = [
    ("jpg", "image/jpeg"),
    ("jpeg", "image/jpeg"),
    ("png", "image/png"),
    ("gif", "image/gif"),
];

/// File extensions and media types of the variants, in server preference
/// order: an AVIF file is usually smaller than a WebP one.
const VARIANT_TYPES: [(&str, &str); 2] = [("avif", "image/avif"), ("webp", "image/webp")];

/// Initializes image format negotiation.
pub fn init(enabled: bool, handler_opts: &mut RequestHandlerOpts) {
    handler_opts.image_negotiation = enabled;
    tracing::info!(enabled, "image negotiation");
}

/// Pre-process a request to check if an image variant should be served.
///
/// Returns the URI path of the variant to serve instead, as described in
/// [`try_variant_uri`]. `None` keeps the requested URI path.
///
/// Only a literal image extension at the end of the URI path is matched, not
/// a percent-encoded one.
pub(crate) fn pre_process<T>(
    req: &Request<T>,
    base_path: &Path,
    uri_path: &str,
    include_hidden: bool,
) -> Option<String> {
    let method = req.method();
    if !method.is_get() && !method.is_head() {
        return None;
    }

    let source_type = source_media_type(uri_path)?;
    let accept = req.headers().typed_get::<Accept>()?;
    let mut variants = ranked_variants(&accept, source_type).peekable();
    variants.peek()?;

    try_variant_uri(base_path, uri_path, include_hidden, variants)
}

/// Post-process the response to add `Vary: Accept` to image responses.
///
/// Every `GET` and `HEAD` response for an image that can have variants gets
/// it, whether a variant was served or not, so a shared cache never gives
/// one client the format negotiated for another. Custom headers can still
/// override it.
pub(crate) fn post_process<T>(
    opts: &RequestHandlerOpts,
    req: &Request<T>,
    mut resp: Response<Body>,
) -> Result<Response<Body>, Error> {
    if !opts.image_negotiation {
        return Ok(resp);
    }

    let method = req.method();
    if (method.is_get() || method.is_head()) && source_media_type(req.uri().path()).is_some() {
        append_vary_accept(&mut resp);
    }

    Ok(resp)
}

/// Returns the media type of the image a URI path points to, when that image
/// type can have variants.
fn source_media_type(uri_path: &str) -> Option<&'static str> {
    let (_, ext) = uri_path.rsplit_once('.')?;
    SOURCE_TYPES
        .iter()
        .find(|(source_ext, _)| ext.eq_ignore_ascii_case(source_ext))
        .map(|&(_, media_type)| media_type)
}

/// Returns the extensions of the variants the client accepts at least as much
/// as the original, most wanted first.
///
/// Only a variant the client names counts: `image/*` and `*/*` are also sent
/// by clients that can't decode AVIF or WebP. Among equal weights the server
/// preference decides, and a variant wins a tie with the original since it is
/// the reason to have one. An original matched by no media range is not
/// acceptable (RFC 9110, section 12.5.1), so any named variant wins.
fn ranked_variants(accept: &Accept, source_type: &str) -> impl Iterator<Item = &'static str> {
    let source_quality = accept.quality(source_type).unwrap_or(0);
    let mut variants = VARIANT_TYPES.map(|(ext, media_type)| {
        let quality = accept.listed_quality(media_type).unwrap_or(0);
        (ext, quality)
    });
    // A stable sort, so equal weights keep the server preference order
    variants.sort_by_key(|&(_, quality)| std::cmp::Reverse(quality));
    variants
        .into_iter()
        .filter(move |&(_, quality)| quality > 0 && quality >= source_quality)
        .map(|(ext, _)| ext)
}

#[cfg(test)]
mod tests {
    use super::*;
    use headers::HeaderValue;

    /// `Accept` sent by Chrome and Edge for images.
    const CHROME: &str = "image/avif,image/webp,image/apng,image/svg+xml,image/*,*/*;q=0.8";
    /// `Accept` sent by Firefox for images.
    const FIREFOX: &str = "image/avif,image/webp,image/png,image/svg+xml,image/*;q=0.8,*/*;q=0.5";
    /// `Accept` sent by Safari for images.
    const SAFARI: &str = "image/webp,image/avif,image/jxl,image/heic,image/heic-sequence,\
        video/*;q=0.8,image/png,image/svg+xml,image/*;q=0.8,*/*;q=0.5";

    fn accept(value: &'static str) -> Accept {
        let mut headers = http::HeaderMap::new();
        headers.insert(http::header::ACCEPT, HeaderValue::from_static(value));
        headers.typed_get::<Accept>().unwrap()
    }

    fn ranked(value: &'static str, source_type: &str) -> Vec<&'static str> {
        ranked_variants(&accept(value), source_type).collect()
    }

    fn image_request(method: &str, uri_path: &str, accept: &'static str) -> Request<()> {
        Request::builder()
            .method(method)
            .uri(uri_path)
            .header("Accept", accept)
            .body(())
            .unwrap()
    }

    /// A web root with `photo.jpg` and both of its variants, `logo.png` and
    /// its WebP variant, and an orphan `orphan.gif.webp`.
    fn image_root() -> (tempfile::TempDir, std::path::PathBuf) {
        let tmp = tempfile::tempdir().unwrap();
        let base = tmp.path().canonicalize().unwrap();
        for name in [
            "photo.jpg",
            "photo.jpg.avif",
            "photo.jpg.webp",
            "logo.png",
            "logo.png.webp",
            "orphan.gif.webp",
        ] {
            std::fs::write(base.join(name), name).unwrap();
        }
        (tmp, base)
    }

    #[test]
    fn source_media_type_by_extension() {
        assert_eq!(source_media_type("/photo.jpg"), Some("image/jpeg"));
        assert_eq!(source_media_type("/a/photo.JPEG"), Some("image/jpeg"));
        assert_eq!(source_media_type("/logo.png"), Some("image/png"));
        assert_eq!(source_media_type("/anim.gif"), Some("image/gif"));
        assert_eq!(source_media_type("/photo.webp"), None);
        assert_eq!(source_media_type("/photo.jpg.webp"), None);
        assert_eq!(source_media_type("/photo.jpg/"), None);
        assert_eq!(source_media_type("/jpg"), None);
        assert_eq!(source_media_type("/photo.jp%67"), None);
    }

    #[test]
    fn ranked_variants_for_browsers() {
        assert_eq!(ranked(CHROME, "image/jpeg"), ["avif", "webp"]);
        assert_eq!(ranked(FIREFOX, "image/png"), ["avif", "webp"]);
        assert_eq!(ranked(SAFARI, "image/png"), ["avif", "webp"]);
    }

    #[test]
    fn ranked_variants_require_a_named_format() {
        assert!(ranked("*/*", "image/jpeg").is_empty());
        assert!(ranked("image/*", "image/jpeg").is_empty());
        assert!(ranked("image/png, image/svg+xml", "image/png").is_empty());
        assert_eq!(ranked("image/webp, */*", "image/jpeg"), ["webp"]);
    }

    #[test]
    fn ranked_variants_honor_weights() {
        assert_eq!(
            ranked("image/webp, image/avif;q=0.9, */*;q=0.8", "image/jpeg"),
            ["webp", "avif"]
        );
        assert_eq!(
            ranked("image/avif;q=0, image/webp, */*", "image/jpeg"),
            ["webp"]
        );
    }

    #[test]
    fn ranked_variants_skip_formats_less_wanted_than_original() {
        assert_eq!(
            ranked("image/png, image/avif;q=0.5, image/webp", "image/png"),
            ["webp"]
        );
        assert!(ranked("image/png, image/webp;q=0.5", "image/png").is_empty());
        // `image/*` wants the original more than the named format
        assert!(ranked("image/*, image/webp;q=0.5", "image/gif").is_empty());
    }

    #[test]
    fn ranked_variants_when_original_not_acceptable() {
        assert_eq!(ranked("image/webp", "image/jpeg"), ["webp"]);
        assert_eq!(
            ranked("image/jpeg;q=0, image/webp;q=0.1, */*", "image/jpeg"),
            ["webp"]
        );
    }

    #[test]
    fn pre_process_serves_most_wanted_variant() {
        let (_tmp, base) = image_root();
        let req = image_request("GET", "/photo.jpg", CHROME);
        let result = pre_process(&req, &base, "/photo.jpg", false);
        assert_eq!(result.as_deref(), Some("/photo.jpg.avif"));

        let req = image_request("HEAD", "/photo.jpg", "image/webp,*/*");
        let result = pre_process(&req, &base, "/photo.jpg", false);
        assert_eq!(result.as_deref(), Some("/photo.jpg.webp"));
    }

    #[test]
    fn pre_process_falls_back_to_next_variant_on_disk() {
        let (_tmp, base) = image_root();
        let req = image_request("GET", "/logo.png", CHROME);
        let result = pre_process(&req, &base, "/logo.png", false);
        assert_eq!(result.as_deref(), Some("/logo.png.webp"));
    }

    #[test]
    fn pre_process_keeps_original_without_variant_or_original() {
        let (_tmp, base) = image_root();
        for uri_path in ["/missing.jpg", "/orphan.gif", "/photo.jpg.webp"] {
            let req = image_request("GET", uri_path, CHROME);
            assert_eq!(
                pre_process(&req, &base, uri_path, false),
                None,
                "{uri_path}"
            );
        }
    }

    #[test]
    fn pre_process_keeps_original_for_other_clients() {
        let (_tmp, base) = image_root();
        for accept in ["*/*", "image/*", "image/jpeg, image/webp;q=0.5"] {
            let req = image_request("GET", "/photo.jpg", accept);
            assert_eq!(
                pre_process(&req, &base, "/photo.jpg", false),
                None,
                "{accept}"
            );
        }

        let req = Request::get("/photo.jpg").body(()).unwrap();
        assert_eq!(pre_process(&req, &base, "/photo.jpg", false), None);
    }

    #[test]
    fn pre_process_only_negotiates_get_and_head() {
        let (_tmp, base) = image_root();
        for method in ["OPTIONS", "POST", "PUT", "DELETE"] {
            let req = image_request(method, "/photo.jpg", CHROME);
            assert_eq!(
                pre_process(&req, &base, "/photo.jpg", false),
                None,
                "{method}"
            );
        }
    }

    #[test]
    fn pre_process_keeps_extension_case() {
        let (_tmp, base) = image_root();
        std::fs::write(base.join("UPPER.JPG"), "jpg").unwrap();
        std::fs::write(base.join("UPPER.JPG.webp"), "webp").unwrap();
        let req = image_request("GET", "/UPPER.JPG", CHROME);
        let result = pre_process(&req, &base, "/UPPER.JPG", false);
        assert_eq!(result.as_deref(), Some("/UPPER.JPG.webp"));
    }

    #[test]
    fn post_process_varies_image_responses_only() {
        let opts = RequestHandlerOpts {
            image_negotiation: true,
            ..Default::default()
        };
        let vary = |method: &str, uri: &str| {
            let req = image_request(method, uri, CHROME);
            let resp = post_process(&opts, &req, Response::new(crate::body::empty())).unwrap();
            resp.headers()
                .get(http::header::VARY)
                .map(|v| v.to_str().unwrap().to_owned())
        };

        assert_eq!(vary("GET", "/photo.jpg").as_deref(), Some("accept"));
        assert_eq!(vary("HEAD", "/logo.PNG").as_deref(), Some("accept"));
        assert_eq!(vary("GET", "/missing.gif").as_deref(), Some("accept"));
        assert_eq!(vary("OPTIONS", "/photo.jpg"), None);
        assert_eq!(vary("GET", "/photo.jpg.webp"), None);
        assert_eq!(vary("GET", "/index.html"), None);
    }

    #[test]
    fn post_process_disabled_adds_nothing() {
        let opts = RequestHandlerOpts::default();
        let req = image_request("GET", "/photo.jpg", CHROME);
        let resp = post_process(&opts, &req, Response::new(crate::body::empty())).unwrap();
        assert!(resp.headers().get(http::header::VARY).is_none());
    }
}
