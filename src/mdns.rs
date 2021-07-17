use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6};

use anyhow::Context;
use if_addrs::IfAddr;
use simple_mdns::ServiceDiscovery;
use tracing::debug;

use crate::config::Config;

pub fn advertise(config: &crate::config::Config) -> anyhow::Result<()> {
    let interfaces = if_addrs::get_if_addrs()?;

    for service in &config.services {
        spawn_service(&config, &service.host_name, &interfaces)?;
    }

    Ok(())
}

fn spawn_service(
    config: &Config,
    host_name: &str,
    interfaces: &[if_addrs::Interface],
) -> anyhow::Result<()> {
    let service_name = Box::leak(Box::new(format!("{}.local", host_name)));

    debug!(%service_name, "spawning service");

    let mut discovery = ServiceDiscovery::new(service_name, 60)
        .with_context(|| format!("failed to spawn service with name {:?}", service_name))?;

    for interface in interfaces {
        let socket_addr = match &interface.addr {
            IfAddr::V4(addr) => SocketAddr::V4(SocketAddrV4::new(addr.ip, config.listen_port)),
            IfAddr::V6(addr) => {
                SocketAddr::V6(SocketAddrV6::new(addr.ip, config.listen_port, 0, 0))
            }
        };
        discovery.add_socket_address(socket_addr);
    }

    Ok(())
}
