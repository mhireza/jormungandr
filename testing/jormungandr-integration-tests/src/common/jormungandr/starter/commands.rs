use super::{FromGenesis, Role};
use crate::common::legacy::BackwardCompatibleConfig;
use std::fs::File;
use std::path::PathBuf;
use std::process::{Command, Stdio};
pub struct JormungandrStarterCommands {
    jormungandr_app: PathBuf,
}

impl JormungandrStarterCommands {
    pub fn from_app(app: PathBuf) -> Self {
        Self {
            jormungandr_app: app,
        }
    }

    pub fn as_leader_node_from_hash(
        &self,
        config_path: &PathBuf,
        genesis_block_hash: &str,
        secret_paths: &[PathBuf],
        log_file_path: Option<PathBuf>,
        reward_history: bool,
    ) -> Command {
        let mut command = Command::new(self.jormungandr_app.as_os_str());
        for secret_path in secret_paths {
            command.arg("--secret").arg(secret_path.as_os_str());
        }

        if reward_history {
            command.arg("--rewards-report-all");
        }

        command
            .arg("--config")
            .arg(config_path.as_os_str())
            .arg("--genesis-block-hash")
            .arg(genesis_block_hash);

        if let Some(log_file) = log_file_path {
            command.stderr(Self::get_stdio_from_log_file(&log_file));
        }

        println!("Running start jormungandr command: {:?}", &command);
        command
    }

    pub fn as_leader_node(
        &self,
        config_path: &PathBuf,
        genesis_block_path: &PathBuf,
        secret_paths: &[PathBuf],
        log_file_path: Option<PathBuf>,
        reward_history: bool,
    ) -> Command {
        let mut command = Command::new(self.jormungandr_app.as_os_str());
        for secret_path in secret_paths {
            command.arg("--secret").arg(secret_path.as_os_str());
        }

        if reward_history {
            command.arg("--rewards-report-all");
        }

        command
            .arg("--config")
            .arg(config_path.as_os_str())
            .arg("--genesis-block")
            .arg(genesis_block_path.as_os_str());

        if let Some(log_file) = log_file_path {
            command.stderr(Self::get_stdio_from_log_file(&log_file));
        }

        println!("Running start jormungandr command: {:?}", &command);
        command
    }

    pub fn as_passive_node(
        &self,
        config_path: &PathBuf,
        genesis_block_hash: &String,
        log_file_path: Option<PathBuf>,
        reward_history: bool,
    ) -> Command {
        let mut command = Command::new(self.jormungandr_app.as_os_str());

        if reward_history {
            command.arg("--rewards-report-all");
        }

        command
            .arg("--config")
            .arg(config_path.as_os_str())
            .arg("--genesis-block-hash")
            .arg(&genesis_block_hash);

        if let Some(log_file) = log_file_path {
            command.stderr(Self::get_stdio_from_log_file(&log_file));
        }

        println!("Running start jormungandr command: {:?}", &command);
        command
    }

    #[cfg(windows)]
    fn get_stdio_from_log_file(log_file_path: &PathBuf) -> std::process::Stdio {
        use std::os::windows::io::{FromRawHandle, IntoRawHandle};
        let file = File::create(log_file_path).expect("couldn't create log file for jormungandr");
        unsafe { Stdio::from_raw_handle(file.into_raw_handle()) }
    }

    #[cfg(unix)]
    fn get_stdio_from_log_file(log_file_path: &PathBuf) -> std::process::Stdio {
        use std::os::unix::io::{FromRawFd, IntoRawFd};
        let file = File::create(log_file_path).expect("couldn't create log file for jormungandr");
        unsafe { Stdio::from_raw_fd(file.into_raw_fd()) }
    }
}

pub fn get_command(
    config: &BackwardCompatibleConfig,
    jormungandr_app_path: PathBuf,
    role: Role,
    from_genesis: FromGenesis,
) -> Command {
    let commands = JormungandrStarterCommands::from_app(jormungandr_app_path);
    match (role, from_genesis) {
        (Role::Passive, _) => commands.as_passive_node(
            &config.node_config_path,
            &config.genesis_block_hash,
            config.log_file_path(),
            config.rewards_history,
        ),
        (Role::Leader, FromGenesis::File) => commands.as_leader_node(
            &config.node_config_path,
            &config.genesis_block_path,
            &config.secret_model_paths,
            config.log_file_path(),
            config.rewards_history,
        ),
        (Role::Leader, FromGenesis::Hash) => commands.as_leader_node_from_hash(
            &config.node_config_path,
            &config.genesis_block_hash,
            &config.secret_model_paths,
            config.log_file_path(),
            config.rewards_history,
        ),
    }
}
