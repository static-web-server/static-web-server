use iron::headers::{AcceptEncoding, ContentType, Encoding};
use mime::Mime;
use std::option::Option;

// Contains a common fixed list of text-based MIME types for Gzip compression
pub const TEXT_MIME_TYPES: [&str; 16] = [
    "text/html",
    "text/css",
    "text/javascript",
    "text/xml",
    "text/plain",
    "text/x-component",
    "application/javascript",
    "application/x-javascript",
    "application/json",
    "application/xml",
    "application/rss+xml",
    "application/atom+xml",
    "font/truetype",
    "font/opentype",
    "application/vnd.ms-fontobject",
    "image/svg+xml",
];

// Checks if a `content-type` header is a common text-based MIME type for Gzip compression
pub fn is_text_mime_type(content_type: Option<&ContentType>) -> bool {
    match content_type {
        Some(content_type) => TEXT_MIME_TYPES
            .iter()
            .any(|h| h.parse::<Mime>().unwrap() == content_type.0),
        None => false,
    }
}

// Checks if an `accept-encoding` header accepts Gzip encoding.
pub fn accept_gzip(accept_encoding: Option<&AcceptEncoding>) -> bool {
    match accept_encoding {
        Some(accept) => accept.0.iter().any(|qi| qi.item == Encoding::Gzip),
        None => false,
    }
}
