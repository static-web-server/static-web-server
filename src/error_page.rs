use iron::mime;
use iron::prelude::*;
use iron::status;
use iron::AfterMiddleware;
use std::fs;
use std::path::Path;

/// Custom Error pages middleware for Iron
pub struct ErrorPage {
    /// HTML file content for 404 errors.
    pub page404: String,
    /// HTML file content for 50x errors.
    pub page50x: String,
}

impl ErrorPage {
    /// Create a new instance of `ErrorPage` middleware with a given html pages.
    pub fn new<P: AsRef<Path>>(page_404_path: P, page_50x_path: P) -> ErrorPage {
        let page404 = if Path::new(&page_404_path.as_ref()).exists() {
            fs::read_to_string(page_404_path).unwrap()
        } else {
            String::new()
        };

        let page50x = if Path::new(&page_50x_path.as_ref()).exists() {
            fs::read_to_string(page_50x_path).unwrap()
        } else {
            String::new()
        };

        ErrorPage { page404, page50x }
    }
}

impl AfterMiddleware for ErrorPage {
    fn after(&self, req: &mut Request<'_, '_>, mut resp: Response) -> IronResult<Response> {
        let content_type = "text/html"
            .parse::<mime::Mime>()
            .expect("Unable to create a default content type header");

        // Check for 4xx and 50x status codes
        let mut no_status_error = false;
        if let Some(stat) = resp.status {
            resp = match stat {
                // 4xx
                status::BadRequest
                | status::Unauthorized
                | status::PaymentRequired
                | status::Forbidden
                | status::NotFound
                | status::MethodNotAllowed
                | status::NotAcceptable
                | status::ProxyAuthenticationRequired
                | status::RequestTimeout
                | status::Conflict
                | status::Gone
                | status::LengthRequired
                | status::PreconditionFailed
                | status::PayloadTooLarge
                | status::UriTooLong
                | status::UnsupportedMediaType
                | status::RangeNotSatisfiable
                | status::ExpectationFailed => {
                    let mut content = String::new();

                    // Extra check for 404 status code and content
                    if stat == status::NotFound && !self.page404.is_empty() {
                        content = self.page404.clone()
                    }

                    if content.is_empty() {
                        content = format!(
                            "<html><head><title>{}</title></head><body><center><h1>{}</h1></center></body></html>",
                            stat.to_string(), stat.to_string());
                    }
                    Response::with((content_type, stat, content))
                }
                // 50x
                status::InternalServerError
                | status::NotImplemented
                | status::BadGateway
                | status::ServiceUnavailable
                | status::GatewayTimeout
                | status::HttpVersionNotSupported
                | status::VariantAlsoNegotiates
                | status::InsufficientStorage
                | status::LoopDetected => {
                    let content = if self.page50x.is_empty() {
                        format!(
                            "<html><head><title>{}</title></head><body><center><h1>{}</h1></center></body></html>",
                            stat.to_string(), stat.to_string()
                        )
                    } else {
                        self.page50x.clone()
                    };
                    Response::with((content_type, stat, content))
                }
                // other status codes like 200, etc
                _ => {
                    no_status_error = true;
                    resp
                }
            };
        }

        // Empty response body only on HEAD requests and status error (404,50x)
        if req.method == iron::method::Head && !no_status_error {
            resp.set_mut(vec![]);
        }

        Ok(resp)
    }
}
