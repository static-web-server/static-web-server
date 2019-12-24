use crate::error_page::ErrorPage;
use crate::gzip::GzipMiddleware;
use crate::logger::Logger;

use iron::prelude::*;
use iron_staticfile_middleware::{Cache, GuessContentType, ModifyWith, Prefix, Staticfile};
use std::time::Duration;

/// An Iron middleware for static files-serving.
pub struct StaticFiles {
    opts: StaticFilesOptions,
}

pub struct StaticFilesOptions {
    pub root_dir: String,
    pub assets_dir: String,
    pub page_50x_path: String,
    pub page_404_path: String,
}

impl StaticFiles {
    /// Create a new instance of `StaticFiles` with given options.
    pub fn new(opts: StaticFilesOptions) -> StaticFiles {
        StaticFiles { opts }
    }

    /// Handle static files for current `StaticFiles` middleware.
    pub fn handle(&self) -> Chain {
        let mut chain = Chain::new(
            Staticfile::new(self.opts.root_dir.as_str())
                .expect("Directory to serve files was not found"),
        );
        let one_day = Duration::new(60 * 60 * 24, 0);
        let one_year = Duration::new(60 * 60 * 24 * 365, 0);
        let default_content_type = "text/html"
            .parse()
            .expect("Unable to create a default content type header");

        chain.link_after(ModifyWith::new(Cache::new(one_day)));
        chain.link_after(Prefix::new(
            &[self.opts.assets_dir.as_str()],
            Cache::new(one_year),
        ));
        chain.link_after(GuessContentType::new(default_content_type));
        chain.link_after(GzipMiddleware);
        chain.link_after(Logger);
        chain.link_after(ErrorPage::new(
            self.opts.page_50x_path.as_str(),
            self.opts.page_404_path.as_str(),
        ));
        chain
    }
}
