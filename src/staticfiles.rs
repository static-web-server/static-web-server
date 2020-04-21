use crate::error_page::ErrorPage;
use crate::gzip::GzipMiddleware;
use crate::helpers;
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
        // Check the root directory
        let root_dir = match helpers::get_valid_dirpath(&self.opts.root_dir) {
            Err(e) => {
                error!("{}", e);
                std::process::exit(1)
            }
            Ok(v) => v,
        };

        // Check the assets directory
        let assets_dir = match helpers::get_valid_dirpath(&self.opts.assets_dir) {
            Err(e) => {
                error!("{}", e);
                std::process::exit(1)
            }
            Ok(v) => v,
        };

        // Get the assets directory name
        let assets_dirname = match helpers::get_dirname(&assets_dir) {
            Err(e) => {
                error!("{}", e);
                std::process::exit(1)
            }
            Ok(v) => v,
        };

        // Define middleware chain
        let mut chain = Chain::new(
            Staticfile::new(&root_dir, &assets_dir)
                .expect("Directory to serve files was not found"),
        );
        let one_day = Duration::new(60 * 60 * 24, 0);
        let one_year = Duration::new(60 * 60 * 24 * 365, 0);
        let default_content_type = "text/html"
            .parse()
            .expect("Unable to create a default content type header");

        chain.link_after(ModifyWith::new(Cache::new(one_day)));
        chain.link_after(Prefix::new(&[assets_dirname], Cache::new(one_year)));
        chain.link_after(GuessContentType::new(default_content_type));
        chain.link_after(GzipMiddleware);
        chain.link_after(Logger);
        chain.link_after(ErrorPage::new(
            &self.opts.page_404_path,
            &self.opts.page_50x_path,
        ));
        chain
    }
}
