use jormungandr_lib::interfaces::NodeConfig;

use std::net::SocketAddr;
use std::path::{Path, PathBuf};

/// Abstracts over different versions of the node configuration.
pub trait TestConfig {
    fn log_file_path(&self) -> Option<&Path>;
    fn update_log_file_path(&mut self, path: impl Into<PathBuf>);
    fn p2p_listen_address(&self) -> poldercast::Address;
    fn p2p_public_address(&self) -> poldercast::Address;
    fn set_p2p_public_address(&mut self, address: poldercast::Address);
    fn rest_socket_addr(&self) -> SocketAddr;
    fn set_rest_socket_addr(&mut self, addr: SocketAddr);
}

impl TestConfig for NodeConfig {
    fn log_file_path(&self) -> Option<&Path> {
        self.log.as_ref().and_then(|log| log.file_path())
    }

    fn update_log_file_path(&mut self, path: impl Into<PathBuf>) {
        if let Some(log) = self.log.as_mut() {
            log.update_file_path(path);
        }
    }

    fn p2p_listen_address(&self) -> poldercast::Address {
        self.p2p.get_listen_address()
    }

    fn p2p_public_address(&self) -> poldercast::Address {
        self.p2p.public_address.clone()
    }

    fn set_p2p_public_address(&mut self, address: poldercast::Address) {
        self.p2p.public_address = address;
    }

    fn rest_socket_addr(&self) -> SocketAddr {
        self.rest.listen
    }

    fn set_rest_socket_addr(&mut self, addr: SocketAddr) {
        self.rest.listen = addr;
    }
}
