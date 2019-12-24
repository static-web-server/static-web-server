extern crate iron;

use iron::mime;
use iron::prelude::*;
use iron::status;
use iron::AfterMiddleware;
use std::fs;
use std::path::Path;

/// Custom Error pages middleware for Iron
pub struct ErrorPage {
    /// HTML file content for 50x errors.
    pub page50x: std::string::String,
    /// HTML file content for 404 errors.
    pub page404: std::string::String,
}

impl ErrorPage {
    /// Create a new instance of `ErrorPage` middleware with a given html pages.
    pub fn new<P: AsRef<Path>>(page_50x_path: P, page_404_path: P) -> ErrorPage {
        let page50x = fs::read_to_string(page_50x_path).unwrap();
        let page404 = fs::read_to_string(page_404_path).unwrap();

        ErrorPage { page50x, page404 }
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
