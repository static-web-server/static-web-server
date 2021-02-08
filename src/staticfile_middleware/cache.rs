use std::time::Duration;
use std::{cmp, u32};

use iron::headers::{CacheControl, CacheDirective};
use iron::modifier::Modifier;
use iron::modifiers::Header;
use iron::prelude::*;
use iron::status;

/// Sets the Cache-Control header for successful responses.
#[derive(Debug, Copy, Clone)]
pub struct Cache(u32);

impl Cache {
    pub fn new(duration: Duration) -> Cache {
        // Capping the value at ~136 years!
        let duration = cmp::min(duration.as_secs(), u32::MAX as u64) as u32;

        Cache(duration)
    }
}

impl Modifier<Response> for Cache {
    fn modify(self, response: &mut Response) {
        match response.status {
            Some(status::Ok) | Some(status::NotModified) => (),
            _ => return,
        }

        Header(CacheControl(vec![
            CacheDirective::Public,
            CacheDirective::MaxAge(self.0),
        ]))
        .modify(response)
    }
}
