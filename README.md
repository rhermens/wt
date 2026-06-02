# wt

A CLI tool that automates common setup tasks when creating or switching to a Git worktree. It can automatically copy files, create symlinks, run shell commands, and spawn a tmux session — keeping each worktree environment ready to use.

## How it works

The `wt` binary creates a new Git worktree at a given path (or opens it if it already exists), then applies the configured actions relative to the main repository root.

**Actions:**

- **copy** — Copy files from the main working tree into the worktree. Supports explicit paths or glob patterns
- **link** — Create symlinks from the worktree pointing to files/directories in the main tree. Supports explicit paths or glob patterns
- **commands** — Run arbitrary shell commands after checkout (e.g. build steps, environment setup)
- **tmux** — Optionally create a named tmux session with additional windows for the new worktree

## Usage

```sh
wt <path> [-- <args>...]
```

Creates (or opens) a worktree at `<path>`, then runs the configured copy, link, command, and tmux actions.

Any arguments after `--` are passed through to the Lua configuration script and are accessible via `wt.args` (0-indexed).

## Installation

Build and install the binary:

```sh
cargo install --path .
```

## Configuration

Settings are written in **Lua** and loaded from the following locations (in order, last wins):

| Location | Description |
|---|---|
| `~/.config/wt/config.lua` | User-level defaults applied to all repositories |
| `<repo>/.wt.lua` | Per-repository configuration |

### Lua API

The following globals and functions are available in your configuration script:

#### Variables

- `wt.args` — Table of CLI arguments passed after `--` (0-indexed). E.g. `wt.args[0]`, `wt.args[1]`, etc.

#### Functions

- `wt.worktrees_directory("<path>")` — Set the directory where worktrees are created. Defaults to the repository root.
- `wt.copy({ src = "<path>" })` — Copy a file or directory from the main tree into the worktree.
- `wt.copy({ glob = "<pattern>", glob_ignore = "<pattern>" })` — Copy files matching a glob pattern. `glob_ignore` is optional and excludes matching paths.
- `wt.link({ src = "<path>" })` — Create a symlink in the worktree pointing to a file or directory in the main tree.
- `wt.link({ glob = "<pattern>", glob_ignore = "<pattern>" })` — Create symlinks for files matching a glob pattern. `glob_ignore` is optional and excludes matching paths.
- `wt.command("<shell_command>")` — Run a shell command inside the worktree directory.
- `wt.tmux.session(true | false)` — Enable or disable creating a tmux session for the worktree.
- `wt.tmux.window("<command>")` — Add an additional tmux window. An empty string `""` creates a blank shell window. Each window runs the given command and then falls back to `$SHELL`.

### Example config

```lua
-- Store all worktrees under .worktrees/ inside the repo
wt.worktrees_directory(".worktrees")

-- Copy .env from the main tree so secrets are available in the worktree
wt.copy({ src = ".env" })

-- Copy all .env* files recursively, except inside .worktrees/
wt.copy({ glob = "**/.env*", glob_ignore = ".worktrees/**" })

-- Symlink node_modules to avoid reinstalling dependencies
wt.link({ src = "node_modules" })

-- Run a command after checkout
wt.command("echo $GIT_DIR")

-- Create a tmux session with extra windows
wt.tmux.session(true)
wt.tmux.window("")           -- blank shell window
wt.tmux.window("nvim")       -- open neovim
wt.tmux.window("")
wt.tmux.window(wt.args[0] and string.format("claude \"%s\"", wt.args[0]) or "claude")
```

See [`examples/config.lua`](examples/config.lua) for a reference configuration.
