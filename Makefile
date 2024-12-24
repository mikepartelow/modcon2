lint:
	RUSTFLAGS="-D warnings" cargo check
	RUSTFLAGS="-D warnings" cargo clippy

fmt::
	cargo fmt
	cargo clippy --fix

arilou:
	CHECK_LENGTH=false cargo run ./mods/arilou.mod

hyperspace:
	TICK_MULTIPLIER=11 cargo run ./mods/hyperspace.mod

kk:
	cargo run ./mods/knulla-kuk.mod

kk3:
	PLAY_CHANS=3 cargo run ./mods/knulla-kuk.mod

mycon:
	cargo run ./mods/Mycon.mod

yehat:
	cargo run ./mods/Yehat.mod

thraddash:
	TICK_MULTIPLIER=10 cargo run ./mods/thraddash.mod
