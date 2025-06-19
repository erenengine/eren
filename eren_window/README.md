```
RUST_LOG=debug cargo run --example test_window
```

```
cargo build --example test_window --target wasm32-unknown-unknown
wasm-bindgen --out-dir ./examples/wasm --target web ./target/wasm32-unknown-unknown/debug/examples/test_window.wasm
```
