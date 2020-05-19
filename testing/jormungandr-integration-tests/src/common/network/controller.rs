use crate::common::{
    configuration::jormungandr_config::JormungandrParams,
    jormungandr::starter::{Starter, StartupError},
    jormungandr::JormungandrProcess,
};
use chain_impl_mockchain::header::HeaderId;
use jormungandr_lib::interfaces::NodeConfig;
use jormungandr_testing_utils::testing::network_builder::NodeSetting;
use jormungandr_testing_utils::testing::network_builder::{
    LeadershipMode, PersistenceMode, Settings, SpawnParams, Wallet,
};

use assert_fs::prelude::*;
use assert_fs::TempDir;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ControllerError {
    #[error("node not found {0}")]
    NodeNotFound(String),
    #[error("wallet not found {0}")]
    WalletNotFound(String),
    #[error("io error")]
    IOError(#[from] std::io::Error),
    #[error("serialization error")]
    SerializationError(#[from] serde_yaml::Error),
    #[error("node startup error")]
    SpawnError(#[from] StartupError),
}

pub struct Controller {
    settings: Settings,
    working_directory: TempDir,
    block0_file: PathBuf,
    block0_hash: HeaderId,
}

impl Controller {
    pub fn new(
        title: &str,
        settings: Settings,
        working_directory: TempDir,
    ) -> Result<Self, ControllerError> {
        use chain_core::property::Serialize as _;

        let block0 = settings.block0.to_block();
        let block0_hash = block0.header.hash();

        let block0_file = working_directory.child("block0.bin").path().into();
        let file = std::fs::File::create(&block0_file)?;
        block0.serialize(file)?;

        Ok(Controller {
            settings: settings,
            block0_file,
            block0_hash,
            working_directory,
        })
    }

    pub fn wallet(&mut self, wallet: &str) -> Result<Wallet, ControllerError> {
        if let Some(wallet) = self.settings.wallets.remove(wallet) {
            Ok(wallet)
        } else {
            Err(ControllerError::WalletNotFound(wallet.to_owned()).into())
        }
    }

    pub fn node_config(&self, alias: &str) -> Result<NodeConfig, ControllerError> {
        Ok(self.node_settings(alias)?.config.clone())
    }

    fn node_settings(&self, alias: &str) -> Result<&NodeSetting, ControllerError> {
        if let Some(node_setting) = self.settings.nodes.get(alias) {
            return Ok(node_setting);
        } else {
            return Err(ControllerError::NodeNotFound(alias.to_string()));
        }
    }

    pub fn spawn_and_wait(&mut self, alias: &str) -> JormungandrProcess {
        self.spawn_node(alias, PersistenceMode::InMemory, LeadershipMode::Leader)
            .expect(&format!("cannot start {}", alias))
    }

    pub fn spawn_as_passive_and_wait(&mut self, alias: &str) -> JormungandrProcess {
        self.spawn_node(alias, PersistenceMode::InMemory, LeadershipMode::Passive)
            .expect(&format!("cannot start {}", alias))
    }

    pub fn spawn_node_async(&mut self, alias: &str) -> Result<JormungandrProcess, ControllerError> {
        let mut spawn_params = SpawnParams::new(alias);
        spawn_params.leadership_mode(LeadershipMode::Leader);
        spawn_params.persistence_mode(PersistenceMode::InMemory);

        let config = self.make_config_for(&mut spawn_params).unwrap();
        Starter::new()
            .config(config)
            .alias(spawn_params.alias.clone())
            .from_genesis(spawn_params.get_leadership_mode().clone().into())
            .role(spawn_params.get_leadership_mode().into())
            .start_async()
            .map_err(|e| ControllerError::SpawnError(e))
    }

    pub fn expect_spawn_failed(
        &mut self,
        spawn_params: &mut SpawnParams,
        expected_msg: &str,
    ) -> Result<(), ControllerError> {
        let config = self.make_config_for(spawn_params).unwrap();
        Starter::new()
            .config(config)
            .from_genesis(spawn_params.get_leadership_mode().clone().into())
            .role(spawn_params.get_leadership_mode().into())
            .start_with_fail_in_logs(expected_msg)
            .map_err(|e| ControllerError::SpawnError(e))
    }

    pub fn spawn_custom(
        &mut self,
        spawn_params: &mut SpawnParams,
    ) -> Result<JormungandrProcess, ControllerError> {
        let config = self.make_config_for(spawn_params).unwrap();
        Starter::new()
            .config(config)
            .alias(spawn_params.alias.clone())
            .from_genesis(spawn_params.get_leadership_mode().clone().into())
            .role(spawn_params.get_leadership_mode().into())
            .start()
            .map_err(|e| ControllerError::SpawnError(e))
    }

    fn make_config_for(
        &mut self,
        spawn_params: &mut SpawnParams,
    ) -> Result<JormungandrParams, ControllerError> {
        let mut node_setting = self.node_settings(&spawn_params.alias)?;
        spawn_params.override_settings(&mut node_setting.config);

        let dir = self.working_directory.child(&node_setting.alias);

        if let PersistenceMode::Persistent = spawn_params.get_persistence_mode() {
            let path_to_storage = dir.child("storage").path().into();
            node_setting.config.storage = Some(path_to_storage);
        }

        dir.create_dir_all().unwrap();

        let config_secret = dir.child("node_secret.xml").path().into();

        serde_yaml::to_writer(
            std::fs::File::create(&config_secret)?,
            node_setting.secrets(),
        )?;

        Ok(JormungandrParams::new(
            self.block0_file.clone(),
            self.block0_hash.to_string(),
            node_setting.config().clone(),
            vec![config_secret],
            self.settings.block0.clone(),
            vec![node_setting.secrets().clone()],
            false,
        ))
    }

    pub fn spawn_node(
        &mut self,
        alias: &str,
        persistence_mode: PersistenceMode,
        leadership_mode: LeadershipMode,
    ) -> Result<JormungandrProcess, ControllerError> {
        self.spawn_custom(
            SpawnParams::new(alias)
                .leadership_mode(leadership_mode)
                .persistence_mode(persistence_mode),
        )
    }
}
