all:


check:
	cargo check

build-release:
	cargo build --release

run-release:
	cargo run --release

arm-check:
	cargo check --target=arm-unknown-linux-musleabi

arm-build-release:
	cargo build --target=arm-unknown-linux-musleabi --release
