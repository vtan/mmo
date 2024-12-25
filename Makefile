.PHONY: run-server
run-server:
	CARGO_TARGET_DIR=${PWD}/server/target \
		RUST_LOG=debug \
		cargo run --bin mmo-server

.PHONY: run-client
run-client:
	CARGO_TARGET_DIR=${PWD}/client/target \
		CARGO_PROFILE_DEV_OPT_LEVEL=1 \
		npm run start

.PHONY: release
release:
	rm -rf target/dist
	CARGO_TARGET_DIR=${PWD}/client/target \
		CARGO_PROFILE_RELEASE_LTO=true \
		CARGO_PROFILE_RELEASE_STRIP=debuginfo \
		npm run build
	CARGO_TARGET_DIR=${PWD}/server/target \
		CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=aarch64-linux-gnu-gcc \
		CARGO_PROFILE_RELEASE_LTO=true \
		RUSTFLAGS='-C target-feature=+neon' \
		cargo build --package mmo-server --release --target aarch64-unknown-linux-musl
	cp server/target/aarch64-unknown-linux-musl/release/mmo-server target/dist/
	cp -r assets data target/dist/
