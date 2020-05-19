use crate::common::{
    configuration::{
        self, jormungandr_config::JormungandrParams, Block0ConfigurationBuilder, NodeConfigBuilder,
        SecretModelFactory,
    },
    jcli_wrapper,
    jormungandr::JormungandrProcess,
    startup::{build_genesis_block, create_new_key_pair},
};
use chain_crypto::Ed25519;
use chain_impl_mockchain::{chaintypes::ConsensusVersion, fee::LinearFee};
use jormungandr_lib::interfaces::{
    ActiveSlotCoefficient, ConsensusLeaderId, EpochStabilityDepth, Initial, InitialUTxO,
    KESUpdateSpeed, Log, Mempool, NodeConfig, NumberOfSlotsPerEpoch, Policy, SignedCertificate,
    SlotDuration, TrustedPeer,
};

use assert_fs::fixture::{ChildPath, PathChild};

pub struct ConfigurationBuilder {
    funds: Vec<Initial>,
    certs: Vec<Initial>,
    block0_hash: Option<String>,
    block0_consensus: ConsensusVersion,
    consensus_genesis_praos_active_slot_coeff: ActiveSlotCoefficient,
    slots_per_epoch: NumberOfSlotsPerEpoch,
    slot_duration: SlotDuration,
    epoch_stability_depth: EpochStabilityDepth,
    kes_update_speed: KESUpdateSpeed,
    linear_fees: LinearFee,
    consensus_leader_ids: Vec<ConsensusLeaderId>,
    node_config_builder: NodeConfigBuilder,
    rewards_history: bool,
}

impl ConfigurationBuilder {
    pub fn new() -> Self {
        ConfigurationBuilder {
            funds: vec![],
            certs: vec![],
            consensus_leader_ids: vec![],
            block0_hash: None,
            block0_consensus: ConsensusVersion::Bft,
            slots_per_epoch: NumberOfSlotsPerEpoch::new(100).unwrap(),
            slot_duration: SlotDuration::new(1).unwrap(),
            epoch_stability_depth: 2600u32.into(),
            linear_fees: LinearFee::new(0, 0, 0),
            consensus_genesis_praos_active_slot_coeff: ActiveSlotCoefficient::MAXIMUM,
            kes_update_speed: KESUpdateSpeed::new(12 * 3600).unwrap(),
            node_config_builder: NodeConfigBuilder::new(),
            rewards_history: false,
        }
    }

    pub fn with_slots_per_epoch(&mut self, slots_per_epoch: u32) -> &mut Self {
        self.slots_per_epoch = NumberOfSlotsPerEpoch::new(slots_per_epoch).unwrap();
        self
    }

    pub fn with_slot_duration(&mut self, slot_duration: u8) -> &mut Self {
        self.slot_duration = SlotDuration::new(slot_duration).unwrap();
        self
    }

    pub fn with_epoch_stability_depth(&mut self, epoch_stability_depth: u32) -> &mut Self {
        self.epoch_stability_depth = epoch_stability_depth.into();
        self
    }

    pub fn with_kes_update_speed(&mut self, kes_update_speed: KESUpdateSpeed) -> &mut Self {
        self.kes_update_speed = kes_update_speed;
        self
    }

    pub fn with_rewards_history(&mut self) -> &mut Self {
        self.rewards_history = true;
        self
    }
    /*
    pub fn with_block_hash_from(&mut self, config: &JormungandrParams) -> &mut Self {
        self.with_block_hash(config.genesis_block_hash.clone())
    }*/

    pub fn with_linear_fees(&mut self, linear_fees: LinearFee) -> &mut Self {
        self.linear_fees = linear_fees;
        self
    }

    pub fn with_consensus_leaders_ids(
        &mut self,
        consensus_leader_ids: Vec<ConsensusLeaderId>,
    ) -> &mut Self {
        self.consensus_leader_ids = consensus_leader_ids.clone();
        self
    }

    pub fn with_initial_certs(&mut self, certs: Vec<SignedCertificate>) -> &mut Self {
        self.certs
            .extend(certs.iter().map(|cert| Initial::Cert(cert.clone())));
        self
    }

    pub fn with_explorer(&mut self) -> &mut Self {
        self.node_config_builder.with_explorer();
        self
    }

    pub fn with_storage(&mut self, temp_dir: &ChildPath) -> &mut Self {
        self.node_config_builder
            .with_storage(temp_dir.path().into());
        self
    }

    pub fn with_block0_consensus(&mut self, consensus: ConsensusVersion) -> &mut Self {
        self.block0_consensus = consensus;
        self
    }

    pub fn with_consensus_genesis_praos_active_slot_coeff(
        &mut self,
        active_slot_coeff: ActiveSlotCoefficient,
    ) -> &mut Self {
        self.consensus_genesis_praos_active_slot_coeff = active_slot_coeff;
        self
    }

    pub fn with_funds(&mut self, initial: Vec<InitialUTxO>) -> &mut Self {
        self.funds.push(Initial::Fund(initial));
        self
    }

    pub fn with_certs(&mut self, initial: Vec<SignedCertificate>) -> &mut Self {
        self.certs
            .extend(initial.iter().cloned().map(Initial::Cert));
        self
    }

    pub fn with_log(&mut self, log: Log) -> &mut Self {
        self.node_config_builder.with_log(log);
        self
    }

    pub fn with_policy(&mut self, policy: Policy) -> &mut Self {
        self.node_config_builder.with_policy(policy);
        self
    }

    pub fn with_mempool(&mut self, mempool: Mempool) -> &mut Self {
        self.node_config_builder.with_mempool(mempool);
        self
    }

    pub fn with_trusted_peers(&mut self, trusted_peers: Vec<TrustedPeer>) -> &mut Self {
        self.node_config_builder.with_trusted_peers(trusted_peers);
        self
    }

    pub fn with_trusted_peer(&mut self, node: &JormungandrProcess) -> &mut Self {
        self.node_config_builder
            .with_trusted_peers(vec![node.to_trusted_peer()]);
        self
    }

    pub fn with_public_address(&mut self, public_address: String) -> &mut Self {
        self.node_config_builder.with_public_address(public_address);
        self
    }

    pub fn with_listen_address(&mut self, listen_address: String) -> &mut Self {
        self.node_config_builder
            .with_listen_address(listen_address.parse().unwrap());
        self
    }

    pub fn with_block_hash(&mut self, block0_hash: String) -> &mut Self {
        self.block0_hash = Some(block0_hash.clone());
        self
    }

    pub fn build(&self, temp_dir: &impl PathChild) -> JormungandrParams<NodeConfig> {
        let node_config = self.node_config_builder.build(temp_dir);

        let leader_key_pair = create_new_key_pair::<Ed25519>();
        let mut leaders_ids = self.consensus_leader_ids.clone();
        leaders_ids.push(leader_key_pair.identifier().into());

        let mut initial: Vec<Initial> = Vec::new();
        initial.extend(self.funds.iter().cloned());
        initial.extend(self.certs.iter().cloned());

        let block0_config = Block0ConfigurationBuilder::new()
            .with_initial(initial)
            .with_leaders(leaders_ids)
            .with_block0_consensus(self.block0_consensus.clone())
            .with_kes_update_speed(self.kes_update_speed.clone())
            .with_slots_per_epoch(self.slots_per_epoch)
            .with_slot_duration(self.slot_duration)
            .with_epoch_stability_depth(self.epoch_stability_depth)
            .with_active_slot_coeff(self.consensus_genesis_praos_active_slot_coeff.clone())
            .with_linear_fees(self.linear_fees.clone())
            .build();

        let path_to_output_block = build_genesis_block(&block0_config, temp_dir);
        let genesis_block_hash = match self.block0_hash {
            Some(ref value) => value.clone(),
            None => jcli_wrapper::assert_genesis_hash(&path_to_output_block),
        };

        let secret_model = SecretModelFactory::bft(leader_key_pair.signing_key());
        let secret_model_file = temp_dir.child("node.secret");
        configuration::write_secret(&secret_model, &secret_model_file);

        JormungandrParams::new(
            path_to_output_block.clone(),
            genesis_block_hash,
            node_config,
            vec![secret_model_file.path().into()],
            block0_config,
            vec![secret_model],
            self.rewards_history,
        )
    }
}
