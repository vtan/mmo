#!/bin/bash

set -e

cargo build -p mmo-client --target=wasm32-unknown-unknown
~/.cargo/bin/wasm-bindgen target/wasm32-unknown-unknown/debug/mmo_client.wasm --target=web --out-dir=client-browser/src/wasm-bindgen/
cp client-browser/src/wasm-bindgen/mmo_client_bg.wasm client-browser/webroot/client.wasm
