```
cargo run --example test_window
```

```
cargo build --example test_window_wasm --target wasm32-unknown-unknown
wasm-bindgen --out-dir ./examples/wasm --target web ./target/wasm32-unknown-unknown/debug/examples/test_window_wasm.wasm
```