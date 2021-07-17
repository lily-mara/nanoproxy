use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub listen_port: u16,
    pub listen_host: String,
    pub services: Vec<Service>,
    pub log: String,
}

#[derive(Deserialize, Debug)]
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
        .set_default("services", Vec::<config::Value>::new())
        .unwrap();

    config.try_into().unwrap()
}
