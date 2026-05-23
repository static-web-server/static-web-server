use bytes::{Bytes, BytesMut};
use futures_util::Stream;
use std::io::{self, Read};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use crate::mem_cache::cache::{CACHE_STORE, MemFile, MemFileTempOpts};

#[derive(Debug)]
pub(crate) struct MemCacheFileStream<T> {
    pub(crate) reader: T,
    pub(crate) buf_size: usize,
    pub(crate) mem_opts: Option<MemFileTempOpts>,
    pub(crate) mem_buf: Option<BytesMut>,
    pub(crate) buf: BytesMut,
}

impl<T> MemCacheFileStream<T> {
    pub(crate) fn new(
        reader: T,
        buf_size: usize,
        mem_opts: Option<MemFileTempOpts>,
        mem_buf: Option<BytesMut>,
    ) -> Self {
        Self {
            reader,
            buf_size,
            mem_opts,
            mem_buf,
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
            Ok(0) => Poll::Ready(None),
            Ok(n) => {
                let buf = this.buf.split_to(n).freeze();

                // Handle in-memory cache if enabled
                if let (Some(mem_opts), Some(buf_data_mut)) =
                    (this.mem_opts.as_ref(), this.mem_buf.as_mut())
                {
                    buf_data_mut.extend_from_slice(&buf);

                    // If file size is reached then proceed cache it
                    if buf_data_mut.len() == buf_data_mut.capacity() {
                        let buf_data = this.mem_buf.take().unwrap().freeze();

                        let mem_file = Arc::new(MemFile::new(
                            buf_data,
                            this.buf_size,
                            mem_opts.content_type.to_owned(),
                            mem_opts.last_modified,
                        ));

                        let file_path = mem_opts.file_path.as_str();
                        tracing::debug!(
                            "file `{}` is inserted to the in-memory cache store",
                            file_path
                        );

                        if let Some(store) = CACHE_STORE.get() {
                            store.insert(file_path.into(), mem_file);
                        }
                    }
                }

                Poll::Ready(Some(Ok(buf)))
            }
            Err(err) => Poll::Ready(Some(Err(err))),
        }
    }
}
