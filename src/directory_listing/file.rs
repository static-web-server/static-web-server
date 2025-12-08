// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

use chrono::{DateTime, Local, Utc};
use serde::{Serialize, Serializer};
use std::ffi::{OsStr, OsString};

pub(crate) const DATETIME_FORMAT_UTC: &str = "%FT%TZ";
pub(crate) const DATETIME_FORMAT_LOCAL: &str = "%F %T";

#[derive(Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum FileType {
    Directory,
    File,
}

/// Defines a file entry and its properties.
#[derive(Serialize)]
pub(crate) struct FileEntry {
    #[serde(serialize_with = "serialize_name")]
    pub(crate) name: OsString,
    #[serde(serialize_with = "serialize_mtime")]
    pub(crate) mtime: Option<DateTime<Local>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) size: Option<u64>,
    pub(crate) r#type: FileType,
    #[serde(skip_serializing)]
    pub(crate) uri: String,
}

impl FileEntry {
    pub(crate) fn is_dir(&self) -> bool {
        self.r#type == FileType::Directory
    }
}

/// Serialize FileEntry::name
fn serialize_name<S: Serializer>(name: &OsStr, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&name.to_string_lossy())
}

/// Serialize FileEntry::mtime field
fn serialize_mtime<S: Serializer>(
    mtime: &Option<DateTime<Local>>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    match mtime {
        Some(dt) => serializer.serialize_str(
            &dt.with_timezone(&Utc)
                .format(DATETIME_FORMAT_UTC)
                .to_string(),
        ),
        None => serializer.serialize_str(""),
    }
}
