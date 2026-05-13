use std::path::PathBuf;

use config::{Config, ConfigError, File};
use directories::ProjectDirs;
use log::info;
use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct Settings {
    pub copy: Vec<String>,
    pub link: Vec<String>,
}

pub fn init_log() {
    env_logger::Builder::from_default_env()
        .filter(None, log::LevelFilter::Info)
        .init();
}

pub fn load_settings(cwd: &PathBuf) -> Result<Settings, ConfigError> {
    let defaults = Settings::default();
    let mut config = Config::builder()
        .set_default("copy", defaults.copy)
        .unwrap()
        .add_source(File::with_name(&cwd.join(".wtutils").display().to_string()).required(false));

    if let Some(dirs) = ProjectDirs::from("", "", "wtutils") {
        config = config.add_source(
            File::with_name(&dirs.config_dir().join("wtutils").display().to_string())
                .required(false),
        );
    };

    if cfg!(debug_assertions) {
        info!("Adding config from {}", env!("CARGO_MANIFEST_DIR"));
        config = config.add_source(
            File::with_name(
                &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("examples/wtutils")
                    .display()
                    .to_string(),
            )
            .required(false),
        );
    }

    let settings = config.build()?;
    settings.try_deserialize()
}
