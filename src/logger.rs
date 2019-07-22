use iron::prelude::*;
use iron::AfterMiddleware;

pub struct Logger;

impl AfterMiddleware for Logger {
    fn after(&self, req: &mut Request, res: Response) -> IronResult<Response> {
        info!(
            "Request [{}] {} - {}",
            req.method,
            res.status
                .into_iter()
                .map(|i| i.to_string())
                .collect::<String>(),
            req.url
                .path()
                .into_iter()
                .map(|i| format!("/{}", i))
                .collect::<String>()
        );

        Ok(res)
    }
}
