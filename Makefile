lint:
	RUSTFLAGS="-D warnings" cargo check
	RUSTFLAGS="-D warnings" cargo clippy

fmt::
	cargo fmt
	cargo clippy --fix

arilou:
	CHECK_LENGTH=false cargo run ../arilou.mod

hyperspace:
	TICK_MULTIPLIER=10 cargo run ../hyperspace.mod

kk:
	cargo run ../knulla-kuk.mod

kk3:
	PLAY_CHANS=3 cargo run ../knulla-kuk.mod

mycon:
	cargo run ../Mycon.mod

yehat:
	cargo run ../Yehat.mod

thraddash:
	TICK_MULTIPLIER=10 cargo run ../thraddash.mod
