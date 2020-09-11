#!/bin/sh

cargo clean && cargo build
cd ./examples/example1
make clean; make build
cd ../../

./target/debug/fuzz -d=./examples/example1 -show example.c
