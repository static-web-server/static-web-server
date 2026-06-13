// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Module for providing MIME type extensions and utilities.
//!

use mime_guess::{Mime, mime};

/// Application types that are essentially text/structured data.
const TEXT_MIME_TYPES: [&str; 9] = [
    "application/csv",
    "application/graphql",
    "application/javascript",
    "application/json",
    "application/rtf",
    "application/sql",
    "application/x-yaml",
    "application/xml",
    "application/yaml",
];

/// Binary types that are not "text" but considered compressible.
const ADDITIONAL_COMPRESSIBLE_TYPES: [&str; 5] = [
    "application/wasm",
    "application/font-sfnt",
    "application/vnd.ms-fontobject",
    "image/x-icon",
    "image/vnd.microsoft.icon",
];

pub(crate) trait MimeExt {
    fn is_text(&self) -> bool;
    fn is_text_only(&self) -> bool;
    fn is_compressible(&self) -> bool;
    fn contains_text_only(&self, allowed: &[&str]) -> bool;
}

impl MimeExt for Mime {
    /// Determines if a MIME type is considered "text-only" and restricted to only `text/*` types.
    fn is_text_only(&self) -> bool {
        self.type_() == mime::TEXT
    }

    /// Determines if a MIME type is considered "text" based on its type and known characteristics.
    /// Not restricted to `text/*` only but also includes types with text-like suffixes (e.g., `application/problem+json`) and known text-based formats.
    fn is_text(&self) -> bool {
        // text/*
        if self.is_text_only() {
            return true;
        }

        // Suffixes (e.g., application/problem+json)
        if let Some(suffix) = self.suffix() {
            let s = suffix.as_str();
            if s == "json" || s == "xml" || s == "yaml" || s == "svg" {
                return true;
            }
        }

        TEXT_MIME_TYPES.contains(&self.essence_str())
    }

    /// Checks if the MIME type's essence matches any in the provided list, but only if it's a text-only type.
    fn contains_text_only(&self, list: &[&str]) -> bool {
        if !self.is_text_only() {
            return false;
        }
        let s = self.essence_str();
        list.contains(&s)
    }

    /// Determines if a MIME type is compressible based on its essence and known characteristics.
    fn is_compressible(&self) -> bool {
        let s = self.essence_str();

        // Text is always compressible
        if self.is_text() {
            return true;
        }

        // Font handling (e.g. TTF/OTF/WOFF(1))
        if self.type_() == mime::FONT {
            // WOFF2 is already compressed
            return s != "font/woff2";
        }

        // Known compressible binary formats
        ADDITIONAL_COMPRESSIBLE_TYPES.contains(&s)
    }
}

#[cfg(test)]
mod tests {
    use mime_guess::Mime;

    use super::MimeExt;

    #[test]
    fn test_is_text() {
        for ext in super::TEXT_MIME_TYPES.iter() {
            assert!(
                ext.parse::<Mime>().unwrap().is_text(),
                "'{ext}' should be considered text"
            );
        }
        const OTHER_TEXT_TYPES: [&str; 7] = [
            "application/problem+json",
            "application/problem+xml",
            "application/vnd.api+json",
            "application/vnd.api+xml",
            "image/svg+xml",
            "text/calendar",
            "text/csv",
        ];
        for ext in OTHER_TEXT_TYPES.iter() {
            assert!(
                ext.parse::<Mime>().unwrap().is_text(),
                "'{ext}' should be considered text due to suffix"
            );
        }
    }

    #[test]
    fn test_is_not_text() {
        const NON_TEXT_TYPES: [&str; 5] = [
            "application/font-woff2",
            "application/octet-stream",
            "application/wasm",
            "font/ttf",
            "image/png",
        ];
        for ext in NON_TEXT_TYPES.iter() {
            assert!(
                !ext.parse::<Mime>().unwrap().is_text(),
                "'{}' should not be considered text",
                ext
            );
        }
    }

    #[test]
    fn test_is_compressible() {
        for ext in super::TEXT_MIME_TYPES.iter() {
            assert!(
                ext.parse::<Mime>().unwrap().is_compressible(),
                "'{ext}' should be considered compressible"
            );
        }
        for ext in super::ADDITIONAL_COMPRESSIBLE_TYPES.iter() {
            assert!(
                ext.parse::<Mime>().unwrap().is_compressible(),
                "'{ext}' should be considered compressible"
            );
        }
    }

    #[test]
    fn test_is_not_compressible() {
        const NON_COMPRESSIBLE_TYPES: [&str; 7] = [
            "application/font-woff2",
            "application/gzip",
            "application/octet-stream",
            "application/pdf",
            "application/zip",
            "image/jpeg",
            "image/png",
        ];

        for ext in NON_COMPRESSIBLE_TYPES.iter() {
            assert!(
                !ext.parse::<Mime>().unwrap().is_compressible(),
                "'{ext}' should not be compressible"
            );
        }
    }

    #[test]
    fn test_is_text_only() {
        const TEXT_ONLY_TYPES: [&str; 7] = [
            "text/plain",
            "text/csv",
            "text/calendar",
            "text/css",
            "text/html",
            "text/markdown",
            "text/plain; boundary=x; charset=utf-8",
        ];
        for ext in TEXT_ONLY_TYPES.iter() {
            assert!(
                ext.parse::<Mime>().unwrap().is_text_only(),
                "'{ext}' should be considered text-only"
            );
        }
    }

    #[test]
    fn test_is_not_text_only() {
        const NON_TEXT_ONLY_TYPES: [&str; 4] = [
            "application/json",
            "application/problem+json",
            "application/xml",
            "image/svg+xml",
        ];
        for ext in NON_TEXT_ONLY_TYPES.iter() {
            assert!(
                !ext.parse::<Mime>().unwrap().is_text_only(),
                "'{ext}' should not be considered text-only"
            );
        }
    }
}
