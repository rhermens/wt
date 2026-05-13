# git-worktree-utils

A `post-checkout` Git hook that automates common setup tasks when switching to a Git worktree. When you check out a worktree, it can automatically copy files, create symlinks, and run shell commands — keeping each worktree environment ready to use.

## How it works

The hook is triggered by Git's `post-checkout` event. It detects whether the checkout occurred in a worktree (not the main working tree), then applies the configured actions relative to the main repository root.

**Actions:**

- **copy** — Copy files from the main working tree into the worktree (e.g. `.env` secrets that shouldn't be committed)
- **link** — Create symlinks from the worktree pointing to files/directories in the main tree (e.g. `node_modules` to avoid redundant installs)
- **commands** — Run arbitrary shell commands after checkout (e.g. build steps, environment setup)

## Installation

```sh
make install
```

This builds the binary in release mode, installs it to `~/.config/git/hooks/post-checkout`, and sets that directory as the global Git hooks path.

## Configuration

Settings are loaded from the following locations (in order, last wins):

| Location | Description |
|---|---|
| `$XDG_CONFIG_HOME/worktree-utils/config.yaml` | User-level defaults applied to all repositories |
| `$GIT_DIR/.worktree.yaml` | Per-repository configuration |

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
```

See [`examples/config.yaml`](examples/config.yaml) for a reference configuration.
