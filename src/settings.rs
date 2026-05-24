use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use log::info;
use mlua::{Lua, LuaSerdeExt};
use serde::Deserialize;

use crate::error::Error;

#[derive(Debug, Default)]
pub struct Settings {
    pub copy: Vec<IoOperation>,
    pub link: Vec<IoOperation>,
    pub commands: Vec<String>,
    pub tmux: TmuxOptions,
}

#[derive(Debug, Default, Deserialize)]
pub struct IoOperation {
    pub src: PathBuf,
}

#[derive(Debug, Default)]
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
    pub fn new(main_path: &Path, worktree_path: &Path, args: &[String]) -> Result<Settings, Error> {
        let settings = Arc::new(Mutex::new(Settings::default()));

        let lua = Lua::new();
        Self::register_api(&lua, &settings, main_path, worktree_path, args)
            .map_err(|e| Error::LuaError { source: e })?;

        for source in Self::config_sources(main_path)? {
            let script = std::fs::read_to_string(&source).map_err(|e| Error::IoError {
                path: source.clone(),
                source: e,
            })?;
            lua.load(&script)
                .set_name(source.display().to_string())
                .exec()
                .map_err(|e| Error::LuaError { source: e })?;
        }

        drop(lua);

        let settings = Arc::try_unwrap(settings)
            .expect("Reference error")
            .into_inner()
            .expect("Poison error");

        Ok(settings)
    }

    fn register_api(
        lua: &Lua,
        settings: &Arc<Mutex<Settings>>,
        main_path: &Path,
        worktree_path: &Path,
        cli_args: &[String],
    ) -> Result<(), mlua::Error> {
        let globals = lua.globals();

        let wt = lua.create_table()?;
        wt.set(
            "args",
            lua.create_table_from(cli_args.iter().map(|s| s.as_str()).enumerate())?,
        )?;

        wt.set("main_path", main_path.to_string_lossy().as_ref())?;
        wt.set("worktree_path", worktree_path.to_string_lossy().as_ref())?;

        let s = Arc::clone(settings);
        wt.set(
            "copy",
            lua.create_function(move |lua, op: mlua::Value| {
                s.lock().unwrap().copy.push(lua.from_value(op)?);
                Ok(())
            })?,
        )?;

        let s = Arc::clone(settings);
        wt.set(
            "link",
            lua.create_function(move |lua, op: mlua::Value| {
                s.lock().unwrap().link.push(lua.from_value(op)?);
                Ok(())
            })?,
        )?;

        let s = Arc::clone(settings);
        wt.set(
            "command",
            lua.create_function(move |_, cmd: String| {
                s.lock().unwrap().commands.push(cmd);
                Ok(())
            })?,
        )?;

        let tmux = lua.create_table()?;

        let s = Arc::clone(settings);
        tmux.set(
            "session",
            lua.create_function(move |_, enabled: bool| {
                s.lock().unwrap().tmux.create_session = enabled;
                Ok(())
            })?,
        )?;

        let s = Arc::clone(settings);
        tmux.set(
            "window",
            lua.create_function(move |_, cmd: String| {
                s.lock().unwrap().tmux.additional_windows.push(cmd);
                Ok(())
            })?,
        )?;

        wt.set("tmux", tmux)?;

        globals.set("wt", wt)?;

        Ok(())
    }

    fn config_sources(main_path: &Path) -> Result<Vec<PathBuf>, Error> {
        let mut sources = Vec::new();

        if !cfg!(debug_assertions) {
            if let Some(home) = std::env::home_dir() {
                let global = home.join(".config/wt/config.lua");
                if global.exists() {
                    sources.push(global);
                }
            }
        }

        let local = main_path.join(".wt.lua");
        if local.exists() {
            sources.push(local);
        }

        if cfg!(debug_assertions) {
            let debug = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/config.lua");
            if debug.exists() {
                info!("Adding config from {}", env!("CARGO_MANIFEST_DIR"));
                sources.push(debug);
            }
        }

        Ok(sources)
    }
}
