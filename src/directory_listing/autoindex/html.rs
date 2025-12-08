// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

use percent_encoding::percent_decode_str;

use crate::directory_listing::file::{DATETIME_FORMAT_LOCAL, FileEntry};
use crate::directory_listing::sort::sort_file_entries;
use crate::directory_listing::style::STYLES;

#[cfg(feature = "directory-listing-download")]
use crate::directory_listing_download::{DOWNLOAD_PARAM_KEY, DirDownloadFmt};

/// Create an auto index in HTML format.
pub(crate) fn html_auto_index<'a>(
    base_path: &'a str,
    dirs_count: usize,
    files_count: usize,
    entries: &'a mut [FileEntry],
    order_code: u8,
    #[cfg(feature = "directory-listing-download")] download: &'a [DirDownloadFmt],
) -> String {
    use maud::{DOCTYPE, html};

    let sort_attrs = sort_file_entries(entries, order_code);
    let current_path = percent_decode_str(base_path).decode_utf8_lossy();

    #[cfg(feature = "directory-listing-download")]
    let download_directory_elem = match download.is_empty() {
        true => html! {},
        false => html! {
            ", " a href={ "?" (DOWNLOAD_PARAM_KEY) } {
                "download tar.gz"
            }
        },
    };
    #[cfg(not(feature = "directory-listing-download"))]
    let download_directory_elem = html! {};

    let styles = STYLES.replace("\n", "").replace("  ", "");
    html! {
        (DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width,minimum-scale=1,initial-scale=1";
                title {
                    "Index of " (current_path)
                }
                style {
                    (styles)
                }
            }
            body {
                h1 {
                    "Index of " (current_path)
                }
                p {
                    small {
                        "directories: " (dirs_count) ", files: " (files_count) (download_directory_elem)
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
                                    a href=(entry.uri) {
                                        (entry.name.to_string_lossy())
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
                                    (entry.size.map(format_file_size).unwrap_or("-".into()))
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
        }
    }.into()
}

/// Formats the file size in bytes to a human-readable string
fn format_file_size(size: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB"];
    let mut size_tmp = size;

    if size_tmp < 1024 {
        // return the size with Byte
        return format!("{} {}", size_tmp, UNITS[0]);
    }

    for unit in &UNITS[1..UNITS.len() - 1] {
        if size_tmp < 1024 * 1024 {
            // return the size divided by 1024 with the unit
            return format!("{:.2} {}", size_tmp as f64 / 1024.0, unit);
        }
        size_tmp >>= 10;
    }

    // if size is too large, return the largest unit
    format!("{:.2} {}", size_tmp as f64 / 1024.0, UNITS[UNITS.len() - 1])
}

#[cfg(test)]
mod tests {
    use super::format_file_size;

    #[test]
    fn handle_byte() {
        let size = 128;
        assert_eq!("128 B", format_file_size(size))
    }

    #[test]
    fn handle_kibibyte() {
        let size = 1024;
        assert_eq!("1.00 KiB", format_file_size(size))
    }

    #[test]
    fn handle_mebibyte() {
        let size = 1048576;
        assert_eq!("1.00 MiB", format_file_size(size))
    }

    #[test]
    fn handle_gibibyte() {
        let size = 1073741824;
        assert_eq!("1.00 GiB", format_file_size(size))
    }

    #[test]
    fn handle_tebibyte() {
        let size = 1099511627776;
        assert_eq!("1.00 TiB", format_file_size(size))
    }

    #[test]
    fn handle_pebibyte() {
        let size = 1125899906842624;
        assert_eq!("1.00 PiB", format_file_size(size))
    }

    #[test]
    fn handle_large() {
        let size = u64::MAX;
        assert_eq!("16384.00 PiB", format_file_size(size))
    }
}
