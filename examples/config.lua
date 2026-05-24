wt.copy({ src = ".env" })
wt.link({ src = "node_modules" })

wt.command("echo $GIT_DIR")
wt.command("ls -lah")

wt.tmux.session(true)
wt.tmux.window("")
wt.tmux.window("nvim")
wt.tmux.window("")
wt.tmux.window("opencode --prompt=" .. wt.args[0])
