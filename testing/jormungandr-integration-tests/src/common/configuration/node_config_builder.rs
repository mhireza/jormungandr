#![allow(dead_code)]

use assert_fs::fixture::PathChild;
use std::path::PathBuf;

use jormungandr_lib::{
    interfaces::{
        Explorer, Log, LogEntry, LogOutput, Mempool, NodeConfig, P2p, Policy, Rest,
        TopicsOfInterest, TrustedPeer,
    },
    time::Duration,
};

pub struct NodeConfigBuilder {
    pub storage: Option<PathBuf>,
    pub log: Option<Log>,
    pub rest: Rest,
    pub p2p: P2p,
    pub mempool: Option<Mempool>,
    pub explorer: Explorer,
}

const DEFAULT_HOST: &str = "127.0.0.1";

fn default_log(temp_dir: &impl PathChild) -> Log {
    let log_file = temp_dir.child("node.log");
    Log(vec![LogEntry {
        level: "trace".to_string(),
        format: "json".to_string(),
        output: LogOutput::File(log_file.path().to_str().unwrap().into()),
    }])
}

impl NodeConfigBuilder {
    pub fn new() -> NodeConfigBuilder {
        let rest_port = super::get_available_port();
        let public_address_port = super::get_available_port();
        let grpc_public_address: poldercast::Address = format!(
            "/ip4/{}/tcp/{}",
            DEFAULT_HOST,
            public_address_port.to_string()
        )
        .parse()
        .unwrap();

        NodeConfigBuilder {
            storage: None,
            log: None,
            rest: Rest {
                listen: format!("{}:{}", DEFAULT_HOST, rest_port.to_string())
                    .parse()
                    .unwrap(),
            },
            p2p: P2p {
                trusted_peers: vec![],
                public_address: grpc_public_address,
                listen_address: None,
                max_inbound_connections: None,
                max_connections: None,
                topics_of_interest: Some(TopicsOfInterest {
                    messages: String::from("high"),
                    blocks: String::from("high"),
                }),
                allow_private_addresses: false,
                policy: Some(Policy {
                    quarantine_duration: Some(Duration::new(1, 0)),
                    quarantine_whitelist: None,
                }),
                layers: None,
            },
            mempool: Some(Mempool::default()),
            explorer: Explorer { enabled: false },
        }
    }

    pub fn with_explorer(&mut self) -> &mut Self {
        self.explorer.enabled = true;
        self
    }

    pub fn with_policy(&mut self, policy: Policy) -> &mut Self {
        self.p2p.policy = Some(policy);
        self
    }

    pub fn with_log(&mut self, log: Log) -> &mut Self {
        self.log = Some(log);
        self
    }

    pub fn with_trusted_peers(&mut self, trusted_peers: Vec<TrustedPeer>) -> &mut Self {
        self.p2p.trusted_peers = trusted_peers;
        self
    }

    pub fn with_public_address(&mut self, public_address: String) -> &mut Self {
        self.p2p.public_address = public_address.parse().unwrap();
        self
    }

    pub fn with_listen_address(&mut self, listen_address: String) -> &mut Self {
        self.p2p.listen_address = Some(listen_address.parse().unwrap());
        self
    }

    pub fn with_mempool(&mut self, mempool: Mempool) -> &mut Self {
        self.mempool = Some(mempool);
        self
    }

    pub fn with_storage(&mut self, path: PathBuf) -> &mut Self {
        self.storage = Some(path);
        self
    }

    pub fn build(&self, temp_dir: &impl PathChild) -> NodeConfig {
        let log = self
            .log
            .as_ref()
            .cloned()
            .unwrap_or_else(|| default_log(temp_dir));
        NodeConfig {
            storage: self.storage.clone(),
            log: Some(log),
            rest: self.rest.clone(),
            p2p: self.p2p.clone(),
            mempool: self.mempool.clone(),
            explorer: self.explorer.clone(),
            bootstrap_from_trusted_peers: Some(!self.p2p.trusted_peers.is_empty()),
            skip_bootstrap: Some(self.p2p.trusted_peers.is_empty()),
        }
    }
}
