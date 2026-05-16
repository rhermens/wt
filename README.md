# wt

A CLI tool that automates common setup tasks when creating or switching to a Git worktree. It can automatically copy files, create symlinks, run shell commands, and spawn a tmux session — keeping each worktree environment ready to use.

## How it works

The `wt` binary creates a new Git worktree at a given path (or opens it if it already exists), then applies the configured actions relative to the main repository root.

**Actions:**

- **copy** — Copy files from the main working tree into the worktree (e.g. `.env` secrets that shouldn't be committed)
- **link** — Create symlinks from the worktree pointing to files/directories in the main tree (e.g. `node_modules` to avoid redundant installs)
- **commands** — Run arbitrary shell commands after checkout (e.g. build steps, environment setup)
- **tmux** — Optionally create a named tmux session with additional windows for the new worktree

## Usage

```sh
wt <path> -- <command_substitions>
```

Creates (or opens) a worktree at `<path>`, then runs the configured copy, link, command, and tmux actions.

## Installation

Build and install the binary:

```sh
cargo install --path .
```

## Configuration

Settings are loaded from the following locations (in order, last wins):

| Location | Description |
|---|---|
| `$XDG_CONFIG_HOME/wt/config.yaml` | User-level defaults applied to all repositories |
| `$GIT_DIR/.wt.yaml` | Per-repository configuration |

### Config format

```yaml
# Files to copy from the main worktree into each new worktree
copy:
  - .env

# Files/directories to symlink from the main worktree into each new worktree
link:
  - node_modules

# Shell commands to run after checkout
commands:
  - npm run build

# Tmux session options
tmux:
  create_session: true
  additional_windows:
    - ""        # blank shell window
    - "nvim"    # open neovim
    - ""
    - "opencode --prompt=$1" # Each placeholder is replaced with <command_substitions>
```

See [`examples/config.yaml`](examples/config.yaml) for a reference configuration.
