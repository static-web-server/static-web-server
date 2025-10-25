#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(rust_2018_idioms)]
#![deny(dead_code)]

#[cfg(all(unix, feature = "experimental"))]
pub mod tests {
    use hyper::Request;
    use std::net::SocketAddr;

    use static_web_server::testing::fixtures::{
        REMOTE_ADDR, fixture_req_handler, fixture_req_handler_opts, fixture_settings,
    };

    #[tokio::test]
    async fn experimental_metrics_enabled() {
        let opts = fixture_settings("toml/experimental_metrics.toml");
        let req_handler_opts = fixture_req_handler_opts(opts.general, opts.advanced);
        let req_handler = fixture_req_handler(req_handler_opts);
        let remote_addr = Some(REMOTE_ADDR.parse::<SocketAddr>().unwrap());

        let mut req = Request::default();
        *req.method_mut() = hyper::Method::GET;
        *req.uri_mut() = "http://localhost/metrics".parse().unwrap();

        prometheus::default_registry()
            .register(Box::new(
                tokio_metrics_collector::default_runtime_collector(),
            ))
            .unwrap();

        match req_handler.handle(&mut req, remote_addr).await {
            Ok(res) => {
                assert_eq!(res.status(), 200);
                assert_eq!(res.headers()["content-type"], "text/plain; charset=utf-8");

                let body = hyper::body::to_bytes(res.into_body())
                    .await
                    .expect("unexpected bytes error during `body` conversion");
                let body_str = std::str::from_utf8(&body).unwrap();

                assert!(body_str.contains("tokio_budget_forced_yield_count 0"));
                assert!(body_str.contains("tokio_total_local_schedule_count 0"));
                assert!(body_str.contains("tokio_workers_count 1"));
            }
            Err(err) => {
                panic!("unexpected error: {err}")
            }
        };
    }
}
