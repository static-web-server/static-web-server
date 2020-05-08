use flate2::write::GzEncoder;
use flate2::Compression;
use iron::headers::{AcceptEncoding, ContentEncoding, ContentType, Encoding};
use iron::prelude::*;
use iron::AfterMiddleware;
use iron_staticfile_middleware::helpers;

pub struct GzipMiddleware;

impl AfterMiddleware for GzipMiddleware {
    fn after(&self, req: &mut Request, mut resp: Response) -> IronResult<Response> {
        // Skip Gzip response on HEAD requests
        if req.method == iron::method::Head {
            return Ok(resp);
        }

        // Enable Gzip compression only for known text-based file types
        let enable_gz = helpers::is_text_mime_type(resp.headers.get::<ContentType>());
        let accept_gz = helpers::accept_gzip(req.headers.get::<AcceptEncoding>());

        if enable_gz && accept_gz {
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
        }

        Ok(resp)
    }
}
