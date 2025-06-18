```
cargo run --example test_instance
```

```
cargo build --example test_instance --target wasm32-unknown-unknown
wasm-bindgen --out-dir ./examples/wasm --target web ./target/wasm32-unknown-unknown/debug/examples/test_instance.wasm
```