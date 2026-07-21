// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Errors that can occur while loading a plugin.

use std::fmt;
use std::path::PathBuf;

/// Errors that can occur while loading a plugin.
#[derive(Debug)]
pub enum LoadError {
    /// The shared library could not be opened.
    Open(PathBuf, libloading::Error),
    /// The entry symbol could not be resolved.
    Symbol(PathBuf, libloading::Error),
    /// The entry symbol returned a null descriptor.
    NullDecl(PathBuf),
    /// The plugin was built against a different ABI version.
    AbiMismatch {
        /// Plugin path.
        path: PathBuf,
        /// ABI version the host expects.
        expected: u32,
        /// ABI version the plugin declared.
        found: u32,
    },
    /// The plugin's constructor returned a null instance.
    InitFailed(String),
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Open(path, err) => {
                write!(f, "cannot open plugin library {}: {err}", path.display())
            }
            Self::Symbol(path, err) => write!(
                f,
                "plugin {} does not export `sws_plugin_declare`: {err}",
                path.display()
            ),
            Self::NullDecl(path) => {
                write!(f, "plugin {} returned a null descriptor", path.display())
            }
            Self::AbiMismatch {
                path,
                expected,
                found,
            } => write!(
                f,
                "plugin {} ABI version {found} is incompatible with host ABI version {expected}",
                path.display()
            ),
            Self::InitFailed(name) => {
                write!(f, "plugin '{name}' failed to initialize")
            }
        }
    }
}

impl std::error::Error for LoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Open(_, err) | Self::Symbol(_, err) => Some(err),
            _ => None,
        }
    }
}
