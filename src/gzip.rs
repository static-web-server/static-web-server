use flate2::write::GzEncoder;
use flate2::Compression;
use iron::headers::{AcceptEncoding, ContentEncoding, ContentType, Encoding};
use iron::mime;
use iron::prelude::*;
use iron::AfterMiddleware;

pub struct GzipMiddleware;

const GZIP_TYPES: [&str; 16] = [
    "text/html",
    "text/css",
    "text/javascript",
    "text/xml",
    "text/plain",
    "text/x-component",
    "application/javascript",
    "application/x-javascript",
    "application/json",
    "application/xml",
    "application/rss+xml",
    "application/atom+xml",
    "font/truetype",
    "font/opentype",
    "application/vnd.ms-fontobject",
    "image/svg+xml",
];

impl AfterMiddleware for GzipMiddleware {
    fn after(&self, req: &mut Request, mut resp: Response) -> IronResult<Response> {
        // Skip Gzip response on HEAD requests
        if req.method == iron::method::Head {
            return Ok(resp);
        }

        // Enable Gzip compression only for known text-based file types
        let enable_gzip = match resp.headers.get::<ContentType>() {
            Some(content_type) => {
                let mut v = false;
                for e in &GZIP_TYPES {
                    if content_type.0 == e.parse::<mime::Mime>().unwrap() {
                        v = true;
                        break;
                    }
                }

                v
            }
            None => false,
        };

        let accept_gz = match req.headers.get::<AcceptEncoding>() {
            Some(accept) => accept.0.iter().any(|qi| qi.item == Encoding::Gzip),
            None => false,
        };

        if enable_gzip && accept_gz {
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
