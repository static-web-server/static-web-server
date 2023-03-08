use headers::{ContentCoding, HeaderMap, HeaderValue};
use std::{
    ffi::OsStr,
    fs::Metadata,
    path::{Path, PathBuf},
};

use crate::{compression, static_files::file_metadata};

/// It defines the pre-compressed file variant metadata of a particular file path.
pub struct CompressedFileVariant<'a> {
    pub file_path: PathBuf,
    pub metadata: Metadata,
    pub extension: &'a str,
}

/// Search for the pre-compressed variant of the given file path.
pub async fn precompressed_variant<'a>(
    file_path: &Path,
    headers: &'a HeaderMap<HeaderValue>,
) -> Option<CompressedFileVariant<'a>> {
    tracing::trace!(
        "preparing pre-compressed file variant path of {}",
        file_path.display()
    );

    // Determine prefered-encoding extension if available
    let comp_ext = match compression::get_prefered_encoding(headers) {
        // https://zlib.net/zlib_faq.html#faq39
        Some(ContentCoding::GZIP | ContentCoding::DEFLATE) => "gz",
        // https://peazip.github.io/brotli-compressed-file-format.html
        Some(ContentCoding::BROTLI) => "br",
        _ => {
            tracing::trace!(
                "preferred encoding based on the file extension was not determined, skipping"
            );
            return None;
        }
    };

    let comp_name = match file_path.file_name().and_then(OsStr::to_str) {
        Some(v) => v,
        None => {
            tracing::trace!("file name was not determined for the current path, skipping");
            return None;
        }
    };

    let file_path = file_path.with_file_name([comp_name, ".", comp_ext].concat());
    tracing::trace!(
        "trying to get the pre-compressed file variant metadata for {}",
        file_path.display()
    );

    let (metadata, is_dir) = match file_metadata(&file_path) {
        Ok(v) => v,
        Err(e) => {
            tracing::trace!("pre-compressed file variant error: {:?}", e);
            return None;
        }
    };

    if is_dir {
        tracing::trace!("pre-compressed file variant found but it's a directory, skipping");
        return None;
    }

    tracing::trace!("pre-compressed file variant found, serving it directly");

    Some(CompressedFileVariant {
        file_path,
        metadata,
        extension: if comp_ext == "gz" { "gzip" } else { comp_ext },
    })
}
