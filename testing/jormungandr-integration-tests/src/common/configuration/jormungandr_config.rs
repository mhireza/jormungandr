#![allow(dead_code)]

use super::TestConfig;
use chain_core::mempack;
use chain_impl_mockchain::{block::Block, fee::LinearFee, fragment::Fragment};
use jormungandr_lib::interfaces::{Block0Configuration, NodeConfig, NodeSecret, UTxOInfo};
use jormungandr_testing_utils::wallet::Wallet;

use assert_fs::prelude::*;
use assert_fs::TempDir;
use serde::Serialize;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct JormungandrParams<Conf = NodeConfig> {
    genesis_block_path: PathBuf,
    genesis_block_hash: String,
    node_config: Conf,
    secret_model_paths: Vec<PathBuf>,
    block0_configuration: Block0Configuration,
    secret_models: Vec<NodeSecret>,
    rewards_history: bool,
}

impl<Conf: TestConfig> JormungandrParams<Conf> {
    pub(crate) fn new(
        genesis_block_path: PathBuf,
        genesis_block_hash: String,
        node_config: Conf,
        secret_model_paths: Vec<PathBuf>,
        block0_configuration: Block0Configuration,
        secret_models: Vec<NodeSecret>,
        rewards_history: bool,
    ) -> Self {
        JormungandrParams {
            genesis_block_path,
            genesis_block_hash,
            node_config,
            secret_model_paths,
            block0_configuration,
            secret_models,
            rewards_history,
        }
    }

    pub fn block0_configuration(&self) -> &Block0Configuration {
        &self.block0_configuration
    }

    pub fn block0_configuration_mut(&mut self) -> &mut Block0Configuration {
        &mut self.block0_configuration
    }

    pub fn genesis_block_path(&self) -> &Path {
        &self.genesis_block_path
    }

    pub fn genesis_block_hash(&self) -> &str {
        &self.genesis_block_hash
    }

    pub fn rewards_history(&self) -> bool {
        self.rewards_history
    }

    pub fn log_file_path(&self) -> Option<&Path> {
        self.node_config.log_file_path()
    }

    pub fn secret_model_paths_mut(&mut self) -> &mut Vec<PathBuf> {
        &mut self.secret_model_paths
    }

    pub fn secret_models_mut(&mut self) -> &mut Vec<NodeSecret> {
        &mut self.secret_models
    }

    pub fn secret_model_paths(&self) -> &Vec<PathBuf> {
        &self.secret_model_paths
    }

    pub fn secret_models(&self) -> &Vec<NodeSecret> {
        &self.secret_models
    }

    pub fn rest_uri(&self) -> String {
        format!("http://{}/api", self.node_config.rest_socket_addr())
    }

    pub fn node_config(&self) -> &Conf {
        &self.node_config
    }

    pub fn refresh_instance_params(&mut self, temp_dir: &TempDir) {
        self.regenerate_ports();
        let log_file = temp_dir.child("node.log");
        self.node_config.update_log_file_path(log_file.path());
    }

    fn regenerate_ports(&mut self) {
        self.node_config.set_rest_socket_addr(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::LOCALHOST),
            super::get_available_port(),
        ));
        self.node_config.set_p2p_public_address(
            format!(
                "/ip4/127.0.0.1/tcp/{}",
                super::get_available_port().to_string()
            )
            .parse()
            .unwrap(),
        );
    }

    pub fn fees(&self) -> LinearFee {
        self.block0_configuration
            .blockchain_configuration
            .linear_fees
            .clone()
    }

    pub fn get_p2p_listen_port(&self) -> u16 {
        let address = self.node_config.p2p_listen_address().to_string();
        let tokens: Vec<&str> = address.split("/").collect();
        assert_eq!(
            tokens.get(3),
            Some(&"tcp"),
            "expected a tcp part in p2p.public_address"
        );
        let port_str = tokens
            .get(4)
            .expect("cannot extract port from p2p.public_address");
        port_str.parse().unwrap()
    }

    pub fn block0_utxo(&self) -> Vec<UTxOInfo> {
        let block0_bytes = std::fs::read(self.genesis_block_path()).expect(&format!(
            "Failed to load block 0 binary file '{}'",
            self.genesis_block_path().display()
        ));
        mempack::read_from_raw::<Block>(&block0_bytes)
            .expect(&format!(
                "Failed to parse block in block 0 file '{}'",
                self.genesis_block_path().display()
            ))
            .contents
            .iter()
            .filter_map(|fragment| match fragment {
                Fragment::Transaction(transaction) => Some((transaction, fragment.hash())),
                _ => None,
            })
            .map(|(transaction, fragment_id)| {
                transaction
                    .as_slice()
                    .outputs()
                    .iter()
                    .enumerate()
                    .map(move |(idx, output)| {
                        UTxOInfo::new(
                            fragment_id.into(),
                            idx as u8,
                            output.address.clone().into(),
                            output.value.into(),
                        )
                    })
            })
            .flatten()
            .collect()
    }

    pub fn block0_utxo_for_address(&self, wallet: &Wallet) -> UTxOInfo {
        let utxo = self
            .block0_utxo()
            .into_iter()
            .find(|utxo| *utxo.address() == wallet.address())
            .expect(&format!(
                "No UTxO found in block 0 for address '{:?}'",
                wallet
            ));
        println!(
            "Utxo found for address {}: {:?}",
            wallet.address().to_string(),
            &utxo
        );
        utxo
    }
}

impl<Conf: Serialize> JormungandrParams<Conf> {
    pub fn write_node_config(&self, temp_dir: &TempDir) -> PathBuf {
        let content =
            serde_yaml::to_string(&self.node_config).expect("cannot serialize node config");
        let file = temp_dir.child("node_config.yml");
        file.write_str(&content);
        file.path().into()
    }
}
