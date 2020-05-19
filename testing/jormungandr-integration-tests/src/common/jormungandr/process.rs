use super::{logger::JormungandrLogger, rest, JormungandrError, JormungandrRest};
use crate::common::configuration::{JormungandrParams, TestConfig};
use crate::common::explorer::Explorer;
use crate::common::jcli_wrapper;
use chain_impl_mockchain::fee::LinearFee;
use jormungandr_lib::crypto::hash::Hash;
use jormungandr_lib::interfaces::TrustedPeer;
use std::net::SocketAddr;
use std::process::Child;
use std::str::FromStr;

#[derive(Debug)]
pub struct JormungandrProcess {
    pub child: Child,
    pub logger: JormungandrLogger,
    alias: String,
    p2p_public_address: poldercast::Address,
    rest_socket_addr: SocketAddr,
    genesis_block_hash: Hash,
    fees: LinearFee,
}

impl JormungandrProcess {
    pub(crate) fn from_config<Conf: TestConfig>(
        child: Child,
        config: &JormungandrParams<Conf>,
        alias: String,
    ) -> Self {
        let log_file_path = config.log_file_path().unwrap();
        let logger = JormungandrLogger::new(log_file_path);
        let node_config = config.node_config();
        let p2p_public_address = node_config.p2p_public_address();
        let rest_socket_addr = node_config.rest_socket_addr();
        let genesis_block_hash = Hash::from_str(config.genesis_block_hash()).unwrap();
        let fees = config.fees();
        JormungandrProcess {
            child,
            alias,
            logger,
            p2p_public_address,
            rest_socket_addr,
            genesis_block_hash,
            fees,
        }
    }

    pub fn alias(&self) -> &str {
        &self.alias
    }

    pub fn rest(&self) -> JormungandrRest {
        JormungandrRest::new(self.rest_uri())
    }

    pub fn shutdown(&self) {
        jcli_wrapper::assert_rest_shutdown(&self.rest_uri());
    }

    pub fn address(&self) -> poldercast::Address {
        self.p2p_public_address.clone()
    }

    pub fn log_stats(&self) {
        println!("{:?}", self.rest().stats());
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
        rest::uri_from_socket_addr(self.rest_socket_addr)
    }

    pub fn fees(&self) -> LinearFee {
        self.fees
    }

    pub fn genesis_block_hash(&self) -> Hash {
        self.genesis_block_hash
    }

    pub fn pid(&self) -> u32 {
        self.child.id()
    }

    pub fn explorer(&self) -> Explorer {
        Explorer::new(self.rest_socket_addr.to_string())
    }

    pub fn to_trusted_peer(&self) -> TrustedPeer {
        TrustedPeer {
            address: self.p2p_public_address.clone(),
        }
    }

    pub fn stop(self) {
        match self.child.kill() {
            Err(e) => println!("Could not kill {}: {}", self.alias, e),
            Ok(_) => {
                println!("Successfully killed {}", self.alias);
            }
        }
    }
}

impl Drop for JormungandrProcess {
    fn drop(&mut self) {
        // There's no kill like overkill
        let _ = self.child.kill();

        // FIXME: These should be better done in a test harness
        self.child.wait().unwrap();
        self.logger.print_error_and_invalid_logs();
    }
}
