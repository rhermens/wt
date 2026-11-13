use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "wt",
    version,
    about = "Automate Git worktree setup",
    long_about = "Create a Git worktree at the given path, or open it if it already exists, \
then apply the configured copy, link, shell command, and tmux actions. \
Copy and link sources are relative to the main repository root and support glob patterns.\n\n\
Configuration is written in Lua and loaded from ~/.config/wt/config.lua, \
then <repo>/.wt.lua. Later settings override earlier settings."
)]
pub struct Args {
    /// Directory from which to discover the Git repository (defaults to the current directory)
    #[arg(long, value_name = "DIRECTORY")]
    pub cwd: Option<PathBuf>,

    /// Path of the worktree to create or open
    ///
    /// Relative paths use the directory set by wt.worktrees_directory(),
    /// which defaults to the repository root. Absolute paths are used as given.
    #[arg(index = 1, required = true)]
    pub path: PathBuf,

    /// Arguments passed to the Lua configuration as the 0-indexed wt.args table
    ///
    /// Use -- to separate these arguments from wt options. Access them in Lua
    /// as wt.args[0], wt.args[1], and so on.
    #[arg(index = 2, num_args = 1..)]
    pub args: Vec<String>,
}
