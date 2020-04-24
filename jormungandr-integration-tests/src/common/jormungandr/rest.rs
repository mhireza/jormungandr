use crate::common::configuration::jormungandr_config::JormungandrConfig;
use jormungandr_lib::interfaces::{
    EpochRewardsInfo, Info, NodeStatsDto, PeerRecord, PeerStats, StakeDistributionDto,
};
use std::{fs::File, io::Read, path::PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RestError {
    #[error("could not deserialize response")]
    CannotDeserialize(#[from] serde_json::Error),
    #[error("could not send reqeuest")]
    SendRequestError(#[from] reqwest::Error),
}

#[derive(Debug)]
pub struct JormungandrRest {
    config: JormungandrConfig,
    certificate: Option<reqwest::Certificate>,
}

impl JormungandrRest {
    pub fn new(config: JormungandrConfig) -> Self {
        Self {
            config: config,
            certificate: None,
        }
    }

    pub fn new_with_cert(config: JormungandrConfig, cert_file: PathBuf) -> Self {
        let mut buf = Vec::new();
        let path = cert_file.as_os_str().to_str().unwrap();
        File::open(path).unwrap().read_to_end(&mut buf).unwrap();
        let cert = reqwest::Certificate::from_pem(&buf).unwrap();
        Self {
            config: config,
            certificate: Some(cert),
        }
    }

    fn print_response_text(&self, text: &str) {
        println!("Response: {}", text);
    }

    pub fn epoch_reward_history(&self, epoch: u32) -> Result<EpochRewardsInfo, RestError> {
        let request = format!("rewards/epoch/{}", epoch);
        let response_text = self.get(&request)?.text()?;
        self.print_response_text(&response_text);
        serde_json::from_str(&response_text).map_err(|err| RestError::CannotDeserialize(err))
    }

    pub fn reward_history(&self, length: u32) -> Result<Vec<EpochRewardsInfo>, RestError> {
        let request = format!("rewards/history/{}", length);
        let response_text = self.get(&request)?.text()?;
        self.print_response_text(&response_text);
        serde_json::from_str(&response_text).map_err(|err| RestError::CannotDeserialize(err))
    }

    fn get(&self, path: &str) -> Result<reqwest::blocking::Response, reqwest::Error> {
        match &self.certificate {
            None => {
                reqwest::blocking::get(&format!("{}/v0/{}", self.config.get_node_address(), path))
            }
            Some(cert) => {
                let client = reqwest::blocking::Client::builder()
                    .add_root_certificate(cert.clone())
                    .build()
                    .unwrap();
                client
                    .get(&format!(
                        "{}/v0/{}",
                        self.config.get_node_address().replace("http", "https"),
                        path
                    ))
                    .send()
            }
        }
    }

    pub fn stake_distribution(&self) -> Result<StakeDistributionDto, RestError> {
        let response_text = self.get("stake")?.text()?;
        self.print_response_text(&response_text);
        serde_json::from_str(&response_text).map_err(|err| RestError::CannotDeserialize(err))
    }

    pub fn stake_pools(&self) -> Result<Vec<String>, RestError> {
        let response_text = self.get("stake_pools")?.text()?;
        self.print_response_text(&response_text);
        serde_json::from_str(&response_text).map_err(|err| RestError::CannotDeserialize(err))
    }

    pub fn stake_distribution_at(&self, epoch: u32) -> Result<StakeDistributionDto, RestError> {
        let request = format!("stake/{}", epoch);
        let response_text = self.get(&request)?.text()?;
        self.print_response_text(&response_text);
        serde_json::from_str(&response_text).map_err(|err| RestError::CannotDeserialize(err))
    }

    pub fn stats(&self) -> Result<NodeStatsDto, RestError> {
        let response_text = self.get("node/stats")?.text()?;
        serde_json::from_str(&response_text).map_err(|err| RestError::CannotDeserialize(err))
    }

    pub fn network_stats(&self) -> Result<Vec<PeerStats>, RestError> {
        let response_text = self.get("network/stats")?.text()?;
        serde_json::from_str(&response_text).map_err(|err| RestError::CannotDeserialize(err))
    }

    pub fn p2p_quarantined(&self) -> Result<Vec<PeerRecord>, RestError> {
        let response_text = self.get("network/p2p/quarantined")?.text()?;
        serde_json::from_str(&response_text).map_err(|err| RestError::CannotDeserialize(err))
    }

    pub fn p2p_non_public(&self) -> Result<Vec<PeerRecord>, RestError> {
        let response_text = self.get("network/p2p/non_public")?.text()?;
        serde_json::from_str(&response_text).map_err(|err| RestError::CannotDeserialize(err))
    }

    pub fn p2p_available(&self) -> Result<Vec<PeerRecord>, RestError> {
        let response_text = self.get("network/p2p/available")?.text()?;
        serde_json::from_str(&response_text).map_err(|err| RestError::CannotDeserialize(err))
    }

    pub fn p2p_view(&self) -> Result<Vec<Info>, RestError> {
        let response_text = self.get("network/p2p/view")?.text()?;
        serde_json::from_str(&response_text).map_err(|err| RestError::CannotDeserialize(err))
    }
}
