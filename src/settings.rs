use std::path::{Path, PathBuf};

use config::{Config, ConfigError, File};
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
            .map(|command| Self::substitute_args(command, &regex, substitutions))
            .collect::<Result<Vec<String>, Error>>()?;
        self.tmux.additional_windows = self
            .tmux
            .additional_windows
            .iter()
            .map(|command| Self::substitute_args(command, &regex, substitutions))
            .collect::<Result<Vec<String>, Error>>()?;

        Ok(self)
    }

    pub fn substitute_args(
        template: &str,
        pattern: &Regex,
        args: &[String],
    ) -> Result<String, Error> {
        let mut missing = false;
        let result = pattern.replace_all(template, |caps: &regex::Captures| {
            let placeholder = &caps[0]; // e.g. "$1"
            match placeholder[1..].parse::<usize>() {
                Ok(n) if n >= 1 && n <= args.len() => args[n - 1].clone(),
                _ => {
                    missing = true;
                    placeholder.to_string()
                }
            }
        });

        if missing {
            return Err(Error::MissingSubstitutions);
        }

        Ok(result.into_owned())
    }

    pub fn new(path: &Path) -> Result<Settings, Error> {
        Self::new_internal(path).map_err(|e| Error::InvalidSettings { source: e })
    }

    fn new_internal(cwd: &Path) -> Result<Settings, ConfigError> {
        let mut config = Config::builder()
            .set_default("copy", Vec::<String>::new())?
            .set_default("link", Vec::<String>::new())?
            .set_default("commands", Vec::<String>::new())?
            .set_default("tmux.create_session", false)?
            .set_default("tmux.additional_windows", Vec::<String>::new())?;

        if let Some(home) = std::env::home_dir()
            && !cfg!(debug_assertions)
        {
            config = config.add_source(
                File::with_name(&home.join(".config/wt/config").display().to_string())
                    .required(false),
            );
        };

        config = config
            .add_source(File::with_name(&cwd.join(".wt").display().to_string()).required(false));

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
