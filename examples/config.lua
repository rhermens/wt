wt.copy({ src = ".env" })
wt.copy({ glob = "**/.env*", glob_ignore = ".worktrees/**" })
wt.link({ src = "node_modules" })

wt.command("echo $GIT_DIR")

wt.tmux.session(true)
wt.tmux.window("")
wt.tmux.window("nvim")
wt.tmux.window("")
wt.tmux.window("opencode --prompt=" .. wt.args[0])
