.PHONY: run-server
run-server:
	CARGO_TARGET_DIR=${PWD}/server/target \
	RUST_LOG=debug \
	cargo run --bin mmo-server

.PHONY: run-client
run-client:
	CARGO_TARGET_DIR=${PWD}/client/target \
	npm run start
