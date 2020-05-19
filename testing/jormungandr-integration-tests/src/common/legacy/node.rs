use super::BackwardCompatibleRest;
use crate::common::{
    configuration::{JormungandrParams, TestConfig},
    explorer::Explorer,
    jcli_wrapper,
    jormungandr::{JormungandrError, JormungandrLogger},
};
use chain_impl_mockchain::{
    fee::LinearFee,
    fragment::{Fragment, FragmentId},
};
use jormungandr_lib::{
    crypto::hash::Hash,
    interfaces::{BlockDate, FragmentLog},
};
use jormungandr_testing_utils::legacy::NodeConfig;
use jormungandr_testing_utils::testing::{FragmentNode, FragmentNodeError, MemPoolCheck};

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::process::Child;
use std::str::FromStr;

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
        if let Some(address) = &self.p2p.listen_address {
            address.clone()
        } else {
            self.p2p.public_address.clone()
        }
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

impl FragmentNode for BackwardCompatibleJormungandr {
    fn alias(&self) -> &str {
        self.alias()
    }
    fn fragment_logs(&self) -> Result<HashMap<FragmentId, FragmentLog>, FragmentNodeError> {
        //TODO: implement conversion
        self.rest()
            .fragment_logs()
            .map_err(|_| FragmentNodeError::UnknownError)
    }
    fn send_fragment(&self, fragment: Fragment) -> Result<MemPoolCheck, FragmentNodeError> {
        self.rest()
            .send_fragment(fragment)
            .map_err(|_| FragmentNodeError::UnknownError)
    }
    fn log_pending_fragment(&self, fragment_id: FragmentId) {
        println!("Fragment '{}' is still pending", fragment_id);
    }
    fn log_rejected_fragment(&self, fragment_id: FragmentId, reason: String) {
        println!("Fragment '{}' rejected: {}", fragment_id, reason);
    }
    fn log_in_block_fragment(&self, fragment_id: FragmentId, date: BlockDate, block: Hash) {
        println!("Fragment '{}' in block: {} ({})", fragment_id, block, date);
    }
    fn log_content(&self) -> String {
        self.logger().get_log_content()
    }
}

#[derive(Debug)]
pub struct BackwardCompatibleJormungandr {
    pub child: Child,
    pub logger: JormungandrLogger,
    pub config: JormungandrParams<NodeConfig>,
    alias: String,
}

impl BackwardCompatibleJormungandr {
    pub fn from_config(child: Child, config: JormungandrParams<NodeConfig>, alias: String) -> Self {
        Self::new(
            child,
            alias,
            config.log_file_path().expect("no log file defined"),
            config,
        )
    }

    pub fn alias(&self) -> &str {
        self.alias.as_str()
    }

    pub fn new(
        child: Child,
        alias: String,
        log_file_path: impl Into<PathBuf>,
        config: JormungandrParams<NodeConfig>,
    ) -> Self {
        Self {
            child: child,
            alias: alias,
            logger: JormungandrLogger::new(log_file_path.into()),
            config: config,
        }
    }

    pub fn logger(&self) -> &JormungandrLogger {
        &self.logger
    }

    pub fn rest(&self) -> BackwardCompatibleRest {
        BackwardCompatibleRest::new(self.config.rest_uri())
    }

    pub fn shutdown(&self) {
        jcli_wrapper::assert_rest_shutdown(&self.config.rest_uri());
    }

    pub fn fees(&self) -> LinearFee {
        self.config.fees()
    }

    pub fn assert_no_errors_in_log_with_message(&self, message: &str) {
        let error_lines = self.logger.get_lines_with_error().collect::<Vec<String>>();

        assert_eq!(
            error_lines.len(),
            0,
            "{} there are some errors in log ({:?}): {:?}",
            message,
            self.logger.log_file_path,
            error_lines,
        );
    }

    pub fn assert_no_errors_in_log(&self) {
        let error_lines = self.logger.get_lines_with_error().collect::<Vec<String>>();

        assert_eq!(
            error_lines.len(),
            0,
            "there are some errors in log ({:?}): {:?}",
            self.logger.log_file_path,
            error_lines
        );
    }

    pub fn check_no_errors_in_log(&self) -> Result<(), JormungandrError> {
        let error_lines = self.logger.get_lines_with_error().collect::<Vec<String>>();

        if error_lines.len() != 0 {
            return Err(JormungandrError::ErrorInLogs {
                logs: self.logger.get_log_content(),
                log_location: self.logger.log_file_path.clone(),
                error_lines: format!("{:?}", error_lines).to_owned(),
            });
        }
        Ok(())
    }

    pub fn rest_uri(&self) -> String {
        self.config.rest_uri()
    }

    pub fn genesis_block_hash(&self) -> Hash {
        Hash::from_str(self.config.genesis_block_hash()).unwrap()
    }

    pub fn config(&self) -> JormungandrParams<NodeConfig> {
        self.config.clone()
    }

    pub fn pid(&self) -> u32 {
        self.child.id()
    }

    pub fn explorer(&self) -> Explorer {
        Explorer::new(self.rest_uri())
    }
}

impl Drop for BackwardCompatibleJormungandr {
    fn drop(&mut self) {
        self.logger.print_error_and_invalid_logs();
        match self.child.kill() {
            Err(e) => println!("Could not kill {}: {}", self.alias, e),
            Ok(_) => println!("Successfully killed {}", self.alias),
        }
    }
}
