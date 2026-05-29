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
                // Safety net: try to insert if we somehow reached EOF with a
                // complete buffer but never tripped the eager-insert branch
                // below (e.g. a zero-byte file: one `Ok(0)` poll, no `Ok(n)`).
                try_insert(&mut this.mem_opts, &mut this.mem_buf, this.file_size);
                Poll::Ready(None)
            }
            Ok(n) => {
                let buf = this.buf.split_to(n).freeze();
                if let Some(mem_buf) = this.mem_buf.as_mut() {
                    mem_buf.extend_from_slice(&buf);
                    // Insert into the cache eagerly on the last chunk.
                    //
                    // We cannot rely on a subsequent `Ok(0)` poll because
                    // hyper does not always drive the body stream to
                    // completion: once `Content-Length` bytes have been
                    // emitted on the wire, the HTTP/1.1 writer may finalize
                    // the response without polling the body again. Without
                    // this eager check the cache would never be populated
                    // for full-file responses served over real HTTP traffic.
                    if mem_buf.len() >= this.file_size {
                        try_insert(&mut this.mem_opts, &mut this.mem_buf, this.file_size);
                    }
                }
                Poll::Ready(Some(Ok(buf)))
            }
            Err(err) => Poll::Ready(Some(Err(err))),
        }
    }
}

/// Move the accumulated body and metadata out of the stream and insert them
/// into the global cache store, but only if exactly `file_size` bytes were
/// accumulated. A shorter buffer indicates a truncated read (e.g. the file
/// was modified or unlinked mid-stream) and must not be cached.
///
/// Calling this with already-taken `Option`s is a no-op, so it is safe to
/// call multiple times.
fn try_insert(
    mem_opts: &mut Option<MemFileTempOpts>,
    mem_buf: &mut Option<BytesMut>,
    file_size: usize,
) {
    let Some(buf) = mem_buf.as_ref() else { return };
    if buf.len() != file_size {
        return;
    }
    // Both `Option`s are `Some` and the length matches: take and insert.
    let (Some(opts), Some(buf)) = (mem_opts.take(), mem_buf.take()) else {
        return;
    };
    let file_path = opts.file_path.as_str();
    tracing::debug!("file `{file_path}` inserted into in-memory cache store");
    let mem_file = Arc::new(MemFile::new(
        buf.freeze(),
        opts.content_type,
        opts.last_modified,
        opts.etag,
    ));
    if let Some(store) = CACHE_STORE.get() {
        store.insert(file_path.into(), mem_file);
    }
}
