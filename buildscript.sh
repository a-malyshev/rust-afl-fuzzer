#!/bin/sh

rustup default stable
cd sources
cargo clean && cargo build
cp target/debug/fuzz ../test-app
cd ../test-app

make clean; make build
./fuzz -show
