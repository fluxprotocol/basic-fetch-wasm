cargo build --target wasm32-wasi --release
wasm-opt -Oz -o ./target/wasm32-wasi/release/basic-fetch.wasm ./target/wasm32-wasi/release/simple-url-call.wasm
cp ./target/wasm32-wasi/release/basic-fetch.wasm ./res/