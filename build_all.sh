#!/bin/bash
set -e

echo "🔧 Building native workspace..."
cargo build

echo "🌐 Building WASM smart contracts..."
cargo build --release --target wasm32-unknown-unknown -p learncoin-contracts --bins

