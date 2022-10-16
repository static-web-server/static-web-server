use headers::{ContentCoding, HeaderMap, HeaderValue};
use std::{fs::Metadata, path::PathBuf};

use crate::{compression, static_files::file_metadata};

/// Search for the pre-compressed variant of the given file path.
pub async fn precompressed_variant(
    file_path: PathBuf,
    headers: &HeaderMap<HeaderValue>,
) -> Option<(PathBuf, Metadata, &str)> {
    let mut precompressed = None;

    tracing::trace!(
        "preparing pre-compressed file path variant of {}",
        file_path.display()
    );

    // Determine prefered-encoding extension if available
    let precomp_ext = match compression::get_prefered_encoding(headers) {
        // https://zlib.net/zlib_faq.html#faq39
        Some(ContentCoding::GZIP | ContentCoding::DEFLATE) => Some("gz"),
        // https://peazip.github.io/brotli-compressed-file-format.html
        Some(ContentCoding::BROTLI) => Some("br"),
        _ => None,
    };

    if precomp_ext.is_none() {
        tracing::trace!("preferred encoding based on the file extension was not determined");
    }

    // Try to find the pre-compressed metadata variant for the given file path
    if let Some(ext) = precomp_ext {
        let mut filepath_precomp = file_path;
        let filename = filepath_precomp.file_name().unwrap().to_str().unwrap();
        let precomp_file_name = [filename, ".", ext].concat();
        filepath_precomp.set_file_name(precomp_file_name);

        tracing::trace!(
            "getting metadata for pre-compressed file variant {}",
            filepath_precomp.display()
        );

        if let Ok((meta, _)) = file_metadata(&filepath_precomp) {
            tracing::trace!("pre-compressed file variant found, serving it directly");

            let encoding = if ext == "gz" { "gzip" } else { ext };
            precompressed = Some((filepath_precomp, meta, encoding));
        }

        // Note: In error case like "no such file or dir" the workflow just continues
    }

    precompressed
}
