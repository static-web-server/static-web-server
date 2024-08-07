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

                    if pinned.mem_opts.is_some() {
                        // TODO: add error handling
                        let tmp_buf = pinned.mem_buf.as_mut().unwrap();
                        tmp_buf.put(buf.clone());

                        if tmp_buf.len() == tmp_buf.capacity() {
                            let tmp_data_owned = pinned.mem_buf.take().unwrap();
                            let tmp_data = tmp_data_owned.freeze();
                            let mem_file_opts = pinned.mem_opts.clone().unwrap();

                            let mem_file = Arc::new(MemFile::new(
                                tmp_data,
                                buf_size,
                                mem_file_opts.content_type,
                                mem_file_opts.last_modified,
                                mem_file_opts.file_ttl,
                            ));

                            tracing::debug!(
                                "file `{}` is inserted to in-memory cache store: {:?}",
                                mem_file_opts.file_path,
                                mem_file
                            );
                            CACHE_STORE.insert(mem_file_opts.file_path.into(), mem_file);
                        }
                    }

                    Poll::Ready(Some(Ok(buf)))
                }
            }
            Err(err) => Poll::Ready(Some(Err(anyhow::Error::from(err)))),
        }
    }
}
