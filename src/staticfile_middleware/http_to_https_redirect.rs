use iron::prelude::*;
use iron::{self, status};

#[derive(Debug)]
pub struct HttpToHttpsRedirect {
    permanent: bool,
    host: String,
    port: u16,
}

impl HttpToHttpsRedirect {
    pub fn new(host: &str, port: u16) -> Self {
        HttpToHttpsRedirect {
            permanent: false,
            host: host.into(),
            port,
        }
    }

    pub fn temporary(self) -> Self {
        HttpToHttpsRedirect {
            permanent: false,
            ..self
        }
    }

    pub fn permanent(self) -> Self {
        HttpToHttpsRedirect {
            permanent: true,
            ..self
        }
    }
}

impl iron::Handler for HttpToHttpsRedirect {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let mut url: url::Url = req.url.clone().into();

        url.set_scheme("https")
            .expect("Unable to rewrite URL scheme");
        url.set_host(Some(&self.host))
            .expect("Unable to rewrite URL host");
        url.set_port(Some(self.port))
            .expect("Unable to rewrite URL port");

        let url = iron::Url::from_generic_url(url).expect("Unable to rewrite HTTP URL to HTTPS");

        let status = if self.permanent {
            status::PermanentRedirect
        } else {
            status::TemporaryRedirect
        };

        Ok(Response::with((status, iron::modifiers::Redirect(url))))
    }
}
