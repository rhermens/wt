use std::path::PathBuf;

use config::{Config, ConfigError, File};
use directories::ProjectDirs;
use log::info;
use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct Settings {
    pub copy: Option<Vec<String>>,
    pub link: Option<Vec<String>>,
    pub commands: Option<Vec<String>>,
}

pub fn init_log() {
    env_logger::Builder::from_default_env()
        .filter(None, log::LevelFilter::Info)
        .init();
}

pub fn load_settings(cwd: &PathBuf) -> Result<Settings, ConfigError> {
    let mut config = Config::builder()
        .add_source(File::with_name(&cwd.join(".worktree").display().to_string()).required(false));

    if let Some(dirs) = ProjectDirs::from("", "", "worktree-utils") {
        config = config.add_source(
            File::with_name(&dirs.config_dir().join("config").display().to_string())
                .required(false),
        );
    };

    if cfg!(debug_assertions) {
        info!("Adding config from {}", env!("CARGO_MANIFEST_DIR"));
        config = config.add_source(
            File::with_name(
                &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("examples/config")
                    .display()
                    .to_string(),
            )
            .required(false),
        );
    }

    let settings = config.build()?;
    settings.try_deserialize()
}
