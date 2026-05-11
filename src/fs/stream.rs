// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! A module that provides file stream functionality.
//!

use bytes::{Bytes, BytesMut};
use futures_util::Stream;
use std::fs::Metadata;
use std::io::{self, Read};
use std::pin::Pin;
use std::task::{Context, Poll};

#[cfg(unix)]
const DEFAULT_READ_BUF_SIZE: usize = 4_096;

#[cfg(not(unix))]
const DEFAULT_READ_BUF_SIZE: usize = 8_192;

#[derive(Debug)]
pub(crate) struct FileStream<T> {
    pub(crate) reader: T,
    pub(crate) buf_size: usize,
    buf: BytesMut,
}

impl<T> FileStream<T> {
    pub(crate) fn new(reader: T, buf_size: usize) -> Self {
        Self {
            reader,
            buf_size,
            buf: BytesMut::with_capacity(buf_size),
        }
    }
}

impl<T: Read + Unpin> Stream for FileStream<T> {
    type Item = io::Result<Bytes>;

    fn poll_next(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = Pin::into_inner(self);
        this.buf.resize(this.buf_size, 0);
        match this.reader.read(&mut this.buf[..]) {
            Ok(0) => Poll::Ready(None),
            Ok(n) => {
                let data = this.buf.split_to(n).freeze();
                Poll::Ready(Some(Ok(data)))
            }
            Err(err) => Poll::Ready(Some(Err(err))),
        }
    }
}

pub(crate) fn optimal_buf_size(metadata: &Metadata) -> usize {
    let block_size = get_block_size(metadata);
    // If file length is smaller than block size,
    // don't waste space reserving a bigger-than-needed buffer.
    std::cmp::min(block_size as u64, metadata.len()) as usize
}

#[cfg(unix)]
fn get_block_size(metadata: &Metadata) -> usize {
    use std::os::unix::fs::MetadataExt;
    // TODO: blksize() returns u64, should handle bad cast...
    // (really, a block size bigger than 4gb?)

    // Use device blocksize unless it's really small.
    std::cmp::max(metadata.blksize() as usize, DEFAULT_READ_BUF_SIZE)
}

#[cfg(not(unix))]
fn get_block_size(_metadata: &Metadata) -> usize {
    DEFAULT_READ_BUF_SIZE
}
