use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};

pub type SharedConfig = Arc<RwLock<Config>>;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub listen_port: u16,
    pub listen_host: String,
    pub services: Vec<Service>,
    pub log: String,
    pub management: Management,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Management {
    pub host_name: String,
    pub enabled: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Service {
    pub host_name: String,
    pub upstream_address: String,
}

pub fn load() -> Config {
    let mut config = config::Config::default();

    config
        .merge(config::File::with_name("nanoproxy").required(false))
        .unwrap()
        .merge(config::File::with_name("/etc/nanoproxy").required(false))
        .unwrap()
        .set_default("listen_host", "0.0.0.0")
        .unwrap()
        .set_default("listen_port", 80)
        .unwrap()
        .set_default("log", "info")
        .unwrap()
        .set_default("management.enabled", true)
        .unwrap()
        .set_default("management.host_name", "nanoproxy")
        .unwrap()
        .set_default("services", Vec::<config::Value>::new())
        .unwrap();

    config.try_into().unwrap()
}
