use bytes::{BufMut, Bytes, BytesMut};
use futures_util::Stream;
use std::io::Read;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use crate::mem_cache::cache::{MemFile, MemFileTempOpts, CACHE_STORE};
use crate::Result;

#[derive(Debug)]
pub(crate) struct MemCacheFileStream<T> {
    pub(crate) reader: T,
    pub(crate) buf_size: usize,
    pub(crate) mem_opts: Option<MemFileTempOpts>,
    pub(crate) mem_buf: Option<BytesMut>,
}

impl<T: Read + Unpin> Stream for MemCacheFileStream<T> {
    type Item = Result<Bytes>;

    fn poll_next(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let buf_size = self.buf_size;
        let mut buf = BytesMut::zeroed(buf_size);
        let pinned = Pin::into_inner(self);

        match pinned.reader.read(&mut buf[..]) {
            Ok(n) => {
                if n == 0 {
                    Poll::Ready(None)
                } else {
                    buf.truncate(n);
                    let buf = buf.freeze();

                    // Handle in-memory cache if enabled
                    if pinned.mem_opts.is_some() && pinned.mem_buf.is_some() {
                        let buf_data_mut = pinned.mem_buf.as_mut().unwrap();
                        buf_data_mut.put(buf.clone());

                        // If file size is reached then proceed cache it
                        if buf_data_mut.len() == buf_data_mut.capacity() {
                            let buf_data = pinned.mem_buf.take().unwrap().freeze();
                            let mem_file_opts = pinned.mem_opts.as_ref().unwrap();

                            let mem_file = Arc::new(MemFile::new(
                                buf_data,
                                buf_size,
                                mem_file_opts.content_type.to_owned(),
                                mem_file_opts.last_modified,
                            ));

                            let file_path = mem_file_opts.file_path.as_str();
                            tracing::debug!(
                                "file `{}` is inserted to the in-memory cache store",
                                file_path
                            );

                            CACHE_STORE
                                .get()
                                .unwrap()
                                .insert(file_path.into(), mem_file);
                        }
                    }

                    Poll::Ready(Some(Ok(buf)))
                }
            }
            Err(err) => Poll::Ready(Some(Err(anyhow::Error::from(err)))),
        }
    }
}
