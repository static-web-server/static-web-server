extern crate iron;

use iron::mime;
use iron::prelude::*;
use iron::status;
use iron::AfterMiddleware;

pub struct ErrorPage;

impl AfterMiddleware for ErrorPage {
    fn after(&self, _: &mut Request, res: Response) -> IronResult<Response> {
        let content_type = "text/html".parse::<mime::Mime>().unwrap();

        match res.status {
            Some(status::NotFound) => Ok(Response::with((
                content_type,
                status::NotFound,
                "404 Not Found",
            ))),
            Some(status::InternalServerError) => Ok(Response::with((
                content_type,
                status::InternalServerError,
                "50x Internal Server Error",
            ))),
            _ => Ok(Response::with((
                content_type,
                status::ServiceUnavailable,
                "503 Service Unavailable",
            ))),
        }
    }
}
