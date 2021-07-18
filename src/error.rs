use actix_web::http::{
    header::{self},
    StatusCode,
};
use tracing::error;

pub fn response(
    e: &(dyn std::error::Error + 'static),
    status_code: StatusCode,
) -> actix_web::HttpResponse {
    error!(error = e, "Error handling request");

    let mut res = actix_web::HttpResponse::new(status_code);

    res.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("text/plain; charset=utf-8"),
    );

    let mut body = String::new();

    for c in anyhow::Chain::new(e) {
        body.push_str(&format!("{}\n", c));
    }

    res.set_body(body.into())
}
