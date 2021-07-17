use actix_web::{
    error::PayloadError,
    http::{uri::InvalidUri, Uri},
    middleware, web, App, HttpRequest, HttpResponse, HttpServer, ResponseError,
};
use awc::{error::SendRequestError, Client};
use tracing::error;

use crate::config::Config;

#[derive(thiserror::Error, Debug)]
enum ServerError {
    #[error("failed to write HTTP response body")]
    PayloadError(#[from] PayloadError),

    #[error("failed to send HTTP request to upstream")]
    SendRequestError(#[from] SendRequestError),

    #[error("failed to construct proxy URI")]
    UriConstructionError(#[from] InvalidUri),

    #[error("request missing host header")]
    MissingHost,

    #[error("got request for unknown host: {0}")]
    UnknownHost(String),
}

impl ResponseError for ServerError {}

pub async fn run(config: Config) -> anyhow::Result<()> {
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(Client::new()))
            .app_data(web::Data::new(crate::config::load()))
            .wrap(middleware::Logger::default())
            .default_service(web::route().to(forward))
    })
    .bind(format!("{}:{}", config.listen_host, config.listen_port))?
    .run()
    .await?;

    Ok(())
}

fn find_upstream_addr(req: &HttpRequest, config: web::Data<Config>) -> Result<String, ServerError> {
    let host = String::from_utf8_lossy(
        req.headers()
            .get("host")
            .ok_or(ServerError::MissingHost)?
            .as_ref(),
    );

    for svc in &config.services {
        if host.starts_with(&svc.host_name) {
            return Ok(svc.upstream_address.clone());
        }
    }

    Err(ServerError::UnknownHost(host.into_owned()))
}

async fn forward(
    req: HttpRequest,
    body: web::Bytes,
    client: web::Data<Client>,
    config: web::Data<Config>,
) -> Result<HttpResponse, ServerError> {
    let mut uri = find_upstream_addr(&req, config)?;

    if let Some(path_and_query) = req.uri().path_and_query() {
        uri.push_str(path_and_query.as_str());
    }

    let new_uri: Uri = uri.parse()?;

    // TODO: This forwarded implementation is incomplete as it only handles the inofficial
    // X-Forwarded-For header but not the official Forwarded one.
    let forwarded_req = client.request_from(new_uri, req.head()).no_decompress();
    let forwarded_req = if let Some(addr) = req.head().peer_addr {
        forwarded_req.append_header(("x-forwarded-for", format!("{}", addr.ip())))
    } else {
        forwarded_req
    };

    let mut res = forwarded_req.send_body(body).await?;

    let mut client_resp = HttpResponse::build(res.status());
    // Remove `Connection` as per
    // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Connection#Directives
    for (header_name, header_value) in res
        .headers()
        .into_iter()
        .filter(|(h, _)| *h != "connection")
    {
        client_resp.append_header((header_name.clone(), header_value.clone()));
    }

    Ok(client_resp.body(res.body().await?))
}
