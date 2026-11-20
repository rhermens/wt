wt.worktrees_directory("~/.worktrees")

wt.copy({ src = ".env" })
wt.copy({ glob = "**/.env*", glob_ignore = ".worktrees/**" })
wt.link({ src = "node_modules" })

wt.command("echo $GIT_DIR")

wt.tmux.session(true)
wt.tmux.window(nil)
wt.tmux.window("nvim")
wt.tmux.window({ "", "" }) -- two stacked shell panes
wt.tmux.window(wt.args[0] and string.format("claude \"%s\"", wt.args[0]) or "claude")
