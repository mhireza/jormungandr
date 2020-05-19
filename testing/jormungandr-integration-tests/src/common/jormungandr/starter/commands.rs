use super::{FromGenesis, Role};
use crate::common::configuration::{JormungandrParams, TestConfig};

use assert_fs::TempDir;
use serde::Serialize;
use std::fs::File;
use std::path::Path;
use std::process::{Command, Stdio};

pub struct CommandBuilder<'a> {
    bin: &'a Path,
    config: Option<&'a Path>,
    genesis_block: GenesisBlockOption<'a>,
    secrets: Vec<&'a Path>,
    log_file: Option<&'a Path>,
    rewards_history: bool,
}

enum GenesisBlockOption<'a> {
    None,
    Hash(&'a str),
    Path(&'a Path),
}

impl<'a> CommandBuilder<'a> {
    pub fn new(bin: &'a Path) -> Self {
        CommandBuilder {
            bin,
            config: None,
            genesis_block: GenesisBlockOption::None,
            secrets: Vec::new(),
            log_file: None,
            rewards_history: false,
        }
    }

    pub fn config(mut self, path: &'a Path) -> Self {
        self.config = Some(path);
        self
    }

    pub fn genesis_block_hash(mut self, hash: &'a str) -> Self {
        self.genesis_block = GenesisBlockOption::Hash(hash);
        self
    }

    pub fn genesis_block_path(mut self, path: &'a Path) -> Self {
        self.genesis_block = GenesisBlockOption::Path(path);
        self
    }

    pub fn leader_with_secrets<Iter>(mut self, secrets: Iter) -> Self
    where
        Iter: IntoIterator,
        Iter::Item: 'a + AsRef<Path>,
    {
        self.secrets = secrets.into_iter().map(|item| item.as_ref()).collect();
        self
    }

    pub fn log_file(mut self, path: Option<&'a Path>) -> Self {
        self.log_file = path;
        self
    }

    pub fn rewards_history(mut self, report: bool) -> Self {
        self.rewards_history = report;
        self
    }

    pub fn command(self) -> Command {
        let mut command = Command::new(self.bin);
        for secret_path in self.secrets {
            command.arg("--secret").arg(secret_path);
        }

        if self.rewards_history {
            command.arg("--rewards-report-all");
        }

        let config_path = self
            .config
            .expect("configuration file path needs to be set");
        command.arg("--config").arg(config_path);

        match self.genesis_block {
            GenesisBlockOption::Hash(hash) => {
                command.arg("--genesis-block-hash").arg(hash);
            }
            GenesisBlockOption::Path(path) => {
                command.arg("--genesis-block-path").arg(path);
            }
            GenesisBlockOption::None => {
                panic!("one of the genesis block options needs to be specified")
            }
        }

        if let Some(log_file) = self.log_file {
            command.stderr(get_stdio_from_log_file(log_file));
        }

        println!("Running start jormungandr command: {:?}", &command);
        command
    }
}

#[cfg(unix)]
fn get_stdio_from_log_file(log_file_path: &Path) -> std::process::Stdio {
    use std::os::unix::io::{FromRawFd, IntoRawFd};
    let file = File::create(log_file_path).expect("couldn't create log file for jormungandr");
    unsafe { Stdio::from_raw_fd(file.into_raw_fd()) }
}

#[cfg(windows)]
fn get_stdio_from_log_file(log_file_path: &Path) -> std::process::Stdio {
    use std::os::windows::io::{FromRawHandle, IntoRawHandle};
    let file = File::create(log_file_path).expect("couldn't create log file for jormungandr");
    unsafe { Stdio::from_raw_handle(file.into_raw_handle()) }
}

pub fn get_command<Conf: TestConfig + Serialize>(
    config: &JormungandrParams<Conf>,
    bin_path: impl AsRef<Path>,
    role: Role,
    from_genesis: FromGenesis,
    temp_dir: &TempDir,
) -> Command {
    let bin_path = bin_path.as_ref();
    let config_path = config.write_node_config(temp_dir);
    let builder = CommandBuilder::new(bin_path)
        .config(&config_path)
        .log_file(config.log_file_path())
        .rewards_history(config.rewards_history());
    let builder = match (role, from_genesis) {
        (Role::Passive, _) => builder.genesis_block_hash(config.genesis_block_hash()),
        (Role::Leader, FromGenesis::File) => builder
            .genesis_block_path(config.genesis_block_path())
            .leader_with_secrets(config.secret_model_paths()),
        (Role::Leader, FromGenesis::Hash) => builder
            .genesis_block_hash(config.genesis_block_hash())
            .leader_with_secrets(config.secret_model_paths()),
    };
    builder.command()
}
