```
cargo build --example test_shared_array_buffer --target wasm32-unknown-unknown
wasm-bindgen --out-dir ./examples/wasm --target web ../target/wasm32-unknown-unknown/debug/examples/test_shared_array_buffer.wasm
```
