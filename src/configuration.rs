// This file manages runtime configuration of the providence daemon.
// Configuration is split into two categories:
//   1. Static configuration (generally configured for hardware; in embedded development this would
//      be persisted in a read-only partition).
//   2. Dynamic configuration (generally configured for a user's environment; in embedded
//      development this would be persisted in a writable partition).

#[derive(Debug)]
enum ClientIpConfig {
    Dhcp,
    Manual {
        ip_addr: std::net::Ipv4Addr,
        subnet_mask: std::net::Ipv4Addr,
        default_gateway: std::net::Ipv4Addr,
        dns_servers: [std::net::Ipv4Addr; 2],
    },
}

#[derive(Debug)]
enum NetworkSecurityConfig {
    Open,
    Wpa {
        password: String,
    },
    Wpa2 {
        password: String,
    },
}

#[derive(Debug)]
pub struct WifiClientConfig {
    network_name: String,
    network_security: NetworkSecurityConfig,
    network_ip_config: ClientIpConfig,
}

#[derive(Debug)]
struct ApIpConfig {
    ip_addr: std::net::Ipv4Addr,
    subnet_mask: std::net::Ipv4Addr,
}

#[derive(Debug)]
pub struct WifiApConfig {
    network_name: String,
    network_security: NetworkSecurityConfig,
    network_ip_config: ApIpConfig,
}
