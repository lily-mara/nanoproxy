use std::sync::{Arc, RwLock};

use crate::config::{Config, Management, Service, SharedConfig};
use actix_http::body::AnyBody;
use actix_web::{
    dev::{Service as _, ServiceResponse},
    test::TestRequest,
    web::{self, Bytes},
    App,
};
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

struct TestBuilder {
    config: Config,
}

struct Test<A> {
    #[allow(dead_code)]
    config: SharedConfig,
    app: A,
}

impl TestBuilder {
    fn new() -> Self {
        let _ = tracing_subscriber::fmt::try_init();

        Self {
            config: Config {
                listen_port: 7432,
                listen_host: "localhost".into(),
                log: "info".into(),
                management: Management {
                    enabled: false,
                    host_name: "".into(),
                },
                services: vec![],
            },
        }
    }

    fn add_service(mut self, host_name: impl Into<String>, mock: &MockServer) -> Self {
        self.config.services.push(Service {
            host_name: host_name.into(),
            upstream_address: format!("{}", mock.uri()),
        });
        self
    }

    async fn build(
        self,
    ) -> Test<
        impl actix_web::dev::Service<
            actix_http::Request,
            Response = ServiceResponse<AnyBody>,
            Error = actix_web::Error,
        >,
    > {
        let config = Arc::new(RwLock::new(self.config));

        let app = actix_web::test::init_service(
            App::new().configure(crate::proxy::configure_app(web::Data::new(config.clone()))),
        )
        .await;

        Test { config, app }
    }
}

#[actix_web::rt::test]
async fn test_it_resolves_302() {
    let mock_server = MockServer::start().await;

    let test = TestBuilder::new()
        .add_service("it_resolves_302", &mock_server)
        .build()
        .await;

    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(302).append_header("Location", "/found"))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/found"))
        .respond_with(ResponseTemplate::new(200).set_body_string("you found me"))
        .mount(&mock_server)
        .await;

    let req = TestRequest::with_uri("/")
        .append_header(("host", "it_resolves_302.local"))
        .to_request();

    let resp = test.app.call(req).await.unwrap();

    let req = TestRequest::with_uri(resp.headers().get("location").unwrap().to_str().unwrap())
        .append_header(("host", "it_resolves_302.local"))
        .to_request();

    let resp = test.app.call(req).await.unwrap();

    let body = actix_web::test::load_body(resp.into_body()).await.unwrap();

    assert_eq!(body, Bytes::from("you found me"));
}
