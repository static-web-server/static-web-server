use iron::prelude::*;

pub struct Rewrite {
    from: Vec<Vec<String>>,
    to: String,
}

impl Rewrite {
    pub fn new(from_paths: Vec<Vec<String>>, to_path: String) -> Self {
        Rewrite {
            from: from_paths,
            to: to_path,
        }
    }
}

impl iron::BeforeMiddleware for Rewrite {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        let should_rewrite = {
            let request_path = req.url.path();
            self.from
                .iter()
                .any(|rewrite_path| request_path == *rewrite_path)
        };

        if should_rewrite {
            let mut u: url::Url = req.url.clone().into();
            u.set_path(&self.to);
            req.url = iron::Url::from_generic_url(u).expect("Invalid rewritten URL");
        }

        Ok(())
    }
}
