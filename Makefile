install:
	cargo build --release
	mkdir -p ~/.config/git/hooks
	rm ~/.config/git/hooks/post-checkout
	cp ./target/release/post-checkout ~/.config/git/hooks/
	git config --global git.hooksPath ~/.config/git/hooks/
