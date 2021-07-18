use std::net::TcpListener;
use tera::{Context, Tera};

use actix_web::{
    get, http::header::ContentType, middleware, web, App, HttpResponse, HttpServer, ResponseError,
};

use crate::config::SharedConfig;

#[derive(thiserror::Error, Debug)]
enum ManagementError {
    #[error("Error rendering template")]
    TemplateError(#[from] tera::Error),
}

impl ResponseError for ManagementError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
    }

    fn error_response(&self) -> actix_web::HttpResponse {
        crate::error::response(self, self.status_code())
    }
}

pub async fn run(config: SharedConfig, listener: TcpListener) -> anyhow::Result<()> {
    let config = web::Data::new(config);
    let mut tera = Tera::default();

    tera.add_raw_template(
        "index.html",
        include_str!("../templates/management/index.html"),
    )?;

    let tera = web::Data::new(tera);

    HttpServer::new(move || {
        App::new()
            .app_data(config.clone())
            .app_data(tera.clone())
            .wrap(middleware::Logger::default())
            .service(index)
    })
    .listen(listener)?
    .run()
    .await?;

    Ok(())
}

#[get("/")]
async fn index(
    tera: web::Data<Tera>,
    config: web::Data<SharedConfig>,
) -> Result<HttpResponse, ManagementError> {
    let config = config.read().unwrap();

    let services = config
        .services
        .iter()
        .map(|s| s.host_name.clone())
        .collect::<Vec<_>>();

    let mut context = Context::new();
    context.insert("services", &services);
    context.insert("listen_port", &config.listen_port);

    let body = tera.render("index.html", &context)?;

    let mut builder = HttpResponse::Ok();
    builder.insert_header(ContentType::html());

    Ok(builder.body(body))
}
