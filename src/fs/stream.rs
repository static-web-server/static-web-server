// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! A module that provides file stream functionality.
//!

use bytes::{BufMut, Bytes, BytesMut};
use futures_util::Stream;
use parking_lot::Mutex;
use std::fs::Metadata;
use std::io::Read;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::mem_cache::{MemFile, MemFileTempOpts, CACHE_STORE};
use crate::Result;

#[cfg(unix)]
const DEFAULT_READ_BUF_SIZE: usize = 4_096;

#[cfg(not(unix))]
const DEFAULT_READ_BUF_SIZE: usize = 8_192;

#[derive(Debug)]
pub(crate) struct FileStream<T> {
    pub(crate) reader: T,
    pub(crate) buf_size: usize,
    pub(crate) mem_file_data: Option<Mutex<Option<BytesMut>>>,
    pub(crate) mem_file_opts: Option<MemFileTempOpts>,
}

impl<T: Read + Unpin> Stream for FileStream<T> {
    type Item = Result<Bytes>;

    fn poll_next(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let buf_size = self.buf_size;
        let mut buf = BytesMut::zeroed(buf_size);
        let this = Pin::into_inner(self);

        match this.reader.read(&mut buf[..]) {
            Ok(n) => {
                if n == 0 {
                    Poll::Ready(None)
                } else {
                    buf.truncate(n);
                    let buf = buf.freeze();

                    if this.mem_file_opts.is_some() {
                        let mut guard = this.mem_file_data.as_ref().unwrap().lock();
                        let tmp_data = guard.as_mut().unwrap();
                        tmp_data.put(buf.clone());

                        if tmp_data.len() == tmp_data.capacity() {
                            let data = guard.take().unwrap().freeze();
                            let mem_file_opts = this.mem_file_opts.clone().unwrap();

                            let mem_file = MemFile::new(
                                data,
                                buf_size,
                                mem_file_opts.content_type,
                                mem_file_opts.last_modified,
                                mem_file_opts.file_ttl,
                            );

                            tracing::debug!(
                                "file `{}` is inserted to in-memory cache store: {:?}",
                                mem_file_opts.file_path,
                                mem_file
                            );
                            let mut cache_store = CACHE_STORE.get().unwrap().lock();
                            cache_store.insert(mem_file_opts.file_path, mem_file);
                        }
                    }

                    Poll::Ready(Some(Ok(buf)))
                }
            }
            Err(err) => Poll::Ready(Some(Err(anyhow::Error::from(err)))),
        }
    }
}

#[derive(Debug)]
pub(crate) struct FileStreamLite<T> {
    pub(crate) reader: T,
    pub(crate) buf_size: usize,
}

impl<T: Read + Unpin> Stream for FileStreamLite<T> {
    type Item = Result<Bytes>;

    fn poll_next(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut buf = BytesMut::zeroed(self.buf_size);
        match Pin::into_inner(self).reader.read(&mut buf[..]) {
            Ok(n) => {
                if n == 0 {
                    Poll::Ready(None)
                } else {
                    buf.truncate(n);
                    Poll::Ready(Some(Ok(buf.freeze())))
                }
            }
            Err(err) => Poll::Ready(Some(Err(anyhow::Error::from(err)))),
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
