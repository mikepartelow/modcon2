fmt:
	cargo fmt
	cargo clippy --fix

hyperspace:
	cargo run ../hyperspace.mod

kk:
	cargo run ../knulla-kuk.mod

kk3:
	PLAY_CHANS=3 cargo run ../knulla-kuk.mod

mycon:
	cargo run ../Mycon.mod

yehat:
	cargo run ../Yehat.mod
