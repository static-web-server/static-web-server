use flate2::write::GzEncoder;
use flate2::Compression;
use iron::headers::{ContentEncoding, Encoding};
use iron::prelude::*;
use iron::AfterMiddleware;

pub struct GzipMiddleware;

impl AfterMiddleware for GzipMiddleware {
    fn after(&self, _: &mut Request, mut resp: Response) -> IronResult<Response> {
        let compressed_bytes = resp.body.as_mut().map(|b| {
            let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
            {
                let _ = b.write_body(&mut encoder);
            }
            encoder.finish().unwrap()
        });

        if let Some(b) = compressed_bytes {
            resp.headers.set(ContentEncoding(vec![Encoding::Gzip]));
            resp.set_mut(b);
        }

        Ok(resp)
    }
}
