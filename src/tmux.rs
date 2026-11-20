use std::sync::{Arc, Mutex};

use mlua::{Lua, Table};

#[derive(Debug, Default)]
pub struct TmuxOptions {
    pub create_session: bool,
    pub additional_windows: Vec<String>,
}

pub trait SpawnsSessions {
    fn spawn_tmux_session(&self) -> Vec<crate::error::Error>;
}

impl TmuxOptions {
    pub fn create_lua_table(
        lua: &Lua,
        settings: &Arc<Mutex<crate::settings::Settings>>,
    ) -> Result<Table, mlua::Error> {
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

        Ok(tmux)
    }
}
