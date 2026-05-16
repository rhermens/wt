use std::path::PathBuf;

use config::{Config, ConfigError, File};
use directories::ProjectDirs;
use log::info;
use regex::Regex;
use serde::Deserialize;

use crate::error::Error;

#[derive(Debug, Deserialize, Default)]
pub struct Settings {
    pub copy: Vec<String>,
    pub link: Vec<String>,
    pub commands: Vec<String>,
    pub tmux: TmuxOptions,
}

#[derive(Debug, Deserialize, Default)]
pub struct TmuxOptions {
    pub create_session: bool,
    pub additional_windows: Vec<String>,
}

pub fn init_log() {
    env_logger::Builder::from_default_env()
        .filter(None, log::LevelFilter::Info)
        .init();
}

impl Settings {
    pub fn substitute(mut self, substitutions: &[String]) -> Result<Self, Error> {
        let regex = Regex::new(r"\$\d+").unwrap();

        self.commands = self
            .commands
            .iter()
            .map(|command| Self::substitute_args(&command, &regex, substitutions))
            .collect::<Result<Vec<String>, Error>>()?;
        self.tmux.additional_windows = self
            .tmux
            .additional_windows
            .iter()
            .map(|command| Self::substitute_args(&command, &regex, substitutions))
            .collect::<Result<Vec<String>, Error>>()?;

        Ok(self)
    }

    pub fn substitute_args(
        template: &str,
        pattern: &Regex,
        args: &[String],
    ) -> Result<String, Error> {
        let mut ret = template.to_string();
        for (index, arg) in args.into_iter().enumerate() {
            ret = ret.replace(&format!("${}", index + 1).to_string(), arg);
        }

        if pattern.is_match(&ret) {
            return Err(Error::MissingSubstitutions);
        }

        return Ok(ret);
    }

    pub fn new(path: &PathBuf) -> Result<Settings, Error> {
        Self::load_settings(path).map_err(|e| Error::InvalidSettings { source: e })
    }

    fn load_settings(cwd: &PathBuf) -> Result<Settings, ConfigError> {
        let mut config = Config::builder()
            .set_default("copy", Vec::<String>::new())?
            .set_default("link", Vec::<String>::new())?
            .set_default("commands", Vec::<String>::new())?
            .set_default("tmux.create_session", false)?
            .set_default("tmux.additional_windows", Vec::<String>::new())?
            .add_source(File::with_name(&cwd.join(".wt").display().to_string()).required(false));

        if let Some(dirs) = ProjectDirs::from("", "", "wt")
            && !cfg!(debug_assertions)
        {
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
}
