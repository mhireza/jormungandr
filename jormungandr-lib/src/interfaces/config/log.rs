use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Log(pub Vec<LogEntry>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub format: String,
    pub level: String,
    pub output: LogOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogOutput {
    Stdout,
    Stderr,
    File(PathBuf),
}

impl Log {
    pub fn file_path(&self) -> Option<&Path> {
        self.0.iter().find_map(|log_entry| match &log_entry.output {
            LogOutput::File(path) => Some(path.as_path()),
            _ => None,
        })
    }

    /// Finds the first `LogOutput::File` entry
    /// and updates its path to the one given.
    /// If no such entry is found, this function does nothing.
    pub fn update_file_path(&mut self, path: impl Into<PathBuf>) {
        for log_entry in self.0.iter_mut() {
            match &mut log_entry.output {
                LogOutput::File(path_buf) => {
                    *path_buf = path.into();
                    break;
                }
                _ => {}
            }
        }
    }
}
