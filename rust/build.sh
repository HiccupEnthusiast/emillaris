#!/bin/bash
cargo +nightly build --features nothreads -Zbuild-std --target wasm32-unknown-emscripten
RUSTFLAGS="-C link-args=-pthread" cargo +nightly build -Zbuild-std --target wasm32-unknown-emscripten
mv target/debug/emillaris_rs.wasm target/debug/emillaris_rs.threads.wasm

