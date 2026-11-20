use std::sync::{Arc, Mutex};

use log::{info, warn};
use mlua::{Lua, LuaSerdeExt, Table};
use tmux_interface::{
    MoveWindow, NewSession, NewWindow, SetOption, SplitWindow, Tmux, TmuxCommands,
};

use crate::{error::Error, git::WorktreeContext};

#[derive(Debug, Default)]
pub struct TmuxOptions {
    pub create_session: bool,
    pub additional_windows: Vec<Vec<String>>,
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
            lua.create_function(move |lua, value: mlua::Value| {
                let commands = match value {
                    mlua::Value::Nil => vec![String::new()],
                    mlua::Value::String(command) => vec![command.to_str()?.to_owned()],
                    value => lua.from_value::<Vec<String>>(value)?,
                };
                s.lock().unwrap().tmux.additional_windows.push(commands);
                Ok(())
            })?,
        )?;

        Ok(tmux)
    }
}

impl SpawnsSessions for WorktreeContext {
    fn spawn_tmux_session(&self) -> Vec<Error> {
        if !self.settings.tmux.create_session {
            return vec![];
        }

        let worktree_name = match self.worktree_path.file_name() {
            Some(n) => n.to_string_lossy().into_owned(),
            None => return vec![Error::InvalidPath(self.worktree_path.clone())],
        };

        let main_name = match self.main_path.file_name() {
            Some(n) => n.to_string_lossy().into_owned(),
            None => return vec![Error::InvalidPath(self.main_path.clone())],
        };

        // tmux parses `.` and `:` in targets as window/pane separators.
        let session_name = format!("{} ({})", worktree_name, main_name).replace(['.', ':'], "_");
        let session_target = format!("={}:", session_name);
        let start_dir = self.worktree_path.display().to_string();

        let shell_command = |command: &str| {
            if command.is_empty() {
                "exec $SHELL".to_owned()
            } else {
                format!("{}; exec $SHELL", command)
            }
        };
        let mut windows = self
            .settings
            .tmux
            .additional_windows
            .iter()
            .filter_map(|commands| commands.split_first());
        let first_window = windows.next();
        let mut new_session = NewSession::new()
            .session_name(&session_name)
            .detached()
            .start_directory(&start_dir);
        if let Some((command, _)) = first_window {
            new_session = new_session.shell_command(shell_command(command));
        }

        let mut commands = TmuxCommands::new();
        commands.push(new_session);

        for (index, (command, pane_commands)) in first_window.into_iter().chain(windows).enumerate()
        {
            if index > 0 {
                commands.push(
                    NewWindow::new()
                        .target_window(&session_target)
                        .start_directory(&start_dir)
                        .shell_command(shell_command(command)),
                );
            }
            for command in pane_commands {
                commands.push(
                    SplitWindow::new()
                        .vertical()
                        .target_pane(&session_target)
                        .start_directory(&start_dir)
                        .shell_command(shell_command(command)),
                );
            }
        }

        match Tmux::with_commands(commands).status() {
            Err(e) => vec![Error::TmuxError(e.to_string())],
            Ok(status) if status.success() => {
                info!("Tmux session spawned");
                vec![]
            }
            Ok(status) => {
                warn!("Tmux exited with {}", status);
                vec![]
            }
        }
    }
}
