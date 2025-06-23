```
RUST_LOG=debug cargo run --example test_pass
```

```
cargo build --example test_pass_wasm --target wasm32-unknown-unknown
wasm-bindgen --out-dir ./examples/wasm --target web ./target/wasm32-unknown-unknown/debug/examples/test_pass_wasm.wasm
```