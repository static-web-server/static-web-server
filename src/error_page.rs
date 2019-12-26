extern crate iron;

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
        let page404: String;
        let page50x: String;

        if Path::new(&page_404_path.as_ref()).exists() {
            page404 = fs::read_to_string(page_404_path).unwrap();
        } else {
            page404 = String::from("<h2>404</h2><p>Content could not found</p>");
        }

        if Path::new(&page_50x_path.as_ref()).exists() {
            page50x = fs::read_to_string(page_50x_path).unwrap();
        } else {
            page50x = String::from(
                "<h2>50x</h2><p>Service is temporarily unavailable due an unexpected error</p>",
            );
        }

        ErrorPage { page404, page50x }
    }
}

impl AfterMiddleware for ErrorPage {
    fn after(&self, _: &mut Request, res: Response) -> IronResult<Response> {
        let content_type = "text/html".parse::<mime::Mime>().unwrap();

        match res.status {
            Some(status::NotFound) => Ok(Response::with((
                content_type,
                status::NotFound,
                self.page404.as_str(),
            ))),
            Some(status::InternalServerError) => Ok(Response::with((
                content_type,
                status::InternalServerError,
                self.page50x.as_str(),
            ))),
            Some(status::BadGateway) => Ok(Response::with((
                content_type,
                status::BadGateway,
                self.page50x.as_str(),
            ))),
            Some(status::ServiceUnavailable) => Ok(Response::with((
                content_type,
                status::ServiceUnavailable,
                self.page50x.as_str(),
            ))),
            Some(status::GatewayTimeout) => Ok(Response::with((
                content_type,
                status::GatewayTimeout,
                self.page50x.as_str(),
            ))),
            _ => Ok(res),
        }
    }
}
