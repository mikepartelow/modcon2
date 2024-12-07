fmt:
	cargo clippy --fix

kk:
	RUST_LOG=info RUST_BACKTRACE=1 cargo run ../knulla-kuk.mod

kk3:
	PLAY_CHANS=3 RUST_LOG=info RUST_BACKTRACE=1 cargo run ../knulla-kuk.mod
