use crate::common::{
    configuration::{jormungandr_config::JormungandrParams, SecretModelFactory},
    file_utils, jcli_wrapper,
    jormungandr::{ConfigurationBuilder, JormungandrProcess, Starter, StartupError},
    process_utils,
};
use chain_crypto::{AsymmetricKey, Ed25519};
use chain_impl_mockchain::chaintypes::ConsensusVersion;
use jormungandr_lib::{
    crypto::key::{Identifier, KeyPair},
    interfaces::{
        Block0Configuration, ConsensusLeaderId, InitialUTxO, NodeSecret, SignedCertificate,
    },
};
use jormungandr_testing_utils::{
    stake_pool::StakePool,
    testing::{signed_delegation_cert, signed_stake_pool_cert},
    wallet::Wallet,
};

use assert_fs::fixture::{ChildPath, PathChild};
use assert_fs::prelude::*;
use rand;
use std::path::PathBuf;

pub fn build_genesis_block(
    block0_config: &Block0Configuration,
    temp_dir: &impl PathChild,
) -> PathBuf {
    let config_file = temp_dir.child("genesis.yaml");
    write_block0_config(&block0_config, &config_file);
    let output_block_file = temp_dir.child("block-0.bin");
    jcli_wrapper::assert_genesis_encode(config_file.path(), &output_block_file);

    output_block_file.path().into()
}

pub fn write_block0_config(block0_config: &Block0Configuration, output_file: &ChildPath) {
    let content = serde_yaml::to_string(&block0_config).unwrap();
    output_file.write_str(&content).unwrap();
}

pub fn create_new_utxo_address() -> Wallet {
    Wallet::new_utxo(&mut rand::rngs::OsRng)
}

pub fn create_new_account_address() -> Wallet {
    Wallet::new_account(&mut rand::rngs::OsRng)
}

pub fn create_new_delegation_address() -> Wallet {
    let account = Wallet::new_account(&mut rand::rngs::OsRng);
    create_new_delegation_address_for(&account.identifier())
}

pub fn create_new_delegation_address_for(delegation_identifier: &Identifier<Ed25519>) -> Wallet {
    Wallet::new_delegation(
        &delegation_identifier.clone().into(),
        &mut rand::rngs::OsRng,
    )
}

pub fn create_new_key_pair<K: AsymmetricKey>() -> KeyPair<K> {
    KeyPair::generate(rand::rngs::OsRng)
}

pub fn start_stake_pool(
    owners: &[Wallet],
    initial_funds: &[Wallet],
    config_builder: &mut ConfigurationBuilder,
) -> Result<(JormungandrProcess, Vec<StakePool>), StartupError> {
    let stake_pools: Vec<StakePool> = owners.iter().map(|x| StakePool::new(x)).collect();

    let stake_pool_registration_certs: Vec<SignedCertificate> = stake_pools
        .iter()
        .map(|x| signed_stake_pool_cert(&x).into())
        .collect();
    let stake_pool_owner_delegation_certs: Vec<SignedCertificate> = stake_pools
        .iter()
        .map(|x| signed_delegation_cert(x.owner(), x.id()).into())
        .collect();

    let mut initial_certs = stake_pool_registration_certs.clone();
    initial_certs.extend(stake_pool_owner_delegation_certs.iter().cloned());

    let leaders: Vec<ConsensusLeaderId> = stake_pools
        .iter()
        .map(|x| x.leader().identifier().into())
        .collect();

    let mut funds: Vec<InitialUTxO> = owners
        .iter()
        .map(|x| InitialUTxO {
            address: x.address(),
            value: 1_000_000_000.into(),
        })
        .collect();

    let funds_non_owners: Vec<InitialUTxO> = initial_funds
        .iter()
        .map(|x| InitialUTxO {
            address: x.address(),
            value: 1_000_000_000.into(),
        })
        .collect();

    funds.extend(funds_non_owners);

    let mut config = config_builder
        .with_block0_consensus(ConsensusVersion::GenesisPraos)
        .with_consensus_leaders_ids(leaders)
        .with_funds(funds)
        .with_explorer()
        .with_initial_certs(initial_certs)
        .build();

    let secrets: Vec<NodeSecret> = stake_pools
        .iter()
        .map(|x| {
            SecretModelFactory::genesis(
                x.kes().signing_key(),
                x.vrf().signing_key(),
                &x.id().to_string(),
            )
        })
        .collect();

    let secret_model_paths = secrets
        .iter()
        .map(|x| SecretModelFactory::serialize(&x))
        .collect();

    *config.secret_models_mut() = secrets;
    *config.secret_model_paths_mut() = secret_model_paths;

    Starter::new()
        .config(config)
        .start()
        .map(|process| (process, stake_pools))
}

pub fn sleep_till_epoch(epoch_interval: u32, grace_period: u32, config: &JormungandrParams) {
    let coeff = epoch_interval * 2;
    let slots_per_epoch: u32 = config
        .block0_configuration()
        .blockchain_configuration
        .slots_per_epoch
        .into();
    let slot_duration: u8 = config
        .block0_configuration()
        .blockchain_configuration
        .slot_duration
        .into();
    let wait_time = ((slots_per_epoch * (slot_duration as u32)) * coeff) + grace_period;
    process_utils::sleep(wait_time.into());
}

pub fn sleep_till_next_epoch(grace_period: u32, config: &JormungandrParams) {
    sleep_till_epoch(1, grace_period, config);
}
