use std::{
    net::TcpListener,
    sync::{Arc, RwLock},
};

use tracing::{debug, error, info};

use crate::config::Service;

mod config;
mod error;
mod management;
mod mdns;
mod proxy;

#[cfg(test)]
mod tests;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let config = crate::config::load();

    tracing_subscriber::fmt::Subscriber::builder()
        .with_env_filter(&config.log)
        .init();

    debug!(?config, "loaded config");

    let config = Arc::new(RwLock::new(config));

    if config.read().unwrap().management.enabled {
        let listener = TcpListener::bind("127.0.0.1:0")?;

        let port = listener.local_addr()?.port();

        info!(port, "spawning management server");

        {
            let mut config = config.write().unwrap();
            let host_name = config.management.host_name.clone();
            config.services.push(Service {
                host_name,
                upstream_address: format!("http://localhost:{}", port),
            });
        }

        tokio::task::spawn_local(management::run(config.clone(), listener));
    }

    {
        let config = config.read().unwrap();
        mdns::advertise(&config)?;
    }

    if let Err(error) = proxy::run(config).await {
        let e: &dyn std::error::Error = &*error;
        error!(error = e, "Error running server");
    }

    Ok(())
}
