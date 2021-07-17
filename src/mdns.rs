use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6};

use anyhow::Context;
use if_addrs::IfAddr;
use simple_mdns::ServiceDiscovery;
use tracing::{debug, warn};

use crate::config::Config;

pub fn advertise(config: &crate::config::Config) -> anyhow::Result<()> {
    let interfaces = if_addrs::get_if_addrs()?;

    for service in &config.services {
        if let Err(e) = spawn_service(&config, service, &interfaces) {
            let e: &dyn std::error::Error = &*e;
            warn!(error = e, "Error spawning service");
        }
    }

    Ok(())
}

fn spawn_service(
    config: &Config,
    service: &crate::config::Service,
    interfaces: &[if_addrs::Interface],
) -> anyhow::Result<()> {
    let service_name = Box::leak(Box::new(format!("{}.local", service.host_name)));

    debug!(%service_name, upstream_address = %service.upstream_address, "spawning service");

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
