use tracing::{debug, error};

mod config;
mod http;
mod mdns;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let config = crate::config::load();

    tracing_subscriber::fmt::Subscriber::builder()
        .with_env_filter(&config.log)
        .init();

    debug!(?config, "loaded config");

    mdns::advertise(&config)?;

    if let Err(error) = http::run(config).await {
        let e: &dyn std::error::Error = &*error;
        error!(error = e, "Error running server");
    }

    Ok(())
}
