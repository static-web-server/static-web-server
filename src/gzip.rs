use flate2::write::GzEncoder;
use flate2::Compression;
use iron::headers::{AcceptEncoding, ContentEncoding, ContentType, Encoding};
use iron::prelude::*;
use iron::AfterMiddleware;

use crate::staticfile_middleware::helpers;

pub struct GzipMiddleware;

impl AfterMiddleware for GzipMiddleware {
    fn after(&self, req: &mut Request, mut resp: Response) -> IronResult<Response> {
        // Skip Gzip compression for HEAD requests
        if req.method == iron::method::Head {
            return Ok(resp);
        }

        // Skip Gzip compression for non-text-based file types
        if !helpers::is_text_mime_type(resp.headers.get::<ContentType>()) {
            return Ok(resp);
        }

        // Skip Gzip compression is there is not gzip accept-encoding value
        if !helpers::accept_gzip(req.headers.get::<AcceptEncoding>()) {
            return Ok(resp);
        }

        let compressed_bytes = resp.body.as_mut().map(|b| {
            let mut encoder = GzEncoder::new(vec![], Compression::fast());
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
