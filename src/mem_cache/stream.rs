use bytes::{Bytes, BytesMut};
use futures_util::Stream;
use std::io::{self, Read};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use crate::mem_cache::cache::{CACHE_STORE, MemFile, MemFileTempOpts};

/// A streaming file reader that, on successful completion, inserts the
/// fully-read bytes into the global in-memory cache store.
///
/// Only use this stream for **full-file** responses (no `Range` header).
/// Range requests must use the plain [`crate::fs::stream::FileStream`] to
/// avoid wasted allocations and to keep partial content out of the cache.
#[derive(Debug)]
pub(crate) struct MemCacheFileStream<T> {
    reader: T,
    buf_size: usize,
    /// Expected total byte length of the file. The cache is populated only
    /// when the accumulated buffer length matches this value, which guards
    /// against truncated reads (e.g. the file being modified mid-stream).
    file_size: usize,
    /// Metadata to attach to the cache entry. Taken on completion.
    mem_opts: Option<MemFileTempOpts>,
    /// Accumulator for the full file body. Taken on completion.
    mem_buf: Option<BytesMut>,
    /// Scratch read buffer reused on every poll.
    buf: BytesMut,
}

impl<T> MemCacheFileStream<T> {
    pub(crate) fn new(
        reader: T,
        buf_size: usize,
        mem_opts: MemFileTempOpts,
        file_size: usize,
    ) -> Self {
        Self {
            reader,
            buf_size,
            file_size,
            mem_opts: Some(mem_opts),
            mem_buf: Some(BytesMut::with_capacity(file_size)),
            buf: BytesMut::with_capacity(buf_size),
        }
    }
}

impl<T: Read + Unpin> Stream for MemCacheFileStream<T> {
    type Item = io::Result<Bytes>;

    fn poll_next(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = Pin::into_inner(self);
        this.buf.resize(this.buf_size, 0);

        match this.reader.read(&mut this.buf[..]) {
            Ok(0) => {
                // Stream complete. Insert into the cache store only if we
                // accumulated exactly `file_size` bytes; a smaller buffer
                // indicates a truncated read (e.g. the file was modified or
                // unlinked mid-stream) and must not be cached.
                if let (Some(mem_opts), Some(mem_buf)) = (this.mem_opts.take(), this.mem_buf.take())
                    && mem_buf.len() == this.file_size
                {
                    let file_path = mem_opts.file_path.as_str();
                    tracing::debug!("file `{file_path}` inserted into in-memory cache store");
                    let mem_file = Arc::new(MemFile::new(
                        mem_buf.freeze(),
                        this.buf_size,
                        mem_opts.content_type,
                        mem_opts.last_modified,
                    ));
                    if let Some(store) = CACHE_STORE.get() {
                        store.insert(file_path.into(), mem_file);
                    }
                }
                Poll::Ready(None)
            }
            Ok(n) => {
                let buf = this.buf.split_to(n).freeze();
                if let Some(mem_buf) = this.mem_buf.as_mut() {
                    mem_buf.extend_from_slice(&buf);
                }
                Poll::Ready(Some(Ok(buf)))
            }
            Err(err) => Poll::Ready(Some(Err(err))),
        }
    }
}
