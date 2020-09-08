all:



arm-check:
	cargo check --target=arm-unknown-linux-musleabi

arm-build-release:
	cargo build --target=arm-unknown-linux-musleabi --release
